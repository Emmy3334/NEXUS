//! Common flags for curated host `gcloud` groups.

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "compute" => Some(COMPUTE),
        "container" => Some(CONTAINER),
        "run" => Some(RUN),
        "iam" => Some(IAM),
        "auth" => Some(AUTH),
        "config" => Some(CONFIG),
        "projects" => Some(PROJECTS),
        "storage" => Some(STORAGE),
        "functions" => Some(FUNCTIONS),
        "sql" => Some(SQL),
        "logging" => Some(LOGGING),
        "pubsub" => Some(PUBSUB),
        "secrets" => Some(SECRETS),
        "services" => Some(SERVICES),
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

const COMPUTE: &[&str] = &[
    "--project",
    "--zone",
    "--region",
    "--quiet",
    "--format",
    "--filter",
    "--verbosity",
    "--configuration",
    "--account",
];

const CONTAINER: &[&str] = &[
    "--project",
    "--zone",
    "--region",
    "--cluster",
    "--quiet",
    "--format",
    "--filter",
    "--verbosity",
];

const RUN: &[&str] = &[
    "--project",
    "--region",
    "--platform",
    "--image",
    "--quiet",
    "--format",
    "--verbosity",
    "--allow-unauthenticated",
];

const IAM: &[&str] = &[
    "--project",
    "--quiet",
    "--format",
    "--filter",
    "--verbosity",
    "--member",
    "--role",
];

const AUTH: &[&str] = &[
    "--project",
    "--quiet",
    "--verbosity",
    "--account",
    "--brief",
    "--update-adc",
];

const CONFIG: &[&str] = &[
    "--project",
    "--quiet",
    "--verbosity",
    "--configuration",
    "--account",
];

const PROJECTS: &[&str] = &[
    "--project",
    "--quiet",
    "--format",
    "--filter",
    "--verbosity",
    "--limit",
];

const STORAGE: &[&str] = &[
    "--project",
    "--quiet",
    "--format",
    "--verbosity",
    "--recursive",
    "--gzip",
];

const FUNCTIONS: &[&str] = &[
    "--project",
    "--region",
    "--quiet",
    "--format",
    "--verbosity",
    "--gen2",
    "--runtime",
    "--trigger-http",
];

const SQL: &[&str] = &[
    "--project",
    "--quiet",
    "--format",
    "--filter",
    "--verbosity",
    "--instance",
];

const LOGGING: &[&str] = &[
    "--project",
    "--quiet",
    "--format",
    "--filter",
    "--verbosity",
    "--limit",
    "--freshness",
];

const PUBSUB: &[&str] = &[
    "--project",
    "--quiet",
    "--format",
    "--verbosity",
    "--message-body",
    "--attribute",
];

const SECRETS: &[&str] = &[
    "--project",
    "--quiet",
    "--format",
    "--verbosity",
    "--data-file",
    "--replication-policy",
];

const SERVICES: &[&str] = &[
    "--project",
    "--quiet",
    "--format",
    "--filter",
    "--verbosity",
    "--enabled",
];
