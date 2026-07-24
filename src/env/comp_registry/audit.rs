//! Refuse world-writable completion dump paths (Unix group/other write).

use std::io;
use std::path::Path;

/// True when `path` is safe to read (not group/world-writable on Unix).
pub fn is_safe(path: &Path) -> io::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)?.permissions().mode();
        Ok(mode & 0o002 == 0)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(true)
    }
}
