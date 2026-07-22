//! Detect whether quotes / backticks are still open.

use crate::lex;

/// True when `source` has no unmatched `'`, `"`, or `` ` ``.
#[must_use]
pub(super) fn quotes_closed(source: &str) -> bool {
    let mut tokens = Vec::new();
    !matches!(
        lex::tokenize_into(source, &mut tokens),
        Err(lex::LexError::UnclosedQuote)
    )
}
