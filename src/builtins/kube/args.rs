//! Shared flag parsing for `@kube` (`-n`, `-A`).

use crate::kube::PodScope;

/// Parse `-A` / `--all-namespaces` and `-n` / `--namespace <ns>`.
pub(super) fn pod_scope(args: &[String]) -> Result<PodScope, String> {
    let mut all = false;
    let mut namespace: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-A" | "--all-namespaces" => all = true,
            "-n" | "--namespace" => {
                let ns = args
                    .get(i + 1)
                    .ok_or_else(|| "usage: -n <namespace>".to_owned())?;
                namespace = Some(ns.clone());
                i += 1;
            }
            flag if flag.starts_with("-n=") || flag.starts_with("--namespace=") => {
                let ns = flag.split_once('=').map(|(_, v)| v).unwrap_or("");
                if ns.is_empty() {
                    return Err("usage: -n <namespace>".into());
                }
                namespace = Some(ns.to_owned());
            }
            _ => {}
        }
        i += 1;
    }
    if all {
        return Ok(PodScope::All);
    }
    Ok(match namespace {
        Some(ns) => PodScope::Namespace(ns),
        None => PodScope::Default,
    })
}

/// Namespace for single-resource ops (`logs` / `describe`); ignores `-A`.
pub(super) fn namespace_opt(args: &[String]) -> Result<Option<String>, String> {
    match pod_scope(args)? {
        PodScope::Namespace(ns) => Ok(Some(ns)),
        PodScope::Default | PodScope::All => Ok(None),
    }
}

/// Whether `-f` / `--follow` appears among args.
#[must_use]
pub(super) fn wants_follow(args: &[String]) -> bool {
    args.iter().any(|a| matches!(a.as_str(), "-f" | "--follow"))
}

/// First non-flag positional after `skip` leading args (subcommand words).
pub(super) fn first_positional(args: &[String], skip: usize) -> Option<&str> {
    let mut i = skip;
    while i < args.len() {
        match args[i].as_str() {
            "-A" | "--all-namespaces" | "-f" | "--follow" => i += 1,
            "-n" | "--namespace" => i += 2,
            flag if flag.starts_with('-') => i += 1,
            other => return Some(other),
        }
    }
    None
}

/// Split `@kube exec …` into `(pod, command)` after the `exec` token (`args[0]`).
///
/// Prefer `@kube exec [-n NS] <pod> -- <cmd>…`; also allow `<pod> <cmd>…`.
pub(super) fn exec_pod_and_cmd(args: &[String]) -> Result<(&str, Vec<String>), String> {
    let usage = "usage: @kube exec [-n NS] <pod> -- <cmd> [args…]";
    let Some(pod) = first_positional(args, 1) else {
        return Err(usage.into());
    };
    let pod_idx = args
        .iter()
        .position(|a| a == pod)
        .ok_or_else(|| usage.to_owned())?;
    let rest = &args[pod_idx + 1..];
    let cmd = if rest.first().map(String::as_str) == Some("--") {
        rest[1..].to_vec()
    } else {
        rest.to_vec()
    };
    if cmd.is_empty() {
        return Err(usage.into());
    }
    Ok((pod, cmd))
}
