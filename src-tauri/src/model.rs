use chrono::{DateTime, Datelike, Days, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub const NAG_GAP: Duration = Duration::minutes(10);
pub const MAX_SNOOZE_MINUTES: u32 = 24 * 60;
pub const TODO_ID: &str = "todo";
pub const DOING_ID: &str = "doing";
pub const DONE_ID: &str = "done";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub id: String,
    pub title: String,
}

pub fn default_theme() -> String {
    "dark".into()
}

fn default_columns() -> Vec<Column> {
    vec![
        Column {
            id: TODO_ID.into(),
            title: "К выполнению".into(),
        },
        Column {
            id: DOING_ID.into(),
            title: "В работе".into(),
        },
        Column {
            id: DONE_ID.into(),
            title: "Готово".into(),
        },
    ]
}

pub fn normalize_columns(columns: Vec<Column>) -> Vec<Column> {
    let mut before_doing = Vec::new();
    let mut after_doing = Vec::new();
    let mut seen_doing = false;
    for column in columns {
        if column.id == TODO_ID || column.id == DONE_ID || column.title.trim().is_empty() {
            continue;
        }
        if column.id == DOING_ID {
            seen_doing = true;
            continue;
        }
        let bucket = if seen_doing {
            &mut after_doing
        } else {
            &mut before_doing
        };
        if bucket
            .iter()
            .any(|existing: &Column| existing.id == column.id)
        {
            continue;
        }
        bucket.push(Column {
            id: column.id,
            title: column.title.trim().to_string(),
        });
    }
    if !seen_doing {
        after_doing.splice(0..0, before_doing.drain(..));
    }
    let mut columns = vec![Column {
        id: TODO_ID.into(),
        title: "К выполнению".into(),
    }];
    columns.extend(before_doing);
    columns.push(Column {
        id: DOING_ID.into(),
        title: "В работе".into(),
    });
    columns.extend(after_doing);
    columns.push(Column {
        id: DONE_ID.into(),
        title: "Готово".into(),
    });
    columns
}

