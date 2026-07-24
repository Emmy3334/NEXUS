//! Tagged completion menu lines with section headers.

use super::list::LIST_MAX;
use super::match_item::{Match, Tag};

/// Menu lines grouped by [`Tag`] with section headers and counts.
pub fn list_menu_lines_tagged(matches: &[Match], highlight: usize) -> Vec<String> {
    let (shown, omitted) = truncate_matches(matches);
    let ordered = group_by_tag(shown);
    let hl = matches.get(highlight).map(|m| m.value.as_str());
    let mut lines = Vec::new();
    let mut i = 0;
    while i < ordered.len() {
        let tag = ordered[i].tag;
        let end = next_tag_break(&ordered, i, tag);
        lines.push(format!("-- {} ({}) --", tag_label(tag), end - i));
        while i < end {
            let on = hl == Some(ordered[i].value.as_str());
            lines.push(highlight_line(&format_match(ordered[i]), on));
            i += 1;
        }
    }
    if omitted > 0 {
        lines.push(format!("... and {omitted} more"));
    }
    lines
}

fn group_by_tag(matches: &[Match]) -> Vec<&Match> {
    let mut tags: Vec<Tag> = matches.iter().map(|m| m.tag).collect();
    tags.sort();
    tags.dedup();
    let mut out = Vec::with_capacity(matches.len());
    for tag in tags {
        for m in matches {
            if m.tag == tag {
                out.push(m);
            }
        }
    }
    out
}

fn next_tag_break(ordered: &[&Match], start: usize, tag: Tag) -> usize {
    ordered[start..]
        .iter()
        .position(|m| m.tag != tag)
        .map_or(ordered.len(), |n| start + n)
}

fn format_match(m: &Match) -> String {
    match &m.description {
        Some(d) => format!("{}  — {d}", m.value),
        None => m.value.clone(),
    }
}

fn tag_label(tag: Tag) -> &'static str {
    match tag {
        Tag::Commands => "commands",
        Tag::History => "history",
        Tag::Flags => "flags",
        Tag::Resources => "resources",
        Tag::Files => "files",
        Tag::Vars => "vars",
        Tag::Other => "other",
    }
}

fn highlight_line(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[7m{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

fn truncate_matches(matches: &[Match]) -> (&[Match], usize) {
    if matches.len() > LIST_MAX {
        (&matches[..LIST_MAX], matches.len() - LIST_MAX)
    } else {
        (matches, 0)
    }
}
