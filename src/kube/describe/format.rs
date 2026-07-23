//! Format a pod into kubectl-like describe lines.

use super::super::pods::{ready_count, restarts};
use k8s_openapi::api::core::v1::Pod;
use kube::ResourceExt;

pub(super) fn format_pod(pod: &Pod) -> String {
    let mut out = String::new();
    line(&mut out, "Name", &pod.name_any());
    line(
        &mut out,
        "Namespace",
        &pod.namespace().unwrap_or_else(|| "default".into()),
    );
    line(&mut out, "Node", &node_name(pod));
    line(&mut out, "Status", &phase(pod));
    line(&mut out, "Ready", &ready_count(pod));
    line(&mut out, "Restarts", &restarts(pod).to_string());
    line(&mut out, "Images", &images(pod));
    line(&mut out, "Conditions", &conditions(pod));
    out
}

fn line(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(":\t");
    out.push_str(value);
    out.push('\n');
}

fn phase(pod: &Pod) -> String {
    pod.status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".into())
}

fn node_name(pod: &Pod) -> String {
    pod.spec
        .as_ref()
        .and_then(|s| s.node_name.clone())
        .unwrap_or_else(|| "<none>".into())
}

fn images(pod: &Pod) -> String {
    let Some(spec) = pod.spec.as_ref() else {
        return "<none>".into();
    };
    let imgs: Vec<&str> = spec
        .containers
        .iter()
        .filter_map(|c| c.image.as_deref())
        .collect();
    if imgs.is_empty() {
        "<none>".into()
    } else {
        imgs.join(", ")
    }
}

fn conditions(pod: &Pod) -> String {
    let Some(conds) = pod.status.as_ref().and_then(|s| s.conditions.as_ref()) else {
        return "<none>".into();
    };
    let parts: Vec<String> = conds
        .iter()
        .map(|c| format!("{}={}", c.type_, c.status))
        .collect();
    if parts.is_empty() {
        "<none>".into()
    } else {
        parts.join(", ")
    }
}