/// Start reminders belong to every status strictly above «В работе».
pub fn awaits_start(columns: &[Column], status: &str) -> bool {
    let Some(doing_at) = columns.iter().position(|column| column.id == DOING_ID) else {
        return status == TODO_ID;
    };
    columns
        .iter()
        .position(|column| column.id == status)
        .is_some_and(|at| at < doing_at)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NagKind {
    Start,
    Deadline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recurrence {
    pub from: NaiveDate,
    pub until: NaiveDate,
    pub weekdays: Vec<u8>,
}

pub fn checked_recurrence(rule: &Recurrence) -> Result<Recurrence, String> {
    if rule.until < rule.from {
        return Err("Повтор заканчивается раньше начала".into());
    }
    let days = rule.until.signed_duration_since(rule.from).num_days();
    if days > 366 * 3 {
        return Err("Диапазон повтора длиннее трёх лет".into());
    }
    let mut weekdays = rule.weekdays.clone();
    weekdays.sort_unstable();
    weekdays.dedup();
    if weekdays.is_empty() || weekdays.iter().any(|day| !(1..=7).contains(day)) {
        return Err("Выберите дни недели".into());
    }
    let mut cursor = rule.from;
    let mut found = false;
    while cursor <= rule.until {
        if weekdays.contains(&(cursor.weekday().number_from_monday() as u8)) {
            found = true;
            break;
        }
        cursor = cursor
            .checked_add_days(Days::new(1))
            .ok_or("Слишком далёкая дата повтора")?;
    }
    if !found {
        return Err("В этом диапазоне нет выбранных дней".into());
    }
    Ok(Recurrence {
        from: rule.from,
        until: rule.until,
        weekdays,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deadline_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub snooze_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_start_notified_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_deadline_notified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
    /// Telegram recipients chosen for this card. Empty means the reminder stays on this computer.
    #[serde(default)]
    pub recipient_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardInput {
    pub id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub status: String,
    pub start_at: Option<DateTime<Utc>>,
    pub deadline_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
    /// None keeps the previous list on edit and the default list on create.
    #[serde(default)]
    pub recipient_ids: Option<Vec<String>>,
}

pub const MAX_RECIPIENTS: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramRecipient {
    pub id: String,
    pub chat_id: i64,
    pub name: String,
    pub username: String,
    #[serde(default)]
    pub always_send: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramContact {
    pub chat_id: i64,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub autostart: bool,
    #[serde(default = "default_columns")]
    pub columns: Vec<Column>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub telegram_token: String,
    #[serde(default)]
    pub telegram_bot: String,
    #[serde(default)]
    pub telegram_invite: String,
    #[serde(default)]
    pub telegram_offset: i64,
    #[serde(default)]
    pub recipients: Vec<TelegramRecipient>,
    /// Second bot: people send tasks into the calendar from any chat.
    #[serde(default)]
    pub task_bot_token: String,
    #[serde(default)]
    pub task_bot: String,
    #[serde(default)]
    pub task_bot_offset: i64,
    /// False until the first poll skips updates that arrived before the token was saved.
    #[serde(default)]
    pub task_bot_ready: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: true,
            columns: default_columns(),
            theme: default_theme(),
            telegram_token: String::new(),
            telegram_bot: String::new(),
            telegram_invite: String::new(),
            telegram_offset: 0,
            recipients: Vec::new(),
            task_bot_token: String::new(),
            task_bot: String::new(),
            task_bot_offset: 0,
            task_bot_ready: false,
        }
    }
}

pub fn default_recipient_ids(settings: &Settings) -> Vec<String> {
    settings
        .recipients
        .iter()
        .filter(|recipient| recipient.always_send)
        .map(|recipient| recipient.id.clone())
        .collect()
}

/// `specified` is what the editor sent. `existing` is the saved card; None means the card is new.
pub fn recipient_ids_for_save(
    specified: Option<&[String]>,
    existing: Option<&[String]>,
    known: &[String],
    defaults: &[String],
) -> Vec<String> {
    let source = specified.or(existing).unwrap_or(defaults);
    let mut chosen = Vec::new();
    for id in source {
        if known.iter().any(|item| item == id) && !chosen.iter().any(|item: &String| item == id) {
            chosen.push(id.clone());
        }
    }
    chosen
}

pub fn merge_recipients(
    recipients: &mut Vec<TelegramRecipient>,
    contacts: &[TelegramContact],
) -> bool {
    let mut changed = false;
    for contact in contacts {
        if contact.chat_id == 0 || contact.name.trim().is_empty() {
            continue;
        }
        if let Some(found) = recipients
            .iter_mut()
            .find(|item| item.chat_id == contact.chat_id)
        {
            if found.name != contact.name || found.username != contact.username {
                found.name = contact.name.clone();
                found.username = contact.username.clone();
                changed = true;
            }
        } else if recipients.len() < MAX_RECIPIENTS {
            recipients.push(TelegramRecipient {
                id: uuid::Uuid::new_v4().to_string(),
                chat_id: contact.chat_id,
                name: contact.name.clone(),
                username: contact.username.clone(),
                always_send: false,
            });
            changed = true;
        }
    }
    changed
}

/// What should fire right now.
/// A passed deadline replaces the start reminder.
/// Snooze silences the whole card. A fresh notification waits ten minutes after the previous one.
pub fn nag_kind(columns: &[Column], card: &Card, now: DateTime<Utc>) -> Option<NagKind> {
    if card.status == DONE_ID {
        return None;
    }
    if let Some(until) = card.snooze_until {
        if now < until {
            return None;
        }
    }
    if let Some(deadline) = card.deadline_at {
        if now >= deadline {
            return ready(card.last_deadline_notified_at, now).then_some(NagKind::Deadline);
        }
    }
    if awaits_start(columns, &card.status) {
        if let Some(start) = card.start_at {
            if now >= start {
                return ready(card.last_start_notified_at, now).then_some(NagKind::Start);
            }
        }
    }
    None
}

/// The popup and the in-app strip stay up while this is true.
/// Snooze silences the card. The ten-minute gap only spaces out toasts, it does not clear the popup.
pub fn needs_attention(columns: &[Column], card: &Card, now: DateTime<Utc>) -> bool {
    if card.status == DONE_ID {
        return false;
    }
    if card.snooze_until.is_some_and(|until| now < until) {
        return false;
    }
    if card.deadline_at.is_some_and(|deadline| now >= deadline) {
        return true;
    }
    awaits_start(columns, &card.status) && card.start_at.is_some_and(|start| now >= start)
}

fn ready(last: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    match last {
        None => true,
        Some(last) => now.signed_duration_since(last) >= NAG_GAP,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnChange {
    pub columns: Vec<Column>,
    pub cards: Vec<Card>,
}

pub fn mark_notified(card: &mut Card, kind: NagKind, now: DateTime<Utc>) {
    match kind {
        NagKind::Start => card.last_start_notified_at = Some(now),
        NagKind::Deadline => card.last_deadline_notified_at = Some(now),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 13, hour, minute, 0).unwrap()
    }

    fn kind(card: &Card, now: DateTime<Utc>) -> Option<NagKind> {
        nag_kind(&default_columns(), card, now)
    }

    fn attention(card: &Card, now: DateTime<Utc>) -> bool {
        needs_attention(&default_columns(), card, now)
    }

    fn card() -> Card {
        Card {
            id: "1".into(),
            title: "помыться".into(),
            description: String::new(),
            status: TODO_ID.into(),
            recurrence: None,
            start_at: Some(at(9, 0)),
            deadline_at: Some(at(18, 0)),
            snooze_until: None,
            last_start_notified_at: None,
            last_deadline_notified_at: None,
            created_at: at(8, 0),
            completed_at: None,
            recipient_ids: Vec::new(),
        }
    }

    #[test]
    fn stays_quiet_before_the_start() {
        let mut item = card();
        item.deadline_at = None;
        assert_eq!(kind(&item, at(8, 59)), None);
    }

    #[test]
    fn start_nags_until_done_and_then_waits() {
        let mut item = card();
        item.deadline_at = None;
        assert_eq!(kind(&item, at(9, 0)), Some(NagKind::Start));
        mark_notified(&mut item, NagKind::Start, at(9, 0));
        assert_eq!(kind(&item, at(9, 9)), None);
        assert_eq!(kind(&item, at(9, 10)), Some(NagKind::Start));
        item.status = DONE_ID.into();
        assert_eq!(kind(&item, at(12, 0)), None);
    }

    #[test]
    fn start_reminder_stays_in_todo() {
        let mut item = card();
        item.deadline_at = None;
        item.status = "doing".into();
        assert_eq!(kind(&item, at(10, 0)), None);
        item.status = TODO_ID.into();
        assert_eq!(kind(&item, at(10, 0)), Some(NagKind::Start));
        item.status = "doing".into();
        item.deadline_at = Some(at(18, 0));
        assert_eq!(kind(&item, at(18, 0)), Some(NagKind::Deadline));
    }

    #[test]
    fn deadline_replaces_the_start_nag_and_snooze_pauses_both() {
        let mut item = card();
        assert_eq!(kind(&item, at(10, 0)), Some(NagKind::Start));
        assert_eq!(kind(&item, at(18, 0)), Some(NagKind::Deadline));
        mark_notified(&mut item, NagKind::Deadline, at(18, 0));
        assert_eq!(kind(&item, at(18, 5)), None);
        item.snooze_until = Some(at(20, 0));
        item.last_deadline_notified_at = None;
        assert_eq!(kind(&item, at(19, 0)), None);
        assert_eq!(kind(&item, at(20, 0)), Some(NagKind::Deadline));
    }

    #[test]
    fn popup_stays_after_a_toast_and_leaves_on_snooze() {
        let mut item = card();
        mark_notified(&mut item, NagKind::Deadline, at(18, 0));
        assert_eq!(kind(&item, at(18, 5)), None);
        assert!(attention(&item, at(18, 5)));
        item.snooze_until = Some(at(18, 20));
        assert!(!attention(&item, at(18, 5)));
    }

    #[test]
    fn a_card_without_times_never_nags() {
        let mut item = card();
        item.start_at = None;
        item.deadline_at = None;
        assert_eq!(kind(&item, at(23, 0)), None);
    }

    #[test]
    fn start_nags_only_above_doing_and_deadline_nags_until_done() {
        let columns = vec![
            Column {
                id: TODO_ID.into(),
                title: "К выполнению".into(),
            },
            Column {
                id: "above".into(),
                title: "Проверка".into(),
            },
            Column {
                id: DOING_ID.into(),
                title: "В работе".into(),
            },
            Column {
                id: "below".into(),
                title: "Ждёт".into(),
            },
            Column {
                id: DONE_ID.into(),
                title: "Готово".into(),
            },
        ];
        let mut item = card();
        item.deadline_at = None;
        item.status = "above".into();
        assert_eq!(nag_kind(&columns, &item, at(10, 0)), Some(NagKind::Start));
        item.status = DOING_ID.into();
        assert_eq!(nag_kind(&columns, &item, at(10, 0)), None);
        item.status = "below".into();
        assert_eq!(nag_kind(&columns, &item, at(10, 0)), None);
        item.deadline_at = Some(at(18, 0));
        assert_eq!(
            nag_kind(&columns, &item, at(18, 0)),
            Some(NagKind::Deadline)
        );
        item.status = DOING_ID.into();
        assert_eq!(
            nag_kind(&columns, &item, at(18, 0)),
            Some(NagKind::Deadline)
        );
        item.status = DONE_ID.into();
        assert_eq!(nag_kind(&columns, &item, at(18, 0)), None);
    }

    #[test]
    fn doing_stays_between_custom_statuses() {
        let columns = normalize_columns(vec![
            Column {
                id: TODO_ID.into(),
                title: "черновик".into(),
            },
            Column {
                id: "a".into(),
                title: "А".into(),
            },
            Column {
                id: DOING_ID.into(),
                title: "занято".into(),
            },
            Column {
                id: "b".into(),
                title: "Б".into(),
            },
            Column {
                id: DONE_ID.into(),
                title: "конец".into(),
            },
        ]);
        let ids: Vec<_> = columns.iter().map(|column| column.id.as_str()).collect();
        assert_eq!(ids, vec![TODO_ID, "a", DOING_ID, "b", DONE_ID]);
        assert_eq!(columns[0].title, "К выполнению");
        assert_eq!(columns[2].title, "В работе");
        assert_eq!(columns[4].title, "Готово");
    }

    #[test]
    fn a_missing_doing_column_returns_before_customs() {
        let columns = normalize_columns(vec![
            Column {
                id: TODO_ID.into(),
                title: "К выполнению".into(),
            },
            Column {
                id: "a".into(),
                title: "А".into(),
            },
            Column {
                id: DONE_ID.into(),
                title: "Готово".into(),
            },
        ]);
        let ids: Vec<_> = columns.iter().map(|column| column.id.as_str()).collect();
        assert_eq!(ids, vec![TODO_ID, DOING_ID, "a", DONE_ID]);
    }

    #[test]
    fn card_without_recipients_defaults_to_nobody() {
        let card: Card = serde_json::from_str(
            r#"{"id":"1","title":"a","status":"todo","createdAt":"2026-11-13T08:00:00Z"}"#,
        )
        .unwrap();
        assert!(card.recipient_ids.is_empty());
    }

    #[test]
    fn settings_without_telegram_stay_empty() {
        let settings: Settings = serde_json::from_str(r#"{"autostart":false}"#).unwrap();
        assert!(settings.telegram_token.is_empty());
        assert!(settings.recipients.is_empty());
        assert!(settings.task_bot_token.is_empty());
        assert!(!settings.task_bot_ready);
    }

    #[test]
    fn new_task_takes_only_people_marked_always() {
        let mut settings = Settings::default();
        settings.recipients = vec![
            TelegramRecipient {
                id: "a".into(),
                chat_id: 1,
                name: "Аня".into(),
                username: String::new(),
                always_send: true,
            },
            TelegramRecipient {
                id: "b".into(),
                chat_id: 2,
                name: "Боря".into(),
                username: String::new(),
                always_send: false,
            },
        ];
        assert_eq!(default_recipient_ids(&settings), vec!["a".to_string()]);
    }

    #[test]
    fn new_task_takes_everyone_only_when_always_is_on() {
        let known = vec!["a".to_string(), "b".to_string()];
        let defaults = known.clone();
        assert!(recipient_ids_for_save(None, None, &known, &[]).is_empty());
        assert_eq!(
            recipient_ids_for_save(None, None, &known, &defaults),
            defaults
        );
        let picked = vec!["b".to_string(), "missing".to_string(), "b".to_string()];
        assert_eq!(
            recipient_ids_for_save(Some(&picked), None, &known, &defaults),
            vec!["b".to_string()]
        );
        let saved = vec!["a".to_string()];
        assert_eq!(
            recipient_ids_for_save(None, Some(&saved), &known, &defaults),
            saved
        );
    }

    #[test]
    fn merge_keeps_the_same_person_and_stops_at_the_cap() {
        let mut recipients = vec![TelegramRecipient {
            id: "keep".into(),
            chat_id: 42,
            name: "Анна".into(),
            username: String::new(),
            always_send: true,
        }];
        assert!(merge_recipients(
            &mut recipients,
            &[TelegramContact {
                chat_id: 42,
                name: "Анна Иванова".into(),
                username: "anna".into(),
            }]
        ));
        assert_eq!(recipients.len(), 1);
        assert_eq!(recipients[0].id, "keep");
        assert_eq!(recipients[0].username, "anna");
        assert!(recipients[0].always_send);
        assert!(!merge_recipients(
            &mut recipients,
            &[TelegramContact {
                chat_id: 42,
                name: "Анна Иванова".into(),
                username: "anna".into(),
            }]
        ));

        let mut full = Vec::new();
        for chat_id in 1..=MAX_RECIPIENTS as i64 {
            full.push(TelegramRecipient {
                id: chat_id.to_string(),
                chat_id,
                name: "человек".into(),
                username: String::new(),
                always_send: false,
            });
        }
        assert!(!merge_recipients(
            &mut full,
            &[TelegramContact {
                chat_id: 10_000,
                name: "лишний".into(),
                username: String::new(),
            }]
        ));
        assert_eq!(full.len(), MAX_RECIPIENTS);
    }
}
