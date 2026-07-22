//! Apply group-level stdin/stdout redirects around a subshell list.

mod apply;

pub(super) use apply::apply_group_stdio;
