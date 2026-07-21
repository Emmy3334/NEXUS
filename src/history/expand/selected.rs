//! Selected words from an event (verbatim or split).

use super::scan::{join_words, split_words};

#[derive(Debug, Clone)]
pub(super) enum Selected {
    Verbatim(String),
    Words(Vec<String>),
}

impl Selected {
    pub(super) fn into_words(self) -> Vec<String> {
        match self {
            Self::Verbatim(s) => split_words(&s),
            Self::Words(w) => w,
        }
    }

    pub(super) fn render(self) -> String {
        match self {
            Self::Verbatim(s) => s,
            Self::Words(w) => join_words(&w),
        }
    }
}
