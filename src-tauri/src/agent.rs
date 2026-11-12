use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use uuid::Uuid;

use crate::model::{Card, CardInput, Status};
use crate::store::Store;

const PORTS: std::ops::Range<u16> = 47321..47331;
static INBOX: Mutex<()> = Mutex::new(());

pub fn serve(app: &AppHandle) -> Result<(), String> {
    let Some((server, port)) = bind() else {
        eprintln!("локальный вход задач не открылся: порты 47321–47330 заняты");
        return Ok(());
    };
    let dir = app.state::<Store>().directory().to_path_buf();
    let token = load_token(&dir);
    let endpoint = json!({
        "token": token,
        "port": port,
        "base": format!("http://127.0.0.1:{port}"),
    });
    fs::write(
        dir.join("endpoint.json"),
        serde_json::to_vec_pretty(&endpoint).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;

    let handle = app.clone();
    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let _ = import_inbox(&handle);
            if let Err(err) = dispatch(&handle, &token, request) {
                eprintln!("вход задач: {err}");
            }
        }
    });
    Ok(())
}

pub fn import_inbox(app: &AppHandle) -> Result<bool, String> {
    let _guard = INBOX.lock().unwrap_or_else(|err| err.into_inner());
    let dir = app.state::<Store>().directory().to_path_buf();
    let live = dir.join("inbox.jsonl");
    let processing = dir.join("inbox.processing.jsonl");
    let mut imported = false;
    if processing.exists() {
        imported |= consume(app, &processing)?;
    }
    if live.exists() {
        fs::rename(&live, &processing).map_err(|err| err.to_string())?;
        imported |= consume(app, &processing)?;
    }
    Ok(imported)
}

fn bind() -> Option<(Server, u16)> {
    for port in PORTS {
        if let Ok(server) = Server::http(format!("127.0.0.1:{port}")) {
            return Some((server, port));
        }
    }
    None
}

fn load_token(dir: &Path) -> String {
    let path = dir.join("endpoint.json");
    if let Ok(bytes) = fs::read(path) {
        if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
            if let Some(token) = value.get("token").and_then(Value::as_str) {
                if !token.is_empty() {
                    return token.to_string();
                }
            }
        }
    }
    Uuid::new_v4().to_string()
}

fn dispatch(app: &AppHandle, token: &str, mut request: Request) -> Result<(), String> {
    if !authorized(&request, token) {
        return respond(request, 401, json!({ "error": "нужен токен из endpoint.json" }));
    }

    let path = request.url().split('?').next().unwrap_or("").to_string();
    let method = request.method().clone();
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|err| err.to_string())?;

    let result = route(app, &method, &path, &body);
    let (status, payload) = match result {
        Ok(value) => (200, value),
        Err(err) => (400, json!({ "error": err })),
    };
    respond(request, status, payload)
}

fn route(app: &AppHandle, method: &Method, path: &str, body: &str) -> Result<Value, String> {
    let store = app.state::<Store>();
    if path == "/health" && *method == Method::Get {
        return Ok(json!({ "ok": true }));
    }
    if path == "/tasks" && *method == Method::Get {
        return Ok(json!({ "tasks": store.list() }));
    }
    if path == "/tasks" && *method == Method::Post {
        let input = incoming_to_input(body, None)?;
        let before = store.list();
        let after = store.save_card(input)?;
        let card = after
            .iter()
            .find(|card| before.iter().all(|old| old.id != card.id))
            .cloned()
            .ok_or("задача не сохранилась")?;
        drop(store);
        publish(app);
        return Ok(json!({ "task": card }));
    }

    if let Some(id) = path.strip_prefix("/tasks/") {
        if id.is_empty() || id.contains('/') {
            return Err("непонятный адрес".into());
        }
        if *method == Method::Patch {
            let card = find_card(&store.list(), id)?;
            let input = patch_input(&card, body)?;
            store.save_card(input)?;
            let saved = find_card(&store.list(), id)?;
            drop(store);
            publish(app);
            return Ok(json!({ "task": saved }));
        }
        if *method == Method::Delete {
            store.delete(id)?;
            drop(store);
            publish(app);
            return Ok(json!({ "ok": true }));
        }
    }

    Err("нет такого запроса".into())
}

