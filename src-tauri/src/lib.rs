mod agent;
mod model;
mod schedule;
mod store;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

use crate::model::{CardInput, Settings, Status};
use crate::schedule::reveal;
use crate::store::Store;

static ALLOW_EXIT: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn list_cards(store: tauri::State<Store>) -> Vec<model::Card> {
    store.list()
}

#[tauri::command]
fn save_card(store: tauri::State<Store>, input: CardInput) -> Result<Vec<model::Card>, String> {
    store.save_card(input)
}

#[tauri::command]
fn set_status(
    store: tauri::State<Store>,
    id: String,
    status: Status,
) -> Result<Vec<model::Card>, String> {
    store.set_status(&id, status)
}

#[tauri::command]
fn delete_card(store: tauri::State<Store>, id: String) -> Result<Vec<model::Card>, String> {
    store.delete(&id)
}

#[tauri::command]
fn snooze_card(
    store: tauri::State<Store>,
    id: String,
    minutes: u32,
) -> Result<Vec<model::Card>, String> {
    store.snooze(&id, minutes)
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let enabled = app.autolaunch().is_enabled().map_err(|err| err.to_string())?;
    app.state::<Store>().set_autostart_flag(enabled)
}

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<Settings, String> {
    apply_autostart(&app, enabled)?;
    app.state::<Store>().set_autostart_flag(enabled)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            reveal(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if ALLOW_EXIT.load(Ordering::Relaxed) {
                    return;
                }
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let store = Store::load(app.handle())?;
            let first_launch = !store.settings_file_existed();
            let want_autostart = if first_launch {
                true
            } else {
                store.settings().autostart
            };
            app.manage(store);
            if let Err(err) = apply_autostart(app.handle(), want_autostart) {
                eprintln!("автозапуск: {err}");
                if first_launch {
                    let _ = app.state::<Store>().set_autostart_flag(false);
                }
            } else if first_launch {
                let _ = app.state::<Store>().set_autostart_flag(true);
            }
            setup_tray(app.handle())?;
            agent::serve(app.handle()).map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;
            let handle = app.handle().clone();
            app.notification()
                .on_action(move |action| {
                    if let Err(err) = schedule::handle_action(&handle, action.action_id()) {
                        eprintln!("действие уведомления: {err}");
                    }
                })
                .map_err(|err| -> Box<dyn std::error::Error> { err.to_string().into() })?;
            let scheduler = app.handle().clone();
            std::thread::spawn(move || loop {
                if let Err(err) = schedule::tick(&scheduler) {
                    eprintln!("планировщик: {err}");
                }
                std::thread::sleep(Duration::from_secs(20));
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_cards,
            save_card,
            set_status,
            delete_card,
            snooze_card,
            get_settings,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("inzh work calendar не запустился");
}

fn apply_autostart(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|err| err.to_string())
    } else {
        manager.disable().map_err(|err| err.to_string())
    }
}

fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Открыть", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("inzh work calendar")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => reveal(app),
            "quit" => {
                ALLOW_EXIT.store(true, Ordering::Relaxed);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                reveal(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    let tray = tray.build(app)?;
    app.manage(tray);
    Ok(())
}
