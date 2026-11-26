use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use chrono::{
    DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc,
};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager};

use crate::model::{CardInput, TODO_ID};
use crate::store::Store;
use crate::telegram;

static LAST_POLL_ERROR: Mutex<String> = Mutex::new(String::new());
static SESSIONS: LazyLock<Mutex<HashMap<(i64, i64), Pending>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const SAMPLE: &str = "Наименование\nдд.мм.гггг чч:мм\nдд.мм.гггг\nКомментарий";
const DATE_HINT: &str = "нужна дата дд.мм.гггг. Время, если есть: чч:мм или чч-мм";
const UPDATES: &str =
    "timeout=0&allowed_updates=%5B%22message%22%2C%22channel_post%22%2C%22callback_query%22%5D";

const ASK_TITLE: &str = "Введите наименование.";
const ASK_START: &str = "Введите дату начала: дд.мм.гггг. Время чч:мм или чч-мм можно дописать. Если времени нет, возьму время этого сообщения.";
const ASK_DEADLINE: &str =
    "Введите дедлайн: дд.мм.гггг. Время так же можно дописать. Если времени нет, поставлю 23:00.";
const ASK_COMMENT: &str = "Введите комментарий или нажмите «Без комментария».";

pub fn forget_sessions() {
    SESSIONS
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clear();
}

pub fn poll(app: &AppHandle) -> Result<(), String> {
    report(poll_once(app))
}

fn poll_once(app: &AppHandle) -> Result<(), String> {
    let store = app.state::<Store>();
    let settings = store.settings();
    let token = settings.task_bot_token.trim().to_string();
    if token.is_empty() {
        return Ok(());
    }
    if !telegram::valid_token(&token) {
        return Err("Токен выглядит не так, как токен BotFather".into());
    }
    if !settings.telegram_token.is_empty() && settings.telegram_token == token {
        return Err("Бот задач совпадает с ботом напоминаний".into());
    }
    let username = settings.task_bot.clone();
    if !settings.task_bot_ready {
        let body = telegram::get(
            &token,
            "getUpdates",
            &format!("offset=-1&limit=1&{UPDATES}"),
        )?;
        let (_, offset) = read_batch(&body, 0)?;
        store.catch_up_task_bot(&token, offset)?;
        return Ok(());
    }
    for _ in 0..10 {
        let settings = store.settings();
        if settings.task_bot_token != token || !settings.task_bot_ready {
            break;
        }
        let offset = settings.task_bot_offset;
        let body = telegram::get(&token, "getUpdates", &format!("offset={offset}&{UPDATES}"))?;
        let (items, next) = read_batch(&body, offset)?;
        if items.is_empty() && next == offset {
            break;
        }
        for item in items {
            if let Err(err) = apply(app, &token, &username, item) {
                eprintln!("бот задач: {err}");
            }
        }
        store.advance_task_bot(&token, next)?;
        if next == offset {
            break;
        }
    }
    Ok(())
}

fn apply(app: &AppHandle, token: &str, username: &str, item: Raw) -> Result<(), String> {
    let (chat_id, user_id, reply_to, clock, event) = match item {
        Raw::Callback {
            id,
            chat_id,
            user_id,
            data,
        } => {
            let _ = telegram::post(
                token,
                "answerCallbackQuery",
                json!({ "callback_query_id": id }),
            );
            let event = match data.as_str() {
                "task:new" => Event::New,
                "task:skip" => Event::Skip,
                "task:cancel" => Event::Cancel,
                _ => return Ok(()),
            };
            (
                chat_id,
                user_id,
                None,
                hour_minute(Local::now().time()),
                event,
            )
        }
        Raw::Text {
            chat_id,
            user_id,
            message_id,
            unix,
            text,
        } => {
            if addressed_elsewhere(&text, username) {
                return Ok(());
            }
            (
                chat_id,
                user_id,
                Some(message_id),
                clock_of(unix),
                interpret(&text),
            )
        }
    };
    let key = (chat_id, user_id);
    let before = sessions().get(&key).cloned();
    let mut session = Session {
        pending: before.clone(),
    };
    let decision = handle(&mut session, event, clock);
    if let Some(draft) = &decision.save {
        if let Err(err) = save_draft(app, draft) {
            deliver(
                token,
                chat_id,
                reply_to,
                &err,
                keyboard_for(before.as_ref()),
            )?;
            return Ok(());
        }
    }
    if app.state::<Store>().settings().task_bot_token == token {
        let mut map = sessions();
        match &session.pending {
            Some(pending) => {
                map.insert(key, pending.clone());
            }
            None => {
                map.remove(&key);
            }
        }
    }
    deliver(token, chat_id, reply_to, &decision.text, decision.keyboard)
}

