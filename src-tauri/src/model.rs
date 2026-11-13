use chrono::{Datelike, DateTime, Days, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub const NAG_GAP: Duration = Duration::minutes(10);
pub const MAX_SNOOZE_MINUTES: u32 = 24 * 60;
pub const TODO_ID: &str = "todo";
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
            id: "doing".into(),
            title: "В работе".into(),
        },
        Column {
            id: DONE_ID.into(),
            title: "Готово".into(),
        },
    ]
}

pub fn normalize_columns(columns: Vec<Column>) -> Vec<Column> {
    let mut middle = Vec::new();
    for column in columns {
        if column.id == TODO_ID || column.id == DONE_ID || column.title.trim().is_empty() {
            continue;
        }
        if middle.iter().any(|existing: &Column| existing.id == column.id) {
            continue;
        }
        middle.push(Column {
            id: column.id,
            title: column.title.trim().to_string(),
        });
    }
    let mut columns = vec![Column {
        id: TODO_ID.into(),
        title: "К выполнению".into(),
    }];
    columns.extend(middle);
    columns.push(Column {
        id: DONE_ID.into(),
        title: "Готово".into(),
    });
    columns
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub autostart: bool,
    #[serde(default = "default_columns")]
    pub columns: Vec<Column>,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: true,
            columns: default_columns(),
            theme: default_theme(),
        }
    }
}

/// What should fire right now.
/// A passed deadline replaces the start reminder.
/// Snooze silences the whole card. A fresh notification waits ten minutes after the previous one.
pub fn nag_kind(card: &Card, now: DateTime<Utc>) -> Option<NagKind> {
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
    if card.status == TODO_ID {
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
pub fn needs_attention(card: &Card, now: DateTime<Utc>) -> bool {
    if card.status == DONE_ID {
        return false;
    }
    if card.snooze_until.is_some_and(|until| now < until) {
        return false;
    }
    if card.deadline_at.is_some_and(|deadline| now >= deadline) {
        return true;
    }
    card.status == TODO_ID && card.start_at.is_some_and(|start| now >= start)
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
        }
    }

    #[test]
    fn stays_quiet_before_the_start() {
        let mut item = card();
        item.deadline_at = None;
        assert_eq!(nag_kind(&item, at(8, 59)), None);
    }

    #[test]
    fn start_nags_until_done_and_then_waits() {
        let mut item = card();
        item.deadline_at = None;
        assert_eq!(nag_kind(&item, at(9, 0)), Some(NagKind::Start));
        mark_notified(&mut item, NagKind::Start, at(9, 0));
        assert_eq!(nag_kind(&item, at(9, 9)), None);
        assert_eq!(nag_kind(&item, at(9, 10)), Some(NagKind::Start));
        item.status = DONE_ID.into();
        assert_eq!(nag_kind(&item, at(12, 0)), None);
    }

    #[test]
    fn start_reminder_stays_in_todo() {
        let mut item = card();
        item.deadline_at = None;
        item.status = "doing".into();
        assert_eq!(nag_kind(&item, at(10, 0)), None);
        item.status = TODO_ID.into();
        assert_eq!(nag_kind(&item, at(10, 0)), Some(NagKind::Start));
        item.status = "doing".into();
        item.deadline_at = Some(at(18, 0));
        assert_eq!(nag_kind(&item, at(18, 0)), Some(NagKind::Deadline));
    }

    #[test]
    fn deadline_replaces_the_start_nag_and_snooze_pauses_both() {
        let mut item = card();
        assert_eq!(nag_kind(&item, at(10, 0)), Some(NagKind::Start));
        assert_eq!(nag_kind(&item, at(18, 0)), Some(NagKind::Deadline));
        mark_notified(&mut item, NagKind::Deadline, at(18, 0));
        assert_eq!(nag_kind(&item, at(18, 5)), None);
        item.snooze_until = Some(at(20, 0));
        item.last_deadline_notified_at = None;
        assert_eq!(nag_kind(&item, at(19, 0)), None);
        assert_eq!(nag_kind(&item, at(20, 0)), Some(NagKind::Deadline));
    }

    #[test]
    fn popup_stays_after_a_toast_and_leaves_on_snooze() {
        let mut item = card();
        mark_notified(&mut item, NagKind::Deadline, at(18, 0));
        assert_eq!(nag_kind(&item, at(18, 5)), None);
        assert!(needs_attention(&item, at(18, 5)));
        item.snooze_until = Some(at(18, 20));
        assert!(!needs_attention(&item, at(18, 5)));
    }

    #[test]
    fn a_card_without_times_never_nags() {
        let mut item = card();
        item.start_at = None;
        item.deadline_at = None;
        assert_eq!(nag_kind(&item, at(23, 0)), None);
    }
}
