//! Exported shell env as Docker/Kube assignment lists.

use super::env_policy::EnvPass;
use crate::env::ShellEnvironment;

/// `KEY=VALUE` pairs for Docker heal (allowlist + loader-path scrub).
#[must_use]
pub fn docker_env(shell_env: &ShellEnvironment) -> Vec<String> {
    let pass = EnvPass::from_shell(shell_env);
    shell_env
        .iter()
        .filter(|(k, _)| should_forward(k, &pass))
        .map(|(k, v)| format!("{k}={v}"))
        .collect()
}

/// Kubernetes `EnvVar` list for Kube heal (allowlist + loader-path scrub).
#[must_use]
pub fn kube_env(shell_env: &ShellEnvironment) -> Vec<k8s_openapi::api::core::v1::EnvVar> {
    let pass = EnvPass::from_shell(shell_env);
    shell_env
        .iter()
        .filter(|(k, _)| should_forward(k, &pass))
        .map(|(k, v)| k8s_openapi::api::core::v1::EnvVar {
            name: k.to_owned(),
            value: Some(v.to_owned()),
            ..Default::default()
        })
        .collect()
}

fn should_forward(key: &str, pass: &EnvPass) -> bool {
    scrub_ok(key) && pass.allows(key)
}

fn scrub_ok(key: &str) -> bool {
    !matches!(
        key,
        "PATH" | "LD_LIBRARY_PATH" | "DYLD_LIBRARY_PATH" | "DYLD_FALLBACK_LIBRARY_PATH"
    )
}
