//! Completion engine: collect → approximate → history frecency → expand.

mod expand;

use super::annotate;
use super::collect::collect_matches;
use super::history;
use super::match_item::{Match, Tag};
use super::matchers::{approx_matches, prefix_score};
use crate::env::CompRegistry;
use crate::history::History;
use expand::expand;

/// Run the completion chain for the current token context.
pub fn run(
    before: &str,
    prefix: &str,
    var_names: &[String],
    registry: &CompRegistry,
    path: &str,
    hist: Option<&History>,
) -> Vec<Match> {
    let raw = collect_matches(before, prefix, var_names, registry, path);
    let mut matches = raw
        .iter()
        .map(|v| to_match(v, prefix_score(v, prefix), infer_tag(v)))
        .collect::<Vec<_>>();
    if matches.is_empty() && !prefix.is_empty() {
        matches = approx_phase(before, prefix, var_names, registry, path);
    }
    if let Some(h) = hist {
        merge_history(&mut matches, h, prefix);
    }
    annotate::enrich(&mut matches, before, registry);
    sort_dedup(&mut matches);
    expand(before, matches)
}

fn approx_phase(
    before: &str,
    prefix: &str,
    var_names: &[String],
    registry: &CompRegistry,
    path: &str,
) -> Vec<Match> {
    let all = collect_matches(before, "", var_names, registry, path);
    approx_matches(&all, prefix)
        .into_iter()
        .map(|(v, d)| to_match(&v, -(d as i32), infer_tag(&v)))
        .collect()
}

fn merge_history(matches: &mut Vec<Match>, hist: &History, prefix: &str) {
    let mut scored = Vec::new();
    history::collect(hist, prefix, &mut scored);
    for (word, boost) in scored {
        if let Some(m) = matches.iter_mut().find(|m| m.value == word) {
            m.score = m.score.saturating_add(boost);
        } else {
            matches.push(to_match(
                &word,
                history::history_score(&word, prefix, boost),
                Tag::History,
            ));
        }
    }
}

fn to_match(value: &str, score: i32, tag: Tag) -> Match {
    Match {
        value: value.to_owned(),
        description: None,
        tag,
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
    // Tag-major order keeps menu sections contiguous; score ranks within a tag.
    matches.sort_by(|a, b| (a.tag, -a.score, &a.value).cmp(&(b.tag, -b.score, &b.value)));
    matches.dedup_by(|a, b| a.value == b.value);
}
