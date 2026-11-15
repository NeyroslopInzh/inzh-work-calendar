use std::sync::Mutex;
use std::time::Duration;

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager};

use crate::model::TelegramContact;
use crate::store::Store;

static LAST_POLL_ERROR: Mutex<String> = Mutex::new(String::new());

pub fn poll(app: &AppHandle) -> Result<(), String> {
    report(poll_once(app))
}

fn poll_once(app: &AppHandle) -> Result<(), String> {
    let store = app.state::<Store>();
    let token = store.settings().telegram_token.trim().to_string();
    if token.is_empty() {
        return Ok(());
    }
    if !valid_token(&token) {
        return Err("Токен выглядит не так, как токен BotFather".into());
    }
    let mut announced = false;
    for _ in 0..10 {
        let settings = store.settings();
        if settings.telegram_token != token {
            break;
        }
        let offset = settings.telegram_offset;
        let body = get(
            &token,
            "getUpdates",
            &format!("offset={offset}&timeout=0&allowed_updates=%5B%22message%22%5D"),
        )?;
        let (contacts, next) = read_updates(&body, offset)?;
        if contacts.is_empty() && next == offset {
            break;
        }
        if store.apply_telegram_contacts(&token, contacts, next)? {
            announced = true;
        }
        if next == offset {
            break;
        }
    }
    if announced {
        let _ = app.emit("telegram-changed", store.settings());
    }
    Ok(())
}

pub fn push(card_title: &str, text: String, token: String, chats: Vec<i64>) {
    if token.is_empty() || chats.is_empty() {
        return;
    }
    let title = card_title.to_string();
    std::thread::spawn(move || {
        for chat_id in chats {
            if let Err(err) = send_message(&token, chat_id, &text) {
                eprintln!("телеграм «{title}»: {err}");
            }
        }
    });
}

pub fn bot_username(token: &str) -> Result<String, String> {
    if !valid_token(token) {
        return Err("Токен выглядит не так, как токен BotFather".into());
    }
    let body = get(token, "getMe", "")?;
    let username = body
        .pointer("/result/username")
        .and_then(Value::as_str)
        .map(clean_username)
        .unwrap_or_default();
    if username.is_empty() {
        return Err("У бота нет имени. Задайте его в @BotFather".into());
    }
    Ok(username)
}

pub fn invite_code() -> String {
    let code = uuid::Uuid::new_v4().simple().to_string();
    code[..12].to_string()
}

pub fn valid_token(token: &str) -> bool {
    let Some((id, secret)) = token.split_once(':') else {
        return false;
    };
    (6..=20).contains(&id.len())
        && id.chars().all(|ch| ch.is_ascii_digit())
        && (20..=80).contains(&secret.len())
        && secret
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn send_message(token: &str, chat_id: i64, text: &str) -> Result<(), String> {
    let text: String = text.trim().chars().take(4000).collect();
    if text.is_empty() {
        return Ok(());
    }
    let _ = post(
        token,
        "sendMessage",
        json!({ "chat_id": chat_id, "text": text }),
    )?;
    Ok(())
}

fn get(token: &str, method: &str, query: &str) -> Result<Value, String> {
    let url = if query.is_empty() {
        format!("https://api.telegram.org/bot{token}/{method}")
    } else {
        format!("https://api.telegram.org/bot{token}/{method}?{query}")
    };
    let response = agent().get(&url).call().map_err(http_error)?;
    read_body(response)
}

fn post(token: &str, method: &str, body: Value) -> Result<Value, String> {
    let url = format!("https://api.telegram.org/bot{token}/{method}");
    let response = agent().post(&url).send_json(body).map_err(http_error)?;
    read_body(response)
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .build()
}

fn read_body(response: ureq::Response) -> Result<Value, String> {
    let value: Value = response
        .into_json()
        .map_err(|_| "Телеграм прислал не JSON".to_string())?;
    if value.get("ok").and_then(Value::as_bool) == Some(true) {
        return Ok(value);
    }
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("Телеграм отклонил запрос");
    Err(friendly(description))
}

fn http_error(err: ureq::Error) -> String {
    match err {
        ureq::Error::Status(code, response) => {
            let text = response.into_string().unwrap_or_default();
            telegram_description(&text).unwrap_or_else(|| format!("Телеграм ответил {code}"))
        }
        ureq::Error::Transport(_) => "нет связи с Телеграмом".into(),
    }
}

fn telegram_description(text: &str) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;
    value
        .get("description")
        .and_then(Value::as_str)
        .map(friendly)
}

