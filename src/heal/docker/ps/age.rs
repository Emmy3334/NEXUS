//! Relative “Created” ages for `@docker ps`.

use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn created_ago(created: Option<i64>) -> String {
    let Some(ts) = created else {
        return "-".into();
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(ts);
    let secs = (now - ts).max(0) as u64;
    const MIN: u64 = 60;
    const HOUR: u64 = 3600;
    const DAY: u64 = 86400;
    const WEEK: u64 = 604_800;
    const MONTH: u64 = 2_629_743;
    const YEAR: u64 = 31_556_926;
    if secs < MIN {
        return format!("{secs} seconds ago");
    }
    if secs < HOUR {
        return format!("{} minutes ago", secs / MIN);
    }
    if secs < DAY {
        return format!("{} hours ago", secs / HOUR);
    }
    if secs < WEEK {
        return format!("{} days ago", secs / DAY);
    }
    if secs < MONTH {
        return format!("{} weeks ago", secs / WEEK);
    }
    if secs < YEAR {
        return format!("{} months ago", secs / MONTH);
    }
    format!("{} years ago", secs / YEAR)
}
