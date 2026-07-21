//! Multi-field builder for expansions that can split words (backticks).

use super::ExpandedWord;

/// Accumulates one or more expanded fields from a single raw word.
pub(super) struct FieldBuilder {
    fields: Vec<ExpandedWord>,
}

impl FieldBuilder {
    pub(super) fn new() -> Self {
        Self {
            fields: vec![ExpandedWord::default()],
        }
    }

    pub(super) fn current(&mut self) -> &mut ExpandedWord {
        self.fields.last_mut().expect("field builder non-empty")
    }

    pub(super) fn start_field(&mut self) {
        self.fields.push(ExpandedWord::default());
    }

    pub(super) fn into_fields(self) -> Vec<ExpandedWord> {
        self.fields
    }
}
