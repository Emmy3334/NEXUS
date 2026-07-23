//! Kubectl-style relative ages (`3m50s`, `2h`, `4d`).

use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use k8s_openapi::jiff::Timestamp;

#[must_use]
pub(super) fn from_time(created: Option<&Time>) -> String {
    let Some(Time(ts)) = created else {
        return "<unknown>".into();
    };
    let secs = Timestamp::now()
        .as_second()
        .saturating_sub(ts.as_second())
        .max(0) as u64;
    format_secs(secs)
}

fn format_secs(secs: u64) -> String {
    if secs < 60 {
        return format!("{secs}s");
    }
    if secs < 3600 {
        let m = secs / 60;
        let s = secs % 60;
        return if s == 0 {
            format!("{m}m")
        } else {
            format!("{m}m{s}s")
        };
    }
    if secs < 86400 {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        return if m == 0 {
            format!("{h}h")
        } else {
            format!("{h}h{m}m")
        };
    }
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    if h == 0 {
        format!("{d}d")
    } else {
        format!("{d}d{h}h")
    }
}
