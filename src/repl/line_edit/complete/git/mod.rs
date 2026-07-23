//! Git-aware Tab completion (local branches).

mod branches;

pub(super) fn collect_branches(prefix: &str, out: &mut Vec<String>) {
    branches::collect(prefix, out);
}
