//! Completion engine: fuzzy prefix, substring, and approximate matching.

use nexus::env::ShellEnvironment;
use nexus::repl::{complete, list_menu_lines_tagged, Match, Tag};

fn complete_at(line: &str) -> (String, Vec<String>) {
    let mut buffer = line.to_owned();
    let mut cursor = buffer.len();
    let matches = complete(&mut buffer, &mut cursor, &ShellEnvironment::default());
    (buffer, matches)
}

#[test]
fn case_insensitive_subcommand_prefix() {
    let (buf, matches) = complete_at("git Checko");
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout");
}

#[test]
fn substring_matches_when_prefix_len_at_least_two() {
    let (buf, matches) = complete_at("git eckout");
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout");
}

#[test]
fn approximate_match_when_exact_empty() {
    let (buf, matches) = complete_at("git staus");
    assert!(matches.is_empty());
    assert_eq!(buf, "git status");
}

#[test]
fn approximate_typo_checkout() {
    let (buf, matches) = complete_at("git chekout");
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout");
}

#[test]
fn tagged_menu_groups_mixed_categories() {
    let matches = vec![
        Match {
            value: "status".into(),
            description: None,
            tag: Tag::Commands,
            score: 100,
        },
        Match {
            value: "--oneline".into(),
            description: None,
            tag: Tag::Flags,
            score: 90,
        },
        Match {
            value: "stash".into(),
            description: None,
            tag: Tag::Commands,
            score: 80,
        },
    ];
    let lines = list_menu_lines_tagged(&matches, 1);
    assert!(lines.iter().any(|l| l == "-- Commands --"));
    assert!(lines.iter().any(|l| l == "-- Flags --"));
    assert!(lines[0].contains("Commands"));
    assert!(lines.iter().any(|l| l.contains("--oneline")));
}
