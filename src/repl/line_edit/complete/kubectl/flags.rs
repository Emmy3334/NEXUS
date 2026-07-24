//! Common flags for curated `kubectl` verbs.

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "get" => Some(GET),
        "describe" => Some(DESCRIBE),
        "logs" => Some(LOGS),
        "apply" => Some(APPLY),
        "delete" => Some(DELETE),
        "exec" => Some(EXEC),
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

const GET: &[&str] = &[
    "-n",
    "-A",
    "-o",
    "-l",
    "-w",
    "--namespace",
    "--all-namespaces",
    "--output",
    "--selector",
    "--watch",
    "--context",
];
const DESCRIBE: &[&str] = &["-n", "-A", "--namespace", "--all-namespaces", "--context"];
const LOGS: &[&str] = &[
    "-n",
    "-f",
    "-p",
    "--namespace",
    "--follow",
    "--previous",
    "--tail",
    "--timestamps",
    "--context",
];
const APPLY: &[&str] = &[
    "-f",
    "-n",
    "-k",
    "--filename",
    "--namespace",
    "--dry-run",
    "--server-side",
    "--context",
];
const DELETE: &[&str] = &[
    "-n",
    "-f",
    "-l",
    "--namespace",
    "--filename",
    "--force",
    "--grace-period",
    "--context",
];
const EXEC: &[&str] = &[
    "-n",
    "-it",
    "-c",
    "--namespace",
    "--container",
    "--stdin",
    "--tty",
    "--context",
];
