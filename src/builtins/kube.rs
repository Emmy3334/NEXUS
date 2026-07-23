//! `@kube` — native Kubernetes API (nodes / pods / logs).

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;
use crate::kube;

use std::io::{self, Write};

pub(super) fn kube_cmd(
    argv: &[String],
    _shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let args = &argv[1..];
    match args.first().map(String::as_str) {
        None | Some("help" | "-h" | "--help") => {
            writeln!(
                stderr,
                "usage: @kube nodes | @kube pods [-A] | @kube logs <pod>"
            )?;
            Ok(BuiltinResult::Status(1))
        }
        Some("nodes" | "node") => print_table(kube::list_nodes_table(), "nodes", stdout, stderr),
        Some("pods") => pods_cmd(args, stdout, stderr),
        Some("logs") => logs_cmd(args, stdout, stderr),
        Some(other) => {
            writeln!(stderr, "@kube: unknown subcommand: {other}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn pods_cmd(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let all = want_all_namespaces(args);
    let hint = if all { "cluster" } else { "default namespace" };
    print_table(kube::list_pods_table(all), hint, stdout, stderr)
}

fn print_table(
    result: io::Result<Vec<String>>,
    empty_scope: &str,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match result {
        Ok(lines) if lines.is_empty() => {
            writeln!(stderr, "@kube: no resources in {empty_scope}")?;
            if empty_scope.contains("default") {
                writeln!(stderr, "try: @kube pods -A")?;
            }
            Ok(BuiltinResult::Status(0))
        }
        Ok(lines) => {
            for line in lines {
                writeln!(stdout, "{line}")?;
            }
            Ok(BuiltinResult::Status(0))
        }
        Err(err) => {
            writeln!(stderr, "@kube: {}", kube::format_err(&err))?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn want_all_namespaces(args: &[String]) -> bool {
    args.iter().any(|a| a == "-A" || a == "--all-namespaces")
}

fn logs_cmd(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let Some(pod) = args.get(1).map(String::as_str) else {
        writeln!(stderr, "usage: @kube logs <pod>")?;
        return Ok(BuiltinResult::Status(1));
    };
    match kube::pod_logs(pod, 100) {
        Ok(text) => {
            write!(stdout, "{text}")?;
            Ok(BuiltinResult::Status(0))
        }
        Err(err) => {
            writeln!(stderr, "@kube: {}", kube::format_err(&err))?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
