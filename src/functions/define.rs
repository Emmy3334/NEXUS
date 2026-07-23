//! Function definition header kinds.

/// A recognized function definition on the current line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionHeader {
    /// `name() { … }` fully closed on this line.
    Complete { name: String, body: String },
    /// `name() {` — body continues on following lines.
    Open { name: String, first: String },
}
