//! Flags for curated `@kube` verbs (builtin surface).

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "pods" | "get" => Some(PODS),
        "logs" => Some(LOGS),
        "exec" => Some(EXEC),
        "describe" => Some(DESCRIBE),
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

const PODS: &[&str] = &["-n", "-A", "--namespace", "--all-namespaces"];
const LOGS: &[&str] = &["-n", "-f", "--namespace", "--follow"];
const EXEC: &[&str] = &["-n", "--namespace"];
const DESCRIBE: &[&str] = &["-n", "--namespace"];
