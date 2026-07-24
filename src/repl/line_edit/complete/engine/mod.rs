//! Completion engine: collect → approximate → expand (stub).

mod expand;

use super::annotate;
use super::collect::collect_matches;
use super::match_item::{Match, Tag};
use super::matchers::{approx_matches, prefix_score};
use expand::expand;

/// Run the completion chain for the current token context.
pub fn run(
    before: &str,
    prefix: &str,
    var_names: &[String],
    registry: &crate::env::CompRegistry,
) -> Vec<Match> {
    let raw = collect_matches(before, prefix, var_names, registry);
    let mut matches = raw
        .iter()
        .map(|v| to_match(v, prefix_score(v, prefix)))
        .collect::<Vec<_>>();
    if matches.is_empty() && !prefix.is_empty() {
        matches = approx_phase(before, prefix, var_names, registry);
    }
    annotate::enrich(&mut matches, before, registry);
    sort_dedup(&mut matches);
    expand(before, matches)
}

fn approx_phase(
    before: &str,
    prefix: &str,
    var_names: &[String],
    registry: &crate::env::CompRegistry,
) -> Vec<Match> {
    let all = collect_matches(before, "", var_names, registry);
    approx_matches(&all, prefix)
        .into_iter()
        .map(|(v, d)| to_match(&v, -(d as i32)))
        .collect()
}

fn to_match(value: &str, score: i32) -> Match {
    Match {
        value: value.to_owned(),
        description: None,
        tag: infer_tag(value),
        score,
    }
}

fn infer_tag(value: &str) -> Tag {
    if value.starts_with('-') {
        Tag::Flags
    } else if value.starts_with('$') {
        Tag::Vars
    } else if value.contains('/') || value.ends_with('/') {
        Tag::Files
    } else {
        Tag::Commands
    }
}

fn sort_dedup(matches: &mut Vec<Match>) {
    matches.sort_by(|a, b| (-a.score, a.tag, &a.value).cmp(&(-b.score, b.tag, &b.value)));
    matches.dedup_by(|a, b| a.value == b.value);
}
