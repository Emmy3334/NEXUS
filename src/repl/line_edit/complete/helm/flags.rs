//! Common flags for curated host `helm` verbs.

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "install" => Some(INSTALL),
        "upgrade" => Some(UPGRADE),
        "uninstall" => Some(UNINSTALL),
        "list" => Some(LIST),
        "status" => Some(STATUS),
        "rollback" => Some(ROLLBACK),
        "template" => Some(TEMPLATE),
        "lint" => Some(LINT),
        "repo" => Some(REPO),
        "search" => Some(SEARCH),
        "get" => Some(GET),
        "history" => Some(HISTORY),
        "pull" | "push" => Some(PULL_PUSH),
        "show" => Some(SHOW),
        "test" => Some(TEST),
        "dependency" => Some(DEPENDENCY),
        _ => None,
    }
}

pub(super) fn collect(verb: &str, prefix: &str, out: &mut Vec<String>) {
    let Some(flags) = flags_for(verb) else {
        return;
    };
    for flag in flags {
        if flag.starts_with(prefix) {
            out.push((*flag).to_owned());
        }
    }
}

const INSTALL: &[&str] = &[
    "-n",
    "-f",
    "--namespace",
    "--values",
    "--set",
    "--set-string",
    "--create-namespace",
    "--wait",
    "--timeout",
    "--dry-run",
    "--version",
    "--repo",
];
const UPGRADE: &[&str] = &[
    "-n",
    "-f",
    "--namespace",
    "--values",
    "--set",
    "--install",
    "--wait",
    "--timeout",
    "--dry-run",
    "--version",
    "--reset-values",
    "--reuse-values",
];
const UNINSTALL: &[&str] = &["-n", "--namespace", "--keep-history", "--wait", "--timeout"];
const LIST: &[&str] = &[
    "-a",
    "-n",
    "-A",
    "--all",
    "--namespace",
    "--all-namespaces",
    "--filter",
    "--deployed",
    "--failed",
    "--pending",
    "--short",
];
const STATUS: &[&str] = &[
    "-n",
    "--namespace",
    "--revision",
    "--show-desc",
    "--show-resources",
];
const ROLLBACK: &[&str] = &[
    "-n",
    "--namespace",
    "--wait",
    "--timeout",
    "--cleanup-on-fail",
];
const TEMPLATE: &[&str] = &[
    "-n",
    "-f",
    "--namespace",
    "--values",
    "--set",
    "--output-dir",
    "--show-only",
    "--version",
    "--repo",
];
const LINT: &[&str] = &[
    "-f",
    "--values",
    "--set",
    "--strict",
    "--quiet",
    "--with-subcharts",
];
const REPO: &[&str] = &["--help"];
const SEARCH: &[&str] = &[
    "--versions",
    "--regexp",
    "--version",
    "--devel",
    "--max-col-width",
];
const GET: &[&str] = &["-n", "--namespace", "--revision"];
const HISTORY: &[&str] = &["-n", "--namespace", "--max", "--output"];
const PULL_PUSH: &[&str] = &[
    "--version",
    "--destination",
    "--untar",
    "--prov",
    "--verify",
];
const SHOW: &[&str] = &["--version", "--repo", "--devel"];
const TEST: &[&str] = &["-n", "--namespace", "--timeout", "--filter", "--logs"];
const DEPENDENCY: &[&str] = &["--skip-refresh", "--keyring", "--verify"];
