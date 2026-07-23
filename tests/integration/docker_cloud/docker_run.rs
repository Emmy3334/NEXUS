//! Shared `@docker` argv runner for integration tests.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;

pub(super) fn run_docker(args: &[&str]) -> (BuiltinResult, String, String) {
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let argv: Vec<String> = std::iter::once("@docker")
        .chain(args.iter().copied())
        .map(str::to_owned)
        .collect();
    let result = builtins::try_run(&argv, &mut env, 0, &mut stdout, &mut stderr)
        .unwrap()
        .unwrap();
    (
        result,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}
