use chrono::{DateTime, Local, Utc};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::{Action, ActionType, NotificationExt};

use crate::model::NagKind;
use crate::store::Store;

pub fn tick(app: &AppHandle) -> Result<(), String> {
    let imported = crate::agent::import_inbox(app)?;
    let due = app.state::<Store>().collect_due(Utc::now())?;
    if due.is_empty() {
        if imported {
            let _ = app.emit("cards-changed", app.state::<Store>().list());
        }
        return Ok(());
    }
    for (card, kind) in &due {
        if let Err(err) = show_nag(app, card, *kind) {
            eprintln!("уведомление «{}»: {err}", card.title);
        }
    }
    let _ = app.emit("cards-changed", app.state::<Store>().list());
    Ok(())
}

pub fn handle_action(app: &AppHandle, action_id: &str) -> Result<(), String> {
    let store = app.state::<Store>();
    let cards = match interpret(action_id) {
        ToastAction::Open => {
            reveal(app);
            return Ok(());
        }
        ToastAction::Done(id) => {
            let cards = store.set_status(&id, crate::model::Status::Done)?;
            reveal(app);
            let _ = app.emit("focus-card", id);
            cards
        }
        ToastAction::Snooze { id, minutes } => {
            let cards = store.snooze(&id, minutes)?;
            reveal(app);
            let _ = app.emit("focus-card", id);
            cards
        }
    };
    let _ = app.emit("cards-changed", cards);
    Ok(())
}

enum ToastAction {
    Open,
    Done(String),
    Snooze { id: String, minutes: u32 },
}

fn interpret(action_id: &str) -> ToastAction {
    if let Some(id) = action_id.strip_prefix("done:") {
        if !id.is_empty() {
            return ToastAction::Done(id.to_string());
        }
    }
    if let Some(rest) = action_id.strip_prefix("snooze:") {
        if let Some((minutes, id)) = rest.split_once(':') {
            if let Ok(minutes) = minutes.parse::<u32>() {
                if !id.is_empty() {
                    return ToastAction::Snooze {
                        id: id.to_string(),
                        minutes,
                    };
                }
            }
        }
    }
    ToastAction::Open
}

fn show_nag(app: &AppHandle, card: &crate::model::Card, kind: NagKind) -> Result<(), String> {
    let (type_id, actions) = match kind {
        NagKind::Start => (
            "start-nag",
            vec![Action::builder(format!("done:{}", card.id), "Готово").build()],
        ),
        NagKind::Deadline => (
            "deadline-nag",
            vec![
                Action::builder(format!("done:{}", card.id), "Готово").build(),
                Action::builder(format!("snooze:15:{}", card.id), "15 мин").build(),
                Action::builder(format!("snooze:60:{}", card.id), "1 час").build(),
                Action::builder(format!("snooze:1440:{}", card.id), "Сутки").build(),
            ],
        ),
    };
    app.notification()
        .register_action_types(vec![ActionType::builder(type_id).actions(actions).build()])
        .map_err(|err| err.to_string())?;

    let (title, body) = nag_copy(card, kind, Utc::now());
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .extra("cardId", &card.id)
        .action_type_id(type_id)
        .show()
        .map_err(|err| err.to_string())
}

fn nag_copy(card: &crate::model::Card, kind: NagKind, now: DateTime<Utc>) -> (String, String) {
    match kind {
        NagKind::Start => (
            "Пора начать".into(),
            format!("«{}» · с {}", card.title, stamp(card.start_at.unwrap_or(now))),
        ),
        NagKind::Deadline => (
            "Дедлайн".into(),
            format!(
                "«{}» · просрочено на {}",
                card.title,
                late(card.deadline_at.unwrap_or(now), now)
            ),
        ),
    }
}

fn stamp(time: DateTime<Utc>) -> String {
    time.with_timezone(&Local)
        .format("%d.%m %H:%M")
        .to_string()
}

fn late(from: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let minutes = now.signed_duration_since(from).num_minutes().max(0);
    if minutes < 1 {
        "меньше минуты".into()
    } else if minutes < 60 {
        format!("{minutes} мин")
    } else if minutes < 60 * 48 {
        format!("{} ч", minutes / 60)
    } else {
        format!("{} д", minutes / (60 * 24))
    }
}

pub fn reveal(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
