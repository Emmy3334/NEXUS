//! Rich completion match model (value, tag, score).

/// One completion candidate with metadata for ranking and display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub value: String,
    pub description: Option<String>,
    pub tag: Tag,
    pub score: i32,
}

/// Category for menu grouping and sort order (lower sorts first on score ties).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tag {
    Resources,
    Flags,
    Commands,
    History,
    Files,
    Vars,
    Other,
}

/// Extract insertable values from ranked matches.
pub fn values(matches: &[Match]) -> Vec<String> {
    matches.iter().map(|m| m.value.clone()).collect()
}
