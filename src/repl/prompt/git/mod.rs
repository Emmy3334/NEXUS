//! Read branch name and dirty state from the nearest `.git` directory.

mod dirty;
mod head;
mod root;

/// `branch` or `branch*` for the work tree containing cwd, if any.
#[must_use]
pub(super) fn segment() -> Option<String> {
    let git_dir = root::find_git_dir()?;
    let branch = head::branch_name(&git_dir)?;
    if dirty::is_dirty(&git_dir) {
        Some(format!("{branch}*"))
    } else {
        Some(branch)
    }
}
