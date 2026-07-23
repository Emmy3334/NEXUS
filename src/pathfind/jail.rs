//! Drop relative / empty `PATH` components (cwd hijack defense).

/// Keep only absolute directory components from a `PATH` string.
#[must_use]
pub fn sanitize_path(path_var: &str) -> String {
    path_var
        .split(':')
        .filter(|dir| is_absolute_component(dir))
        .collect::<Vec<_>>()
        .join(":")
}

#[must_use]
fn is_absolute_component(dir: &str) -> bool {
    !dir.is_empty() && dir.starts_with('/')
}
