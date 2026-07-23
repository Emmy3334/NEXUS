//! Shell functions: `name() { … }` / `function name { … }`.

pub(crate) mod body;
mod call;
mod define;
mod header;

pub use call::try_run;
pub use define::FunctionHeader;
pub use header::parse_header;
