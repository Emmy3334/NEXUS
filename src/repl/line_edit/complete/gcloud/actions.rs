//! Curated `gcloud <group> <command>` completions.

use super::super::matchers::matches_prefix;

const COMPUTE: &[&str] = &[
    "instances",
    "ssh",
    "disks",
    "zones",
    "firewall-rules",
    "networks",
    "images",
    "addresses",
    "operations",
    "regions",
];

pub(super) fn awaiting(words: &[&str], group: &str) -> bool {
    let mut i = 1;
    while i < words.len() {
        if words[i].starts_with('-') {
            i += 1;
            continue;
        }
        if words[i] == group {
            return words[i + 1..].iter().all(|w| w.starts_with('-'));
        }
        i += 1;
    }
    false
}

pub(super) fn collect(group: &str, prefix: &str, out: &mut Vec<String>) {
    let cmds = match group {
        "compute" => COMPUTE,
        _ => return,
    };
    for cmd in cmds {
        if matches_prefix(cmd, prefix) {
            out.push((*cmd).to_owned());
        }
    }
}
