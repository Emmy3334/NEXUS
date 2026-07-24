//! Common flags for curated host `aws` services.

pub(super) fn flags_for(verb: &str) -> Option<&'static [&'static str]> {
    match verb {
        "s3" | "s3api" => Some(S3),
        "ec2" => Some(EC2),
        "iam" => Some(IAM),
        "lambda" => Some(LAMBDA),
        "sts" => Some(STS),
        "logs" => Some(LOGS),
        "ecr" | "ecs" | "eks" => Some(CONTAINERS),
        "rds" | "dynamodb" => Some(DATA),
        "cloudformation" => Some(CFN),
        "configure" => Some(CONFIGURE),
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

const S3: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--endpoint-url",
    "--recursive",
    "--exclude",
    "--include",
    "--dryrun",
    "--delete",
    "--acl",
    "--no-cli-pager",
];

const EC2: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--filters",
    "--instance-ids",
    "--query",
    "--no-cli-pager",
];

const IAM: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--user-name",
    "--role-name",
    "--group-name",
    "--query",
    "--no-cli-pager",
];

const LAMBDA: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--function-name",
    "--payload",
    "--query",
    "--no-cli-pager",
];

const STS: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--role-arn",
    "--role-session-name",
    "--duration-seconds",
    "--query",
    "--no-cli-pager",
];

const LOGS: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--log-group-name",
    "--log-stream-name",
    "--follow",
    "--query",
    "--no-cli-pager",
];

const CONTAINERS: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--cluster",
    "--service",
    "--repository-name",
    "--query",
    "--no-cli-pager",
];

const DATA: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--table-name",
    "--db-instance-identifier",
    "--query",
    "--no-cli-pager",
];

const CFN: &[&str] = &[
    "--region",
    "--profile",
    "--output",
    "--stack-name",
    "--template-body",
    "--parameters",
    "--query",
    "--no-cli-pager",
];

const CONFIGURE: &[&str] = &["--profile", "--region"];
