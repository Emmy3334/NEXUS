//! Host `aws` service flags.

mod flags;

const SERVICES: &[&str] = &[
    "s3",
    "s3api",
    "ec2",
    "iam",
    "lambda",
    "sts",
    "logs",
    "ecr",
    "ecs",
    "eks",
    "rds",
    "dynamodb",
    "cloudformation",
    "configure",
];

/// Curated service after `aws`, skipping leading global flags.
#[must_use]
pub(super) fn verb_in(words: &[&str]) -> Option<&'static str> {
    if words.first().copied() != Some("aws") {
        return None;
    }
    for w in words.iter().skip(1) {
        if w.starts_with('-') {
            continue;
        }
        if let Some(v) = SERVICES.iter().copied().find(|v| *v == *w) {
            return Some(v);
        }
    }
    None
}

pub(super) fn collect_for_verb(verb: &str, prefix: &str, out: &mut Vec<String>) {
    if prefix.starts_with('-') {
        flags::collect(verb, prefix, out);
    }
}