fn publish(app: &AppHandle) {
    let _ = app.emit("cards-changed", app.state::<Store>().list());
}

fn find_card(cards: &[Card], id: &str) -> Result<Card, String> {
    cards
        .iter()
        .find(|card| card.id == id)
        .cloned()
        .ok_or("Задача не найдена".into())
}

fn authorized(request: &Request, token: &str) -> bool {
    request.headers().iter().any(|header| {
        header.field.equiv("Authorization")
            && header
                .value
                .as_str()
                .strip_prefix("Bearer ")
                .is_some_and(|value| value.trim() == token)
    })
}

fn respond(request: Request, status: u16, payload: Value) -> Result<(), String> {
    let body = payload.to_string();
    let header = Header::from_bytes(
        &b"Content-Type"[..],
        &b"application/json; charset=utf-8"[..],
    )
    .map_err(|_| "заголовок ответа".to_string())?;
    request
        .respond(
            Response::from_string(body)
                .with_status_code(StatusCode(status))
                .with_header(header),
        )
        .map_err(|err| err.to_string())
}

fn consume(app: &AppHandle, path: &Path) -> Result<bool, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut failed = Vec::new();
    let mut count = 0;
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match incoming_to_input(line, None).and_then(|input| {
            app.state::<Store>().save_card(input)?;
            Ok(())
        }) {
            Ok(()) => count += 1,
            Err(err) => {
                eprintln!("inbox, строка {}: {err}", index + 1);
                failed.push(line.to_string());
            }
        }
    }
    fs::remove_file(path).map_err(|err| err.to_string())?;
    if !failed.is_empty() {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.with_file_name("inbox.failed.jsonl"))
            .map_err(|err| err.to_string())?;
        for line in failed {
            writeln!(file, "{line}").map_err(|err| err.to_string())?;
        }
    }
    if count > 0 {
        publish(app);
    }
    Ok(count > 0)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Incoming {
    title: String,
    #[serde(default)]
    start_at: Option<String>,
    #[serde(default)]
    deadline_at: Option<String>,
    #[serde(default)]
    status: Option<Status>,
}

fn incoming_to_input(body: &str, id: Option<String>) -> Result<CardInput, String> {
    let incoming: Incoming = serde_json::from_str(body).map_err(|err| err.to_string())?;
    Ok(CardInput {
        id,
        title: incoming.title,
        status: incoming.status.unwrap_or(Status::Todo),
        start_at: optional_when(incoming.start_at, "startAt")?,
        deadline_at: optional_when(incoming.deadline_at, "deadlineAt")?,
    })
}

fn patch_input(card: &Card, body: &str) -> Result<CardInput, String> {
    let value: Value = serde_json::from_str(body).map_err(|err| err.to_string())?;
    let title = match value.get("title") {
        None => card.title.clone(),
        Some(Value::String(title)) => title.clone(),
        Some(_) => return Err("title должен быть строкой".into()),
    };
    let status = match value.get("status") {
        None => card.status,
        Some(status) => serde_json::from_value(status.clone()).map_err(|_| "неизвестный статус".to_string())?,
    };
    Ok(CardInput {
        id: Some(card.id.clone()),
        title,
        status,
        start_at: take_time(&value, "startAt", card.start_at)?,
        deadline_at: take_time(&value, "deadlineAt", card.deadline_at)?,
    })
}

fn take_time(value: &Value, key: &str, current: Option<DateTime<Utc>>) -> Result<Option<DateTime<Utc>>, String> {
    match value.get(key) {
        None => Ok(current),
        Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => optional_when(Some(text.clone()), key),
        Some(_) => Err(format!("{key}: нужна строка или null")),
    }
}

fn optional_when(value: Option<String>, field: &str) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    parse_when(value, field).map(Some)
}

fn parse_when(value: &str, field: &str) -> Result<DateTime<Utc>, String> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Ok(parsed.with_timezone(&Utc));
    }
    for format in ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            let local = Local
                .from_local_datetime(&naive)
                .single()
                .ok_or_else(|| format!("{field}: неоднозначное местное время"))?;
            return Ok(local.with_timezone(&Utc));
        }
    }
    Err(format!(
        "{field}: нужна дата и время, например 2026-11-13T21:00"
    ))
}
