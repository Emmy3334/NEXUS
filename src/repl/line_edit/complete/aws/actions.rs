//! Curated `aws <service> <action>` completions.

use super::super::matchers::matches_prefix;

const S3: &[&str] = &["ls", "cp", "sync", "mb", "rb", "rm", "presign"];

/// True when completing the first action token after a curated service.
pub(super) fn awaiting(words: &[&str], service: &str) -> bool {
    let mut i = 1;
    while i < words.len() {
        if words[i].starts_with('-') {
            i += 1;
            continue;
        }
        if words[i] == service {
            return words[i + 1..].iter().all(|w| w.starts_with('-'));
        }
        i += 1;
    }
    false
}

pub(super) fn collect(service: &str, prefix: &str, out: &mut Vec<String>) {
    let actions = match service {
        "s3" => S3,
        _ => return,
    };
    for action in actions {
        if matches_prefix(action, prefix) {
            out.push((*action).to_owned());
        }
    }
}
