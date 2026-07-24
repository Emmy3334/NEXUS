//! Tagged completion menu lines with section headers.

use super::list::LIST_MAX;
use super::match_item::{Match, Tag};

/// Menu lines grouped by [`Tag`] with section headers.
pub fn list_menu_lines_tagged(matches: &[Match], highlight: usize) -> Vec<String> {
    let (shown, omitted) = truncate_matches(matches);
    let mut lines = Vec::new();
    let mut last_tag: Option<Tag> = None;
    for (i, m) in shown.iter().enumerate() {
        if last_tag != Some(m.tag) {
            lines.push(format!("-- {} --", tag_label(m.tag)));
            last_tag = Some(m.tag);
        }
        lines.push(highlight_line(&format_match(m), i == highlight));
    }
    if omitted > 0 {
        lines.push(format!("... and {omitted} more"));
    }
    lines
}

fn format_match(m: &Match) -> String {
    match &m.description {
        Some(d) => format!("{}  — {d}", m.value),
        None => m.value.clone(),
    }
}

fn tag_label(tag: Tag) -> &'static str {
    match tag {
        Tag::Commands => "Commands",
        Tag::Flags => "Flags",
        Tag::Resources => "Resources",
        Tag::Files => "Files",
        Tag::Vars => "Vars",
        Tag::Other => "Other",
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
