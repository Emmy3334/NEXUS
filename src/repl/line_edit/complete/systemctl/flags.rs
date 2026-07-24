//! Common flags for curated host `systemctl` verbs.

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "start" | "stop" | "restart" | "reload" => Some(UNIT_ACTION),
        "status" => Some(STATUS),
        "enable" | "disable" => Some(ENABLE),
        "list-units" => Some(LIST_UNITS),
        "list-unit-files" => Some(LIST_UNIT_FILES),
        "daemon-reload" => Some(DAEMON_RELOAD),
        "show" => Some(SHOW),
        "cat" => Some(CAT),
        "kill" => Some(KILL),
        "mask" | "unmask" => Some(MASK),
        "is-active" | "is-enabled" | "is-failed" => Some(IS),
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

const UNIT_ACTION: &[&str] = &[
    "--user",
    "--system",
    "--no-block",
    "--no-ask-password",
    "-H",
    "--host",
];
const STATUS: &[&str] = &[
    "--user",
    "--system",
    "-l",
    "-n",
    "--full",
    "--lines",
    "--no-pager",
    "-H",
    "--host",
];
const ENABLE: &[&str] = &[
    "--user",
    "--system",
    "--now",
    "--runtime",
    "--no-reload",
    "-H",
    "--host",
];
const LIST_UNITS: &[&str] = &[
    "--user",
    "--system",
    "-a",
    "--all",
    "--type",
    "--state",
    "--failed",
    "--no-pager",
    "-H",
    "--host",
];
const LIST_UNIT_FILES: &[&str] = &[
    "--user",
    "--system",
    "--type",
    "--state",
    "--no-pager",
    "-H",
    "--host",
];
const DAEMON_RELOAD: &[&str] = &["--user", "--system", "-H", "--host"];
const SHOW: &[&str] = &[
    "--user",
    "--system",
    "-p",
    "--property",
    "--value",
    "--all",
    "-H",
    "--host",
];
const CAT: &[&str] = &["--user", "--system", "-H", "--host"];
const KILL: &[&str] = &[
    "--user",
    "--system",
    "-s",
    "--signal",
    "-H",
    "--host",
    "--kill-whom",
];
const MASK: &[&str] = &["--user", "--system", "--runtime", "--now", "-H", "--host"];
const IS: &[&str] = &["--user", "--system", "-H", "--host", "--quiet"];
