//! `dirs -S` / `dirs -L` histfile-style stack persistence.

use super::pushd::push_path;
use crate::builtins::cd;
use crate::env::ShellEnvironment;

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub(super) fn save_stack(
    shell_env: &ShellEnvironment,
    path: Option<&Path>,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let resolved = resolve_dirsfile(shell_env, path);
    let entries: Vec<&PathBuf> = shell_env.dir_stack.iter().collect();
    if entries.is_empty() {
        return write_file(&resolved, "", stderr);
    }
    let mut body = String::new();
    let bottom = entries[entries.len() - 1];
    body.push_str(&format!("cd {}\n", bottom.display()));
    for path in entries.iter().rev().skip(1) {
        body.push_str(&format!("pushd {}\n", path.display()));
    }
    write_file(&resolved, &body, stderr)
}

pub(super) fn load_stack(
    shell_env: &mut ShellEnvironment,
    path: Option<&Path>,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let resolved = resolve_dirsfile(shell_env, path);
    let text = match fs::read_to_string(&resolved) {
        Ok(text) => text,
        Err(err) => {
            writeln!(stderr, "dirs: {}: {err}.", resolved.display())?;
            return Ok(1);
        }
    };
    apply_dirs_script(&text, shell_env, last_status, stdout, stderr)
}

fn apply_dirs_script(
    text: &str,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let code = apply_line(line, shell_env, last_status, stdout, stderr)?;
        if code != 0 {
            return Ok(code);
        }
    }
    Ok(0)
}

fn apply_line(
    line: &str,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if let Some(path) = line.strip_prefix("cd ") {
        return cd::change_directory(
            Path::new(path.trim()),
            shell_env,
            last_status,
            stdout,
            stderr,
        );
    }
    if let Some(path) = line.strip_prefix("pushd ") {
        return push_path(
            PathBuf::from(path.trim()),
            shell_env,
            last_status,
            stdout,
            stderr,
        );
    }
    writeln!(stderr, "dirs: Badly formed line.")?;
    Ok(1)
}

fn write_file(path: &Path, body: &str, stderr: &mut impl Write) -> io::Result<u8> {
    match fs::write(path, body) {
        Ok(()) => Ok(0),
        Err(err) => {
            writeln!(stderr, "dirs: {}: {err}.", path.display())?;
            Ok(1)
        }
    }
}

fn resolve_dirsfile(shell_env: &ShellEnvironment, path: Option<&Path>) -> PathBuf {
    if let Some(p) = path {
        return p.to_path_buf();
    }
    if let Some(df) = shell_env.get_local("dirsfile") {
        return PathBuf::from(df);
    }
    let home = shell_env
        .lookup("home")
        .or_else(|| shell_env.lookup("HOME"))
        .unwrap_or(".");
    PathBuf::from(home).join(".nexus_dirs")
}