fn friendly(description: &str) -> String {
    if description.eq_ignore_ascii_case("unauthorized") {
        "Токен не подошёл".into()
    } else {
        description.to_string()
    }
}

pub fn read_updates(
    body: &Value,
    previous_offset: i64,
) -> Result<(Vec<TelegramContact>, i64), String> {
    let result = body
        .get("result")
        .and_then(Value::as_array)
        .ok_or("Телеграм не прислал обновления")?;
    let mut contacts = Vec::new();
    let mut next = previous_offset;
    for update in result {
        if let Some(id) = update.get("update_id").and_then(Value::as_i64) {
            next = next.max(id.saturating_add(1));
        }
        if let Some(message) = update.get("message") {
            if let Some(contact) = message_contact(message) {
                contacts.push(contact);
            }
        }
    }
    Ok((contacts, next))
}

fn message_contact(message: &Value) -> Option<TelegramContact> {
    let chat = message.get("chat")?;
    if chat.get("type").and_then(Value::as_str) != Some("private") {
        return None;
    }
    let from = message.get("from").unwrap_or(chat);
    if from.get("is_bot").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let chat_id = chat.get("id")?.as_i64()?;
    if chat_id == 0 {
        return None;
    }
    let username = clean_username(from.get("username").and_then(Value::as_str).unwrap_or(""));
    let name = clean_name(
        from.get("first_name").and_then(Value::as_str).unwrap_or(""),
        from.get("last_name").and_then(Value::as_str).unwrap_or(""),
        &username,
    );
    Some(TelegramContact {
        chat_id,
        name,
        username,
    })
}

fn clean_username(value: &str) -> String {
    let value = value.trim().trim_start_matches('@');
    if (1..=32).contains(&value.chars().count())
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        value.to_string()
    } else {
        String::new()
    }
}

fn clean_name(first: &str, last: &str, username: &str) -> String {
    let name = [first.trim(), last.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if !name.is_empty() {
        return name.chars().take(80).collect();
    }
    if username.is_empty() {
        "без имени".into()
    } else {
        format!("@{username}")
    }
}

fn report(result: Result<(), String>) -> Result<(), String> {
    let mut last = LAST_POLL_ERROR
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    match &result {
        Ok(()) => last.clear(),
        Err(err) if last.as_str() != err => {
            eprintln!("телеграм: {err}");
            last.clone_from(err);
        }
        Err(_) => {}
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_shape() {
        assert!(valid_token("123456:AAEebdcefghijklmnopqrstuvwxyz12"));
        assert!(!valid_token("123456:short"));
        assert!(!valid_token("@someone"));
        assert!(!valid_token(""));
    }

    #[test]
    fn private_message_becomes_a_recipient_and_a_group_does_not() {
        let body = json!({
            "ok": true,
            "result": [
                {
                    "update_id": 10,
                    "message": {
                        "message_id": 1,
                        "from": {"id": 42, "is_bot": false, "first_name": "Анна", "last_name": "Иванова", "username": "anna"},
                        "chat": {"id": 42, "type": "private"},
                        "text": "/start abc"
                    }
                },
                {
                    "update_id": 11,
                    "message": {
                        "message_id": 2,
                        "from": {"id": 7, "is_bot": false, "first_name": "Группа"},
                        "chat": {"id": -100, "type": "group", "title": "чат"},
                        "text": "привет"
                    }
                },
                {
                    "update_id": 12,
                    "message": {
                        "message_id": 3,
                        "from": {"id": 1, "is_bot": true, "first_name": "бот"},
                        "chat": {"id": 42, "type": "private"},
                        "text": "эхо"
                    }
                }
            ]
        });
        let (contacts, next) = read_updates(&body, 0).unwrap();
        assert_eq!(next, 13);
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].chat_id, 42);
        assert_eq!(contacts[0].name, "Анна Иванова");
        assert_eq!(contacts[0].username, "anna");
    }
}
