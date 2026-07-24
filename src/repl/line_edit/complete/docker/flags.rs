//! Flags for curated `@docker` verbs (builtin surface).

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "ps" => Some(PS),
        "logs" => Some(LOGS),
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

const PS: &[&str] = &["-a", "-q", "--all", "--quiet", "--filter", "--format"];
const LOGS: &[&str] = &[
    "-f",
    "-t",
    "--follow",
    "--tail",
    "--timestamps",
    "--details",
];
