use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::model::{
    checked_recurrence, mark_notified, nag_kind, needs_attention, normalize_columns, Card, CardInput,
    Column,
    NagKind, Settings, DONE_ID, MAX_SNOOZE_MINUTES, TODO_ID,
};

pub struct Store {
    dir: PathBuf,
    cards_path: PathBuf,
    settings_path: PathBuf,
    cards: Mutex<Vec<Card>>,
    settings: Mutex<Settings>,
    settings_existed: bool,
}

impl Store {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
        fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        let cards_path = dir.join("cards.json");
        let settings_path = dir.join("settings.json");
        let settings_existed = settings_path.exists();
        let mut settings: Settings = read_json(&settings_path)?;
        let columns = normalize_columns(settings.columns);
        settings.columns = columns;
        let store = Self {
            dir,
            cards: Mutex::new(read_json(&cards_path)?),
            settings: Mutex::new(settings),
            cards_path,
            settings_path,
            settings_existed,
        };
        write_json(&store.settings_path, &*lock(&store.settings))?;
        Ok(store)
    }

    pub fn directory(&self) -> &Path {
        &self.dir
    }

    pub fn settings_file_existed(&self) -> bool {
        self.settings_existed
    }

    pub fn list(&self) -> Vec<Card> {
        lock(&self.cards).clone()
    }

    pub fn settings(&self) -> Settings {
        lock(&self.settings).clone()
    }

    pub fn columns(&self) -> Vec<Column> {
        lock(&self.settings).columns.clone()
    }

    pub fn add_column(&self, title: &str) -> Result<Vec<Column>, String> {
        let title = title.trim();
        if title.is_empty() {
            return Err("Нужно название статуса".into());
        }
        if title.chars().count() > 40 {
            return Err("Название статуса длиннее 40 символов".into());
        }
        let mut settings = lock(&self.settings);
        if settings
            .columns
            .iter()
            .any(|column| column.title.to_lowercase() == title.to_lowercase())
        {
            return Err("Такой статус уже есть".into());
        }
        let done_at = settings
            .columns
            .iter()
            .position(|column| column.id == DONE_ID)
            .unwrap_or(settings.columns.len());
        settings.columns.insert(
            done_at,
            Column {
                id: Uuid::new_v4().to_string(),
                title: title.to_string(),
            },
        );
        write_json(&self.settings_path, &*settings)?;
        Ok(settings.columns.clone())
    }

    pub fn remove_column(&self, id: &str) -> Result<crate::model::ColumnChange, String> {
        if id == TODO_ID || id == DONE_ID {
            return Err("«К выполнению» и «Готово» нельзя убрать".into());
        }
        if !self.columns().iter().any(|column| column.id == id) {
            return Err("Статус не найден".into());
        }
        let cards = self.update_cards(|cards| {
            for card in cards.iter_mut() {
                if card.status == id {
                    card.status = TODO_ID.to_string();
                    card.completed_at = None;
                }
            }
            Ok(())
        })?;
        let mut settings = lock(&self.settings);
        settings.columns.retain(|column| column.id != id);
        write_json(&self.settings_path, &*settings)?;
        Ok(crate::model::ColumnChange {
            columns: settings.columns.clone(),
            cards,
        })
    }

    pub fn set_theme(&self, theme: &str) -> Result<Settings, String> {
        if theme != "dark" && theme != "light" {
            return Err("Тема может быть светлой или тёмной".into());
        }
        let mut settings = lock(&self.settings);
        settings.theme = theme.to_string();
        write_json(&self.settings_path, &*settings)?;
        Ok(settings.clone())
    }

    pub fn set_autostart_flag(&self, enabled: bool) -> Result<Settings, String> {
        let mut settings = lock(&self.settings);
        settings.autostart = enabled;
        write_json(&self.settings_path, &*settings)?;
        Ok(settings.clone())
    }

    pub fn save_card(&self, input: CardInput) -> Result<Vec<Card>, String> {
        let now = Utc::now();
        let title = input.title.trim();
        if title.is_empty() {
            return Err("Нужно название".into());
        }
        if title.chars().count() > 160 {
            return Err("Название длиннее 160 символов".into());
        }
        let description = input.description.trim();
        if description.chars().count() > 4000 {
            return Err("Описание длиннее 4000 символов".into());
        }
        if !self.columns().iter().any(|column| column.id == input.status) {
            return Err("Неизвестный статус".into());
        }
        let recurrence = match &input.recurrence {
            Some(rule) => Some(checked_recurrence(rule)?),
            None => None,
        };
        if let (Some(start), Some(deadline)) = (input.start_at, input.deadline_at) {
            if deadline < start {
                return Err("Дедлайн раньше начала".into());
            }
        }

        self.update_cards(|cards| {
            if let Some(id) = input.id.as_ref() {
                let Some(card) = cards.iter_mut().find(|card| card.id == *id) else {
                    return Err("Задача не найдена".into());
                };
                if card.start_at != input.start_at {
                    card.last_start_notified_at = None;
                }
                if card.deadline_at != input.deadline_at {
                    card.last_deadline_notified_at = None;
                    card.snooze_until = None;
                }
                card.title = title.to_string();
                card.description = description.to_string();
                card.status = input.status.clone();
                card.start_at = input.start_at;
                card.deadline_at = input.deadline_at;
                card.recurrence = recurrence.clone();
                apply_completion(card, now);
            } else {
                let mut card = Card {
                    id: Uuid::new_v4().to_string(),
                    title: title.to_string(),
                    description: description.to_string(),
                    status: input.status.clone(),
                    start_at: input.start_at,
                    deadline_at: input.deadline_at,
                    snooze_until: None,
                    last_start_notified_at: None,
                    last_deadline_notified_at: None,
                    created_at: now,
                    completed_at: None,
                    recurrence: recurrence.clone(),
                };
                apply_completion(&mut card, now);
                cards.push(card);
            }
            Ok(())
        })
    }

    pub fn set_status(&self, id: &str, status: &str) -> Result<Vec<Card>, String> {
        if !self.columns().iter().any(|column| column.id == status) {
            return Err("Неизвестный статус".into());
        }
        let now = Utc::now();
        let status = status.to_string();
        self.update_cards(|cards| {
            let Some(card) = cards.iter_mut().find(|card| card.id == id) else {
                return Err("Задача не найдена".into());
            };
            card.status = status;
            apply_completion(card, now);
            Ok(())
        })
    }

    pub fn delete(&self, id: &str) -> Result<Vec<Card>, String> {
        self.update_cards(|cards| {
            let before = cards.len();
            cards.retain(|card| card.id != id);
            if cards.len() == before {
                return Err("Задача не найдена".into());
            }
            Ok(())
        })
    }

    pub fn snooze(&self, id: &str, minutes: u32) -> Result<Vec<Card>, String> {
        if minutes == 0 || minutes > MAX_SNOOZE_MINUTES {
            return Err("Отложить можно от 1 минуты до 24 часов".into());
        }
        let now = Utc::now();
        self.update_cards(|cards| {
            let Some(card) = cards.iter_mut().find(|card| card.id == id) else {
                return Err("Задача не найдена".into());
            };
            if card.status == DONE_ID {
                return Err("Завершённую задачу не нужно откладывать".into());
            }
            let Some(deadline) = card.deadline_at else {
                return Err("Отложить можно только задачу с дедлайном".into());
            };
            if now < deadline {
                return Err("Отложить можно, когда дедлайн уже наступил".into());
            }
            card.snooze_until = Some(now + Duration::minutes(i64::from(minutes)));
            card.last_deadline_notified_at = None;
            card.last_start_notified_at = None;
            Ok(())
        })
    }

    pub fn due(&self, now: DateTime<Utc>) -> Vec<(Card, NagKind)> {
        lock(&self.cards)
            .iter()
            .filter_map(|card| nag_kind(card, now).map(|kind| (card.clone(), kind)))
            .collect()
    }

    pub fn attention(&self, now: DateTime<Utc>) -> Vec<Card> {
        lock(&self.cards)
            .iter()
            .filter(|card| needs_attention(card, now))
            .cloned()
            .collect()
    }

    pub fn acknowledge(&self, id: &str, kind: NagKind, now: DateTime<Utc>) -> Result<(), String> {
        self.update_cards(|cards| {
            let Some(card) = cards.iter_mut().find(|card| card.id == id) else {
                return Ok(());
            };
            mark_notified(card, kind, now);
            Ok(())
        })?;
        Ok(())
    }

    fn update_cards(&self, change: impl FnOnce(&mut Vec<Card>) -> Result<(), String>) -> Result<Vec<Card>, String> {
        let mut guard = lock(&self.cards);
        let mut next = guard.clone();
        change(&mut next)?;
        write_json(&self.cards_path, &next)?;
        *guard = next.clone();
        Ok(next)
    }
}

fn apply_completion(card: &mut Card, now: DateTime<Utc>) {
    if card.status == DONE_ID {
        if card.completed_at.is_none() {
            card.completed_at = Some(now);
        }
        card.snooze_until = None;
    } else {
        card.completed_at = None;
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

fn read_json<T: serde::de::DeserializeOwned + Default>(path: &Path) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    match serde_json::from_slice(&bytes) {
        Ok(value) => Ok(value),
        Err(err) => {
            let backup = path.with_extension("json.bak");
            fs::rename(path, &backup).map_err(|rename| {
                format!("{err}; файл {} оставлен на месте: {rename}", path.display())
            })?;
            eprintln!(
                "{} повреждён и переложен в {}",
                path.display(),
                backup.display()
            );
            Ok(T::default())
        }
    }
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| err.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes).map_err(|err| err.to_string())?;
    fs::rename(&tmp, path).map_err(|err| err.to_string())?;
    Ok(())
}