fn save_draft(app: &AppHandle, draft: &TaskDraft) -> Result<(), String> {
    let input = CardInput {
        id: None,
        title: draft.title.clone(),
        description: draft.description.clone(),
        status: TODO_ID.to_string(),
        start_at: Some(to_utc(draft.start)?),
        deadline_at: Some(to_utc(draft.deadline)?),
        recurrence: None,
        recipient_ids: None,
    };
    let cards = app.state::<Store>().save_card(input)?;
    let _ = app.emit("cards-changed", &cards);
    crate::schedule::sync_nag_window(app);
    Ok(())
}

fn to_utc(naive: NaiveDateTime) -> Result<DateTime<Utc>, String> {
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(local) => Ok(local.with_timezone(&Utc)),
        LocalResult::Ambiguous(_, _) => {
            Err("Это местное время повторяется из-за перевода часов.".into())
        }
        LocalResult::None => Err("Такого местного времени нет: часы переводили.".into()),
    }
}

fn deliver(
    token: &str,
    chat_id: i64,
    reply_to: Option<i64>,
    text: &str,
    keyboard: Keyboard,
) -> Result<(), String> {
    let text: String = text.trim().chars().take(4000).collect();
    if text.is_empty() {
        return Ok(());
    }
    let mut body = json!({
        "chat_id": chat_id,
        "text": text,
        "allow_sending_without_reply": true,
        "reply_markup": markup(keyboard),
    });
    if let Some(message_id) = reply_to {
        body["reply_to_message_id"] = json!(message_id);
    }
    telegram::post(token, "sendMessage", body)?;
    Ok(())
}

fn markup(keyboard: Keyboard) -> Value {
    let rows: Vec<Vec<Value>> = match keyboard {
        Keyboard::NewTask => vec![vec![button("Новая задача", "task:new")]],
        Keyboard::Cancel => vec![vec![button("Отмена", "task:cancel")]],
        Keyboard::Comment => vec![
            vec![button("Без комментария", "task:skip")],
            vec![button("Отмена", "task:cancel")],
        ],
        Keyboard::HelpBusy => vec![
            vec![button("Новая задача", "task:new")],
            vec![button("Отмена", "task:cancel")],
        ],
    };
    json!({ "inline_keyboard": rows })
}

fn button(text: &str, data: &str) -> Value {
    json!({ "text": text, "callback_data": data })
}

fn sessions() -> std::sync::MutexGuard<'static, HashMap<(i64, i64), Pending>> {
    SESSIONS.lock().unwrap_or_else(|err| err.into_inner())
}

fn clock_of(unix: i64) -> NaiveTime {
    DateTime::from_timestamp(unix, 0)
        .map(|utc| hour_minute(utc.with_timezone(&Local).time()))
        .unwrap_or_else(|| hour_minute(Local::now().time()))
}

fn hour_minute(time: NaiveTime) -> NaiveTime {
    NaiveTime::from_hms_opt(time.hour(), time.minute(), 0).unwrap_or(time)
}

