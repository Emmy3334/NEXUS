//! Read branch name and dirty state from the nearest `.git` directory.

mod dirty;
mod head;
mod root;

/// Structured git work-tree info for themed prompts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitInfo {
    pub branch: String,
    pub dirty: bool,
}

/// Branch / dirty snapshot for cwd, if inside a work tree.
#[must_use]
pub fn info() -> Option<GitInfo> {
    let git_dir = root::find_git_dir()?;
    let branch = head::branch_name(&git_dir)?;
    Some(GitInfo {
        dirty: dirty::is_dirty(&git_dir),
        branch,
    })
}

/// `branch` or `branch*` for the work tree containing cwd, if any.
#[must_use]
pub(super) fn segment() -> Option<String> {
    let info = info()?;
    if info.dirty {
        Some(format!("{}*", info.branch))
    } else {
        Some(info.branch)
    }
}
