//! READY and RESTARTS columns from container statuses.

use k8s_openapi::api::core::v1::Pod;

#[must_use]
pub(super) fn ready_count(pod: &Pod) -> String {
    let Some(statuses) = pod
        .status
        .as_ref()
        .and_then(|s| s.container_statuses.as_ref())
    else {
        let expected = pod.spec.as_ref().map(|s| s.containers.len()).unwrap_or(0);
        return format!("0/{expected}");
    };
    let total = statuses.len();
    let ready = statuses.iter().filter(|c| c.ready).count();
    format!("{ready}/{total}")
}

#[must_use]
pub(super) fn restarts(pod: &Pod) -> i32 {
    pod.status
        .as_ref()
        .and_then(|s| s.container_statuses.as_ref())
        .map(|statuses| statuses.iter().map(|c| c.restart_count).sum())
        .unwrap_or(0)
}
