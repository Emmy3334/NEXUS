//! Exported shell env as Docker/Kube assignment lists.

use crate::env::ShellEnvironment;

/// `KEY=VALUE` pairs from the exported shell map (same as external children).
///
/// Host `PATH` / loader paths are omitted so image binaries stay on the
/// container’s default search path (heal command→image map depends on this).
#[must_use]
pub fn docker_env(shell_env: &ShellEnvironment) -> Vec<String> {
    shell_env
        .iter()
        .filter(|(k, _)| forward_key(k))
        .map(|(k, v)| format!("{k}={v}"))
        .collect()
}

/// Kubernetes `EnvVar` list from the exported shell map.
#[must_use]
pub fn kube_env(shell_env: &ShellEnvironment) -> Vec<k8s_openapi::api::core::v1::EnvVar> {
    shell_env
        .iter()
        .filter(|(k, _)| forward_key(k))
        .map(|(k, v)| k8s_openapi::api::core::v1::EnvVar {
            name: k.to_owned(),
            value: Some(v.to_owned()),
            ..Default::default()
        })
        .collect()
}

fn forward_key(key: &str) -> bool {
    !matches!(
        key,
        "PATH" | "LD_LIBRARY_PATH" | "DYLD_LIBRARY_PATH" | "DYLD_FALLBACK_LIBRARY_PATH"
    )
}
