//! Common flags for curated host `docker` verbs.

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "ps" => Some(PS),
        "logs" => Some(LOGS),
        "run" => Some(RUN),
        "exec" => Some(EXEC),
        "rm" => Some(RM),
        "images" => Some(IMAGES),
        "pull" => Some(PULL),
        "build" => Some(BUILD),
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

const PS: &[&str] = &[
    "-a", "-q", "-s", "-n", "--all", "--quiet", "--filter", "--format",
];
const LOGS: &[&str] = &[
    "-f",
    "-t",
    "--follow",
    "--tail",
    "--timestamps",
    "--details",
];
const RUN: &[&str] = &[
    "-d",
    "-it",
    "-p",
    "-e",
    "-v",
    "--rm",
    "--name",
    "--network",
    "--entrypoint",
];
const EXEC: &[&str] = &[
    "-it",
    "-u",
    "-e",
    "-w",
    "--detach",
    "--privileged",
    "--user",
];
const RM: &[&str] = &["-f", "-v", "--force", "--volumes", "--link"];
const IMAGES: &[&str] = &["-a", "-q", "--all", "--quiet", "--filter", "--digests"];
const PULL: &[&str] = &["-a", "-q", "--all-tags", "--quiet", "--platform"];
const BUILD: &[&str] = &[
    "-t",
    "-f",
    "--tag",
    "--file",
    "--no-cache",
    "--pull",
    "--target",
    "--build-arg",
];
