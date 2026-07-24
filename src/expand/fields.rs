//! Multi-field builder for expansions that can split words (backticks / arrays).

use super::ExpandedWord;

/// Accumulates one or more expanded fields from a single raw word.
pub(super) struct FieldBuilder {
    fields: Vec<ExpandedWord>,
    /// When set, a sole empty field is dropped in [`Self::into_fields`] (empty `${arr[@]}`).
    elide_empty: bool,
}

impl FieldBuilder {
    pub(super) fn new() -> Self {
        Self {
            fields: vec![ExpandedWord::default()],
            elide_empty: false,
        }
    }

    pub(super) fn current(&mut self) -> &mut ExpandedWord {
        self.fields.last_mut().expect("field builder non-empty")
    }

    pub(super) fn start_field(&mut self) {
        self.elide_empty = false;
        self.fields.push(ExpandedWord::default());
    }

    /// Mark that an empty `${name[@]}` contributed nothing; drop a sole empty field later.
    pub(super) fn mark_empty_at(&mut self) {
        if self.fields.len() == 1 && self.current().as_str().is_empty() {
            self.elide_empty = true;
        }
    }

    /// Clear empty-`@` elision after real content is written.
    pub(super) fn clear_elide(&mut self) {
        self.elide_empty = false;
    }

    pub(super) fn into_fields(self) -> Vec<ExpandedWord> {
        if self.elide_empty && self.fields.len() == 1 && self.fields[0].as_str().is_empty() {
            return Vec::new();
        }
        self.fields
    }
}
