//! Unary `[[` tests (`-f`, `-d`, `-e`, `-z`, `-n`, …).

use std::path::Path;

pub(super) fn eval(op: &str, arg: &str) -> Result<bool, &'static str> {
    match op {
        "-z" => Ok(arg.is_empty()),
        "-n" => Ok(!arg.is_empty()),
        "-e" => Ok(Path::new(arg).exists()),
        "-f" => Ok(Path::new(arg).is_file()),
        "-d" => Ok(Path::new(arg).is_dir()),
        "-r" => Ok(access(arg, 0o444)),
        "-w" => Ok(access(arg, 0o222)),
        "-x" => Ok(access(arg, 0o111)),
        _ => Err("unknown unary operator"),
    }
}

fn access(path: &str, mask: u32) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & mask != 0
    }
    #[cfg(not(unix))]
    {
        let _ = mask;
        metadata.is_file() || metadata.is_dir()
    }
}
