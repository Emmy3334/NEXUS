//! Tests for file and heredoc redirection apply.

use super::common::{parse_list, test_env};
use nexus::exec::{execute_list, CommandResult};
use std::fs;
use std::path::PathBuf;

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_redir_{}_{name}", std::process::id()))
}

#[test]
fn redirect_stdout_write_captures_env() {
    let path = temp_file("write.txt");
    let _ = fs::remove_file(&path);
    let source = format!("env > {}", path.display());
    let list = parse_list(&source);
    let mut env = test_env();
    env.set("NEXUS_REDIR", "yes");
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    let contents = fs::read_to_string(&path).expect("outfile");
    assert!(contents.contains("NEXUS_REDIR=yes"));
    let _ = fs::remove_file(&path);
}

#[test]
fn redirect_stdout_append() {
    let path = temp_file("append.txt");
    let _ = fs::remove_file(&path);
    fs::write(&path, "first\n").unwrap();
    let source = format!("env >> {}", path.display());
    let list = parse_list(&source);
    let mut env = test_env();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.starts_with("first\n"));
    assert!(contents.contains("PATH="));
    let _ = fs::remove_file(&path);
}

#[test]
fn redirect_stdin_from_file() {
    let path = temp_file("stdin.txt");
    fs::write(&path, "ignored-by-true\n").unwrap();
    let source = format!("true < {}", path.display());
    let list = parse_list(&source);
    let mut env = test_env();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    let _ = fs::remove_file(&path);
}

#[test]
fn redirect_missing_input_file_is_error() {
    let path = temp_file("missing_input.txt");
    let _ = fs::remove_file(&path);
    let source = format!("true < {}", path.display());
    let list = parse_list(&source);
    let mut env = test_env();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(1));
    assert!(!stderr.is_empty());
}

#[test]
fn redirect_overrides_pipe_stdout() {
    let path = temp_file("pipe_out.txt");
    let _ = fs::remove_file(&path);
    let source = format!("env > {} | true", path.display());
    let list = parse_list(&source);
    let mut env = test_env();
    env.set("NEXUS_PIPE_REDIR", "1");
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("NEXUS_PIPE_REDIR=1"));
    let _ = fs::remove_file(&path);
}

#[test]
fn heredoc_feeds_cat_stdin() {
    let path = temp_file("heredoc_out.txt");
    let _ = fs::remove_file(&path);
    let source = format!("cat << EOF > {}", path.display());
    let list = parse_list(&source);
    let bodies = vec!["line1\nline2\n".to_string()];
    let mut env = test_env();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        bodies,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    assert_eq!(fs::read_to_string(&path).unwrap(), "line1\nline2\n");
    assert!(stderr.is_empty());
    let _ = fs::remove_file(&path);
}

#[test]
fn heredoc_overrides_pipe_stdin() {
    let path = temp_file("heredoc_pipe.txt");
    let _ = fs::remove_file(&path);
    let source = format!("cat << EOF | cat > {}", path.display());
    let list = parse_list(&source);
    let bodies = vec!["piped-body\n".to_string()];
    let mut env = test_env();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        bodies,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    assert_eq!(fs::read_to_string(&path).unwrap(), "piped-body\n");
    let _ = fs::remove_file(&path);
}