fn end_of_day() -> NaiveTime {
    NaiveTime::from_hms_opt(23, 0, 0).expect("23:00")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Keyboard {
    NewTask,
    Cancel,
    Comment,
    HelpBusy,
}

#[derive(Clone)]
enum Event {
    Help,
    New,
    Cancel,
    Skip,
    Text(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Step {
    Title,
    Start,
    Deadline,
    Comment,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Pending {
    step: Step,
    title: String,
    start: Option<NaiveDateTime>,
    deadline: Option<NaiveDateTime>,
}

impl Pending {
    fn title() -> Self {
        Self {
            step: Step::Title,
            title: String::new(),
            start: None,
            deadline: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct TaskDraft {
    title: String,
    description: String,
    start: NaiveDateTime,
    deadline: NaiveDateTime,
}

struct Session {
    pending: Option<Pending>,
}

impl Default for Session {
    fn default() -> Self {
        Self { pending: None }
    }
}

struct Decision {
    text: String,
    keyboard: Keyboard,
    save: Option<TaskDraft>,
}

impl Decision {
    fn say(text: impl Into<String>, keyboard: Keyboard) -> Self {
        Self {
            text: text.into(),
            keyboard,
            save: None,
        }
    }

    fn saved(draft: TaskDraft) -> Self {
        Self {
            text: saved_text(&draft),
            keyboard: Keyboard::NewTask,
            save: Some(draft),
        }
    }
}

fn handle(session: &mut Session, event: Event, clock: NaiveTime) -> Decision {
    match event {
        Event::Help => Decision::say(help_text(), help_keyboard(session)),
        Event::New => {
            session.pending = Some(Pending::title());
            Decision::say(ASK_TITLE, Keyboard::Cancel)
        }
        Event::Cancel => {
            if session.pending.take().is_none() {
                Decision::say(
                    "Сейчас задача не заполняется.".to_string(),
                    Keyboard::NewTask,
                )
            } else {
                Decision::say("Не добавляю эту задачу.".to_string(), Keyboard::NewTask)
            }
        }
        Event::Skip => {
            if session
                .pending
                .as_ref()
                .is_some_and(|pending| pending.step == Step::Comment)
            {
                finish(session, String::new())
            } else {
                accept_text(session, "Без комментария".to_string(), clock)
            }
        }
        Event::Text(text) => accept_text(session, text, clock),
    }
}

fn help_keyboard(session: &Session) -> Keyboard {
    if session.pending.is_some() {
        Keyboard::HelpBusy
    } else {
        Keyboard::NewTask
    }
}

fn keyboard_for(pending: Option<&Pending>) -> Keyboard {
    match pending.map(|pending| pending.step) {
        Some(Step::Comment) => Keyboard::Comment,
        Some(_) => Keyboard::Cancel,
        None => Keyboard::NewTask,
    }
}

fn accept_text(session: &mut Session, text: String, clock: NaiveTime) -> Decision {
    let step = session.pending.as_ref().map(|pending| pending.step);
    if step != Some(Step::Comment) {
        if let Some(decision) = sniff(session, &text, clock) {
            return decision;
        }
    }
    match step {
        None => Decision::say(unrecognized(), Keyboard::NewTask),
        Some(Step::Title) => accept_title(session, &text),
        Some(Step::Start) => accept_start(session, &text, clock),
        Some(Step::Deadline) => accept_deadline(session, &text),
        Some(Step::Comment) => finish(session, text.trim().to_string()),
    }
}

fn sniff(session: &mut Session, text: &str, clock: NaiveTime) -> Option<Decision> {
    if content_lines(text).len() < 3 {
        return None;
    }
    match parse_template(text, clock) {
        Ok(draft) => {
            session.pending = None;
            Some(Decision::saved(draft))
        }
        Err(reason) => Some(Decision::say(
            format!("{reason}\n\n{}", help_text()),
            if session.pending.is_some() {
                Keyboard::HelpBusy
            } else {
                Keyboard::NewTask
            },
        )),
    }
}

fn accept_title(session: &mut Session, text: &str) -> Decision {
    let title = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.is_empty() {
        return Decision::say(ASK_TITLE, Keyboard::Cancel);
    }
    if title.chars().count() > 160 {
        return Decision::say(
            "Название длиннее 160 символов.".to_string(),
            Keyboard::Cancel,
        );
    }
    if let Some(pending) = session.pending.as_mut() {
        pending.title = title;
        pending.step = Step::Start;
    }
    Decision::say(ASK_START, Keyboard::Cancel)
}

fn accept_start(session: &mut Session, text: &str, clock: NaiveTime) -> Decision {
    let (date, time) = match parse_date_line(text) {
        Ok(parsed) => parsed,
        Err(err) => return Decision::say(format!("Дата начала: {err}"), Keyboard::Cancel),
    };
    if let Some(pending) = session.pending.as_mut() {
        pending.start = Some(NaiveDateTime::new(date, time.unwrap_or(clock)));
        pending.step = Step::Deadline;
    }
    Decision::say(ASK_DEADLINE, Keyboard::Cancel)
}

fn accept_deadline(session: &mut Session, text: &str) -> Decision {
    let (date, time) = match parse_date_line(text) {
        Ok(parsed) => parsed,
        Err(err) => return Decision::say(format!("Дедлайн: {err}"), Keyboard::Cancel),
    };
    let deadline = NaiveDateTime::new(date, time.unwrap_or_else(end_of_day));
    if session
        .pending
        .as_ref()
        .and_then(|pending| pending.start)
        .is_some_and(|start| deadline < start)
    {
        return Decision::say(
            "Дедлайн раньше начала. Введите дедлайн ещё раз.".to_string(),
            Keyboard::Cancel,
        );
    }
    if let Some(pending) = session.pending.as_mut() {
        pending.deadline = Some(deadline);
        pending.step = Step::Comment;
    }
    Decision::say(ASK_COMMENT, Keyboard::Comment)
}

fn finish(session: &mut Session, comment: String) -> Decision {
    if comment.chars().count() > 4000 {
        return Decision::say(
            "Комментарий длиннее 4000 символов.".to_string(),
            Keyboard::Comment,
        );
    }
    let Some(pending) = session.pending.clone() else {
        return Decision::say(unrecognized(), Keyboard::NewTask);
    };
    let (Some(start), Some(deadline)) = (pending.start, pending.deadline) else {
        return Decision::say(ASK_DEADLINE, Keyboard::Cancel);
    };
    if deadline < start {
        return Decision::say("Дедлайн раньше начала.".to_string(), Keyboard::Cancel);
    }
    session.pending = None;
    Decision::saved(TaskDraft {
        title: pending.title,
        description: comment,
        start,
        deadline,
    })
}

fn parse_template(text: &str, clock: NaiveTime) -> Result<TaskDraft, String> {
    let lines = content_lines(text);
    if lines.len() < 3 {
        return Err("мало строк".into());
    }
    let title = lines[0].trim();
    if title.is_empty() {
        return Err("Нужно наименование.".into());
    }
    if title.chars().count() > 160 {
        return Err("Название длиннее 160 символов.".into());
    }
    let (start_date, start_time) =
        parse_date_line(lines[1]).map_err(|err| format!("Дата начала: {err}"))?;
    let (deadline_date, deadline_time) =
        parse_date_line(lines[2]).map_err(|err| format!("Дедлайн: {err}"))?;
    let start = NaiveDateTime::new(start_date, start_time.unwrap_or(clock));
    let deadline = NaiveDateTime::new(deadline_date, deadline_time.unwrap_or_else(end_of_day));
    if deadline < start {
        return Err("Дедлайн раньше начала.".into());
    }
    let description = lines[3..].join("\n");
    if description.chars().count() > 4000 {
        return Err("Комментарий длиннее 4000 символов.".into());
    }
    Ok(TaskDraft {
        title: title.to_string(),
        description,
        start,
        deadline,
    })
}

fn content_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn parse_date_line(text: &str) -> Result<(NaiveDate, Option<NaiveTime>), String> {
    let mut tokens: Vec<&str> = text
        .split_whitespace()
        .map(|token| token.trim_matches(|ch: char| ch == ',' || ch == ';'))
        .filter(|token| !token.is_empty())
        .collect();
    if tokens.len() == 3 && tokens[1].to_lowercase() == "в" {
        tokens.remove(1);
    }
    match tokens.as_slice() {
        [date] => Ok((parse_date(date)?, None)),
        [date, time] => Ok((parse_date(date)?, Some(parse_clock(time)?))),
        _ => Err(DATE_HINT.into()),
    }
}

fn parse_date(token: &str) -> Result<NaiveDate, String> {
    let mut parts: Vec<&str> = token.split('.').collect();
    if parts.last().is_some_and(|part| part.is_empty()) {
        parts.pop();
    }
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()))
        || parts[2].len() != 4
    {
        return Err(DATE_HINT.into());
    }
    let day: u32 = parts[0].parse().map_err(|_| DATE_HINT.to_string())?;
    let month: u32 = parts[1].parse().map_err(|_| DATE_HINT.to_string())?;
    let year: i32 = parts[2].parse().map_err(|_| DATE_HINT.to_string())?;
    if !(1900..=2100).contains(&year) {
        return Err("год нужен от 1900 до 2100".into());
    }
    NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| "такой даты нет".to_string())
}

fn parse_clock(token: &str) -> Result<NaiveTime, String> {
    let token = token.trim_end_matches('.');
    let Some((hour, minute)) = token.split_once(':').or_else(|| token.split_once('-')) else {
        return Err("время: чч:мм или чч-мм".into());
    };
    if hour.is_empty()
        || minute.is_empty()
        || hour.len() > 2
        || minute.len() > 2
        || !hour.chars().all(|ch| ch.is_ascii_digit())
        || !minute.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err("время: чч:мм или чч-мм".into());
    }
    let hour: u32 = hour
        .parse()
        .map_err(|_| "время: чч:мм или чч-мм".to_string())?;
    let minute: u32 = minute
        .parse()
        .map_err(|_| "время: чч:мм или чч-мм".to_string())?;
    NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| "время: чч:мм или чч-мм".to_string())
}

fn help_text() -> String {
    format!(
        "Пришлите задачу так:\n\n{SAMPLE}\n\n\
         Дата — дд.мм.гггг. Время можно дописать через пробел: чч:мм или чч-мм.\n\
         Если у начала времени нет, беру время сообщения. Если у дедлайна времени нет, ставлю 23:00.\n\
         Комментарий можно не писать.\n\n\
         Кнопка «Новая задача» спросит название, начало, дедлайн и комментарий по очереди."
    )
}

fn unrecognized() -> String {
    format!("Не разобрал сообщение.\n\n{}", help_text())
}

fn saved_text(draft: &TaskDraft) -> String {
    let mut text = format!(
        "Добавил «{}».\nНачало: {}\nДедлайн: {}",
        draft.title,
        draft.start.format("%d.%m.%Y %H:%M"),
        draft.deadline.format("%d.%m.%Y %H:%M"),
    );
    if !draft.description.is_empty() {
        text.push_str("\nКомментарий: ");
        text.push_str(&draft.description);
    }
    text
}

fn interpret(text: &str) -> Event {
    let compact = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if compact.is_empty() {
        return Event::Text(String::new());
    }
    match command_name(text).as_deref() {
        Some("/start") | Some("/help") => return Event::Help,
        Some("/new") | Some("/task") => return Event::New,
        Some("/cancel") => return Event::Cancel,
        _ => {}
    }
    if compact == "новая задача" {
        return Event::New;
    }
    if compact == "отмена" {
        return Event::Cancel;
    }
    if compact == "без комментария" {
        return Event::Skip;
    }
    Event::Text(text.trim().to_string())
}

fn command_name(text: &str) -> Option<String> {
    let first = text.trim().split_whitespace().next()?;
    if !first.starts_with('/') {
        return None;
    }
    let name = first.split('@').next().unwrap_or(first);
    Some(name.to_ascii_lowercase())
}

fn addressed_elsewhere(text: &str, username: &str) -> bool {
    if username.is_empty() {
        return false;
    }
    let Some(first) = text.trim().split_whitespace().next() else {
        return false;
    };
    let Some((name, target)) = first.split_once('@') else {
        return false;
    };
    name.starts_with('/') && !target.eq_ignore_ascii_case(username)
}

enum Raw {
    Text {
        chat_id: i64,
        user_id: i64,
        message_id: i64,
        unix: i64,
        text: String,
    },
    Callback {
        id: String,
        chat_id: i64,
        user_id: i64,
        data: String,
    },
}

fn read_batch(body: &Value, previous: i64) -> Result<(Vec<Raw>, i64), String> {
    let result = body
        .get("result")
        .and_then(Value::as_array)
        .ok_or("Телеграм не прислал обновления")?;
    let mut next = previous;
    let mut items = Vec::new();
    for update in result {
        if let Some(id) = update.get("update_id").and_then(Value::as_i64) {
            next = next.max(id.saturating_add(1));
        }
        if let Some(item) = raw_update(update) {
            items.push(item);
        }
    }
    Ok((items, next))
}

fn raw_update(update: &Value) -> Option<Raw> {
    if let Some(callback) = update.get("callback_query") {
        return raw_callback(callback);
    }
    let message = update
        .get("message")
        .or_else(|| update.get("channel_post"))?;
    raw_text(message)
}

fn raw_text(message: &Value) -> Option<Raw> {
    let chat_id = message.get("chat")?.get("id")?.as_i64()?;
    if message
        .get("from")
        .and_then(|from| from.get("is_bot"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        return None;
    }
    let user_id = message
        .get("from")
        .and_then(|from| from.get("id"))
        .and_then(Value::as_i64)
        .unwrap_or(chat_id);
    let text = message.get("text")?.as_str()?.to_string();
    if text.trim().is_empty() {
        return None;
    }
    Some(Raw::Text {
        chat_id,
        user_id,
        message_id: message.get("message_id")?.as_i64()?,
        unix: message.get("date").and_then(Value::as_i64).unwrap_or(0),
        text,
    })
}

fn raw_callback(callback: &Value) -> Option<Raw> {
    if callback
        .get("from")
        .and_then(|from| from.get("is_bot"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        return None;
    }
    Some(Raw::Callback {
        id: callback.get("id")?.as_str()?.to_string(),
        chat_id: callback.get("message")?.get("chat")?.get("id")?.as_i64()?,
        user_id: callback.get("from")?.get("id")?.as_i64()?,
        data: callback.get("data")?.as_str()?.to_string(),
    })
}

fn report(result: Result<(), String>) -> Result<(), String> {
    let mut last = LAST_POLL_ERROR
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    match &result {
        Ok(()) => last.clear(),
        Err(err) if last.as_str() != err => {
            eprintln!("бот задач: {err}");
            last.clone_from(err);
        }
        Err(_) => {}
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(hour: u32, minute: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
    }

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, day).unwrap(),
            clock(hour, minute),
        )
    }

    #[test]
    fn template_uses_message_clock_and_deadline_evening() {
        let draft = parse_template(
            "Купить молоко\n\n26.11.2026\n27.11.2026\nдва литра",
            clock(15, 4),
        )
        .unwrap();
        assert_eq!(draft.title, "Купить молоко");
        assert_eq!(draft.start, at(2026, 11, 26, 15, 4));
        assert_eq!(draft.deadline, at(2026, 11, 27, 23, 0));
        assert_eq!(draft.description, "два литра");
    }

    #[test]
    fn template_keeps_written_time_and_joins_the_comment() {
        let draft = parse_template(
            "Стенд\n1.2.2026 9:05\n02.02.2026 18-00\nстрока 1\nстрока 2",
            clock(1, 2),
        )
        .unwrap();
        assert_eq!(draft.start, at(2026, 2, 1, 9, 5));
        assert_eq!(draft.deadline, at(2026, 2, 2, 18, 0));
        assert_eq!(draft.description, "строка 1\nстрока 2");
    }

    #[test]
    fn template_accepts_the_word_v_and_a_comma() {
        let draft =
            parse_template("Замер\n26.11.2026 в 14:30\n26.11.2026, 16-05", clock(8, 0)).unwrap();
        assert_eq!(draft.start, at(2026, 11, 26, 14, 30));
        assert_eq!(draft.deadline, at(2026, 11, 26, 16, 5));
        assert!(draft.description.is_empty());
    }

    #[test]
    fn template_rejects_a_deadline_before_the_start_and_a_bad_date() {
        let early = parse_template("Дело\n27.11.2026 10:00\n26.11.2026", clock(9, 0)).unwrap_err();
        assert!(early.contains("Дедлайн раньше начала"));
        let missing = parse_template("Дело\nзавтра\n26.11.2026", clock(9, 0)).unwrap_err();
        assert!(missing.contains("дд.мм.гггг"));
        let absent = parse_template("Дело\n32.13.2026\n01.01.2026", clock(9, 0)).unwrap_err();
        assert!(absent.contains("такой даты нет"));
    }

    #[test]
    fn a_short_message_draws_the_template() {
        let mut session = Session::default();
        let decision = handle(&mut session, Event::Text("привет".into()), clock(12, 0));
        assert!(decision.save.is_none());
        assert!(decision.text.contains("Не разобрал"));
        assert!(decision.text.contains("Наименование"));
        assert!(session.pending.is_none());
    }

    #[test]
    fn the_button_walks_the_fields_in_order() {
        let mut session = Session::default();
        let ask = handle(&mut session, Event::New, clock(10, 0));
        assert!(ask.text.contains("наименование"));
        assert_eq!(session.pending.as_ref().unwrap().step, Step::Title);

        let ask = handle(&mut session, Event::Text("Молоко".into()), clock(10, 0));
        assert!(ask.text.contains("дату начала"));
        let ask = handle(&mut session, Event::Text("26.11.2026".into()), clock(15, 4));
        assert!(ask.text.contains("дедлайн"));
        assert_eq!(
            session.pending.as_ref().unwrap().start,
            Some(at(2026, 11, 26, 15, 4))
        );

        let ask = handle(
            &mut session,
            Event::Text("26.11.2026".into()),
            clock(16, 40),
        );
        assert!(ask.text.contains("комментарий"));
        assert_eq!(
            session.pending.as_ref().unwrap().deadline,
            Some(at(2026, 11, 26, 23, 0))
        );

        let done = handle(&mut session, Event::Skip, clock(16, 41));
        let draft = done.save.unwrap();
        assert_eq!(draft.title, "Молоко");
        assert!(draft.description.is_empty());
        assert_eq!(draft.start, at(2026, 11, 26, 15, 4));
        assert_eq!(draft.deadline, at(2026, 11, 26, 23, 0));
        assert!(session.pending.is_none());
        assert!(done.text.contains("26.11.2026 15:04"));
        assert!(done.text.contains("26.11.2026 23:00"));
    }

    #[test]
    fn a_bad_date_stays_on_the_same_step_and_cancel_drops_the_draft() {
        let mut session = Session::default();
        handle(&mut session, Event::New, clock(11, 0));
        handle(&mut session, Event::Text("Стенд".into()), clock(11, 0));
        let again = handle(&mut session, Event::Text("завтра".into()), clock(11, 5));
        assert!(again.text.contains("Дата начала"));
        assert_eq!(session.pending.as_ref().unwrap().step, Step::Start);

        handle(
            &mut session,
            Event::Text("26.11.2026 23:30".into()),
            clock(11, 5),
        );
        let early = handle(&mut session, Event::Text("26.11.2026".into()), clock(11, 6));
        assert!(early.text.contains("Дедлайн раньше начала"));
        assert_eq!(session.pending.as_ref().unwrap().step, Step::Deadline);

        let gone = handle(&mut session, Event::Cancel, clock(11, 7));
        assert!(session.pending.is_none());
        assert!(gone.text.contains("Не добавляю"));
    }

    #[test]
    fn a_full_template_in_the_middle_of_the_wizard_is_the_task() {
        let mut session = Session::default();
        handle(&mut session, Event::New, clock(8, 0));
        let done = handle(
            &mut session,
            Event::Text("Отчёт\n26.11.2026 09:00\n27.11.2026\nчерновик".into()),
            clock(8, 30),
        );
        let draft = done.save.unwrap();
        assert_eq!(draft.title, "Отчёт");
        assert_eq!(draft.start, at(2026, 11, 26, 9, 0));
        assert_eq!(draft.deadline, at(2026, 11, 27, 23, 0));
        assert_eq!(draft.description, "черновик");
        assert!(session.pending.is_none());
    }

    #[test]
    fn commands_and_other_bots_are_told_apart() {
        assert!(addressed_elsewhere("/new@OtherBot", "TaskBot"));
        assert!(!addressed_elsewhere("/new@TaskBot", "TaskBot"));
        assert!(!addressed_elsewhere("/new", "TaskBot"));
        assert!(matches!(interpret("/start@TaskBot"), Event::Help));
        assert!(matches!(interpret("  Новая   задача "), Event::New));
        assert!(matches!(interpret("Без комментария"), Event::Skip));
    }

    #[test]
    fn any_chat_can_send_a_task_and_a_button_press_is_kept() {
        let body = json!({
            "ok": true,
            "result": [
                {
                    "update_id": 10,
                    "message": {
                        "message_id": 1,
                        "from": {"id": 7, "is_bot": false, "first_name": "Анна"},
                        "chat": {"id": -100, "type": "supergroup", "title": "цех"},
                        "date": 1760000000,
                        "text": "Стенд\n26.11.2026\n27.11.2026"
                    }
                },
                {
                    "update_id": 11,
                    "message": {
                        "message_id": 2,
                        "from": {"id": 1, "is_bot": true, "first_name": "бот"},
                        "chat": {"id": 7, "type": "private"},
                        "date": 1760000001,
                        "text": "эхо"
                    }
                },
                {
                    "update_id": 12,
                    "message": {
                        "message_id": 3,
                        "from": {"id": 7, "is_bot": false, "first_name": "Анна"},
                        "chat": {"id": 7, "type": "private"},
                        "date": 1760000002
                    }
                },
                {
                    "update_id": 13,
                    "callback_query": {
                        "id": "cb",
                        "from": {"id": 7, "is_bot": false, "first_name": "Анна"},
                        "data": "task:new",
                        "message": {"message_id": 4, "chat": {"id": -100, "type": "supergroup"}}
                    }
                },
                {
                    "update_id": 14,
                    "channel_post": {
                        "message_id": 5,
                        "chat": {"id": -200, "type": "channel", "title": "лента"},
                        "date": 1760000003,
                        "text": "Канал\n01.12.2026 08:00\n01.12.2026 18:00"
                    }
                }
            ]
        });
        let (items, next) = read_batch(&body, 0).unwrap();
        assert_eq!(next, 15);
        assert_eq!(items.len(), 3);
        match &items[0] {
            Raw::Text {
                chat_id,
                user_id,
                text,
                ..
            } => {
                assert_eq!(*chat_id, -100);
                assert_eq!(*user_id, 7);
                assert!(text.starts_with("Стенд"));
            }
            Raw::Callback { .. } => panic!("сначала текст из группы"),
        }
        match &items[1] {
            Raw::Callback { chat_id, data, .. } => {
                assert_eq!(*chat_id, -100);
                assert_eq!(data, "task:new");
            }
            Raw::Text { .. } => panic!("дальше кнопка"),
        }
        match &items[2] {
            Raw::Text {
                chat_id, user_id, ..
            } => {
                assert_eq!(*chat_id, -200);
                assert_eq!(*user_id, -200);
            }
            Raw::Callback { .. } => panic!("дальше пост канала"),
        }
    }
}
