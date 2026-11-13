use chrono::{DateTime, Local, Utc};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
#[cfg(not(windows))]
use tauri_plugin_notification::{Action, ActionType, NotificationExt};

use crate::model::NagKind;
use crate::store::Store;

pub fn tick(app: &AppHandle) -> Result<(), String> {
    let imported = crate::agent::import_inbox(app)?;
    let now = Utc::now();
    let due = app.state::<Store>().due(now);
    if !due.is_empty() {
        let _ = open_nag_window(app);
    }
    sync_nag_window(app);
    let mut sent = false;
    for (card, kind) in due {
        match show_nag(app, &card, kind) {
            Ok(()) => {
                app.state::<Store>().acknowledge(&card.id, kind, now)?;
                sent = true;
            }
            Err(err) => eprintln!("уведомление «{}»: {err}", card.title),
        }
    }
    if imported || sent {
        let _ = app.emit("cards-changed", app.state::<Store>().list());
    }
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
            let cards = store.set_status(&id, crate::model::DONE_ID)?;
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
    sync_nag_window(app);
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
    let (title, body) = nag_copy(card, kind, Utc::now());
    let buttons: Vec<(&str, String)> = match kind {
        NagKind::Start => vec![("Готово", format!("done:{}", card.id))],
        NagKind::Deadline => vec![
            ("Готово", format!("done:{}", card.id)),
            ("15 мин", format!("snooze:15:{}", card.id)),
            ("1 час", format!("snooze:60:{}", card.id)),
            ("3 часа", format!("snooze:180:{}", card.id)),
            ("Сутки", format!("snooze:1440:{}", card.id)),
        ],
    };
    #[cfg(windows)]
    {
        show_windows_toast(app, &title, &body, &buttons)
    }
    #[cfg(not(windows))]
    {
        show_plugin_toast(app, &title, &body, &buttons)
    }
}

#[cfg(windows)]
fn show_windows_toast(
    app: &AppHandle,
    title: &str,
    body: &str,
    buttons: &[(&str, String)],
) -> Result<(), String> {
    use tauri_winrt_notification::Toast;
    let _ = ensure_windows_toast_identity();
    match send_windows_toast(app, "com.inzh.workcalendar", title, body, buttons) {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("свой тост не открылся ({err}), запасной канал");
            send_windows_toast(app, Toast::POWERSHELL_APP_ID, title, body, buttons)
        }
    }
}

#[cfg(windows)]
fn send_windows_toast(
    app: &AppHandle,
    app_id: &str,
    title: &str,
    body: &str,
    buttons: &[(&str, String)],
) -> Result<(), String> {
    use tauri_winrt_notification::{Scenario, Toast};
    let handle = app.clone();
    let mut toast = Toast::new(app_id)
        .title(title)
        .text1(body)
        .scenario(Scenario::Reminder);
    for (label, action) in buttons {
        toast = toast.add_button(label, action);
    }
    toast
        .on_activated(move |action| {
            let action = action.unwrap_or_else(|| "tap".into());
            if let Err(err) = handle_action(&handle, &action) {
                eprintln!("действие уведомления: {err}");
            }
            Ok(())
        })
        .show()
        .map_err(|err| err.to_string())
}

#[cfg(windows)]
fn ensure_windows_toast_identity() -> Result<(), String> {
    let key = windows_registry::CURRENT_USER
        .create(r"SOFTWARE\Classes\AppUserModelId\com.inzh.workcalendar")
        .map_err(|err| err.to_string())?;
    key.set_string("DisplayName", "inzh's work calendar")
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
fn show_plugin_toast(
    app: &AppHandle,
    title: &str,
    body: &str,
    buttons: &[(&str, String)],
) -> Result<(), String> {
    let actions = buttons
        .iter()
        .map(|(label, action)| Action::builder(action, *label).build())
        .collect();
    app.notification()
        .register_action_types(vec![ActionType::builder("nag").actions(actions).build()])
        .map_err(|err| err.to_string())?;
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .action_type_id("nag")
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

pub fn sync_nag_window(app: &AppHandle) {
    let pending = app.state::<Store>().attention(Utc::now());
    if pending.is_empty() {
        hide_nag_window(app);
    }
}

pub fn hide_nag_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("nag") {
        let _ = window.hide();
    }
}

pub fn open_nag_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("nag") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = app.emit_to("nag", "nag-refresh", ());
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(app, "nag", WebviewUrl::App("index.html".into()))
        .title("inzh's work calendar")
        .inner_size(NAG_WIDTH, NAG_HEIGHT)
        .min_inner_size(NAG_WIDTH, 160.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .focused(false)
        .visible(false)
        .shadow(true)
        .build()
        .map_err(|err| err.to_string())?;
    place_nag_window(&window, app);
    let _ = window.show();
    Ok(())
}

const NAG_WIDTH: f64 = 400.0;
const NAG_HEIGHT: f64 = 280.0;

fn place_nag_window(window: &tauri::WebviewWindow, app: &AppHandle) {
    let Ok(Some(monitor)) = app.primary_monitor() else {
        return;
    };
    let scale = monitor.scale_factor().max(1.0);
    let area = monitor.work_area();
    let width = NAG_WIDTH * scale;
    let height = NAG_HEIGHT * scale;
    let margin = 16.0 * scale;
    let x = area.position.x as f64 + area.size.width as f64 - width - margin;
    let y = area.position.y as f64 + area.size.height as f64 - height - margin;
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

pub fn reveal(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
