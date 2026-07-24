//! Cloud completion depth: nested actions, descriptions, resource ranking.

use nexus::env::ShellEnvironment;
use nexus::repl::{complete_matches_for_test, list_menu_lines_tagged, Match, Tag};

fn values(line: &str) -> Vec<String> {
    complete_matches_for_test(line, &ShellEnvironment::default())
        .into_iter()
        .map(|m| m.value)
        .collect()
}

#[test]
fn aws_s3_action_completes_ls_and_sync() {
    let matches = values("aws s3 ");
    assert!(matches.iter().any(|m| m == "ls"));
    assert!(matches.iter().any(|m| m == "sync"));
    let narrowed = values("aws s3 l");
    assert!(narrowed.iter().any(|m| m == "ls"));
}

#[test]
fn gcloud_compute_action_completes_firewall_rules() {
    let matches = values("gcloud compute ");
    assert!(matches.iter().any(|m| m == "firewall-rules"));
    let narrowed = values("gcloud compute fire");
    assert!(narrowed.iter().any(|m| m == "firewall-rules"));
}

#[test]
fn heal_help_has_description_in_menu() {
    let matches = complete_matches_for_test("heal ", &ShellEnvironment::default());
    let help = matches
        .iter()
        .find(|m| m.value == "help")
        .expect("help match");
    assert_eq!(
        help.description.as_deref(),
        Some("Show heal usage and tips")
    );
    let lines = list_menu_lines_tagged(&matches, 0);
    assert!(lines.iter().any(|l| l.contains("help  — Show heal usage")));
}

#[test]
fn resource_tag_sorts_before_files_on_score_tie() {
    let matches = vec![
        Match {
            value: "./file.txt".into(),
            description: None,
            tag: Tag::Files,
            score: 0,
        },
        Match {
            value: "my-pod".into(),
            description: Some("Running pod".into()),
            tag: Tag::Resources,
            score: 0,
        },
    ];
    let mut sorted = matches;
    sorted.sort_by(|a, b| (-a.score, a.tag, &a.value).cmp(&(-b.score, b.tag, &b.value)));
    assert_eq!(sorted[0].tag, Tag::Resources);
}
