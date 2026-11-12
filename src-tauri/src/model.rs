use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub const NAG_GAP: Duration = Duration::minutes(10);
pub const MAX_SNOOZE_MINUTES: u32 = 24 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Todo,
    Doing,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NagKind {
    Start,
    Deadline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub title: String,
    pub status: Status,
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardInput {
    pub id: Option<String>,
    pub title: String,
    pub status: Status,
    pub start_at: Option<DateTime<Utc>>,
    pub deadline_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { autostart: true }
    }
}

/// What should fire right now.
/// A passed deadline replaces the start reminder.
/// Snooze silences the whole card. A fresh notification waits ten minutes after the previous one.
pub fn nag_kind(card: &Card, now: DateTime<Utc>) -> Option<NagKind> {
    if card.status == Status::Done {
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
    if let Some(start) = card.start_at {
        if now >= start {
            return ready(card.last_start_notified_at, now).then_some(NagKind::Start);
        }
    }
    None
}

fn ready(last: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    match last {
        None => true,
        Some(last) => now.signed_duration_since(last) >= NAG_GAP,
    }
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
            status: Status::Todo,
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
        item.status = Status::Done;
        assert_eq!(nag_kind(&item, at(12, 0)), None);
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
    fn a_card_without_times_never_nags() {
        let mut item = card();
        item.start_at = None;
        item.deadline_at = None;
        assert_eq!(nag_kind(&item, at(23, 0)), None);
    }
}
