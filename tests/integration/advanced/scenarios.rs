//! Combinatorial scripts: vars + globs + pipes + redirects + builtins.

use super::{lock_cwd, run_script, scratch};
use std::fs;

#[test]
fn set_expand_unset_roundtrip() {
    let dir = scratch("set");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("out.txt");
    let out2 = dir.join("out2.txt");
    let script = format!(
        "set MSG=hello_world\n\
         printf '%s\\n' $MSG > {}\n\
         unset MSG\n\
         printf '%s\\n' x$MSG > {}\n",
        out.display(),
        out2.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "hello_world\n");
    assert_eq!(fs::read_to_string(&out2).unwrap(), "x\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn status_and_question_track_last_command() {
    let dir = scratch("status");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("s.txt");
    let script = format!(
        "false\n\
         printf '%s\\n' $? > {}\n\
         true\n\
         printf '%s\\n' $status >> {}\n\
         false ; printf '%s\\n' $? >> {}\n",
        out.display(),
        out.display(),
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "1\n0\n1\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn quotes_block_dollar_and_glob() {
    let dir = scratch("quotes");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.txt"), "").unwrap();
    let out = dir.join("o.txt");
    let _guard = lock_cwd();
    let start = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let script = format!(
        "set HOME=/tmp/home\n\
         printf '%s\\n' '$HOME' > {}\n\
         printf '%s\\n' '*' >> {}\n\
         printf '%s\\n' \"*\" >> {}\n",
        out.display(),
        out.display(),
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "$HOME\n*\n*\n");

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn glob_pipe_redirect_chain() {
    let _guard = lock_cwd();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("chain");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("b.dat"), "").unwrap();
    fs::write(dir.join("a.dat"), "").unwrap();
    fs::write(dir.join(".skip"), "").unwrap();
    let out = dir.join("names.txt");
    std::env::set_current_dir(&dir).unwrap();

    let (code, _, err) = run_script("printf '%s\\n' *.dat | cat > names.txt\n");
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "a.dat\nb.dat\n");

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn braced_var_in_redirect_and_argv() {
    let dir = scratch("braced");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("file.txt");
    let script = format!(
        "set OUT={}\n\
         set WORD=nexus\n\
         printf '%s\\n' ${{WORD}} > ${{OUT}}\n",
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "nexus\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn local_not_exported_to_env_builtin() {
    let (code, out, err) = run_script("set SECRET=nope\nsetenv VISIBLE yes\nenv\n");
    assert_eq!(code, 0, "err={err}");
    assert!(out.contains("VISIBLE=yes"));
    assert!(!out.contains("SECRET=nope"));
}

#[test]
fn semicolon_list_with_set_and_status() {
    let dir = scratch("list");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "set A=1 ; set B=2 ; false ; printf '%s\\n' $A$B$? > {}\n",
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "121\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ambiguous_glob_redirect_fails_cleanly() {
    let _guard = lock_cwd();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("ambig");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("1.txt"), "").unwrap();
    fs::write(dir.join("2.txt"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let (code, _, err) = run_script("printf x > *.txt\n");
    assert_eq!(code, 1);
    assert!(err.contains("Ambiguous redirect"));

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn multipipe_and_missing_left_stage() {
    let (code, _, err) = run_script("true | true | false\n");
    assert_eq!(code, 1, "err={err}");

    let (code, _, err) = run_script("nexus_missing_left_zzz | true\n");
    assert_eq!(code, 0, "err={err}");
    assert!(err.contains("Command not found"));
}

#[test]
fn cd_updates_cwd_local() {
    let _guard = lock_cwd();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("cwd");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("cwd.txt");
    let target = dir.canonicalize().unwrap();

    let script = format!(
        "cd {}\n\
         printf '%s\\n' $cwd > {}\n",
        target.display(),
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        fs::read_to_string(&out).unwrap().trim_end(),
        target.to_str().unwrap()
    );

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn escaped_dollar_and_escaped_glob() {
    let dir = scratch("esc");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "set HOME=/tmp\n\
         printf '%s\\n' \\$HOME > {}\n\
         printf '%s\\n' \\* >> {}\n",
        out.display(),
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "$HOME\n*\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bracket_glob_and_question() {
    let _guard = lock_cwd();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("class");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a"), "").unwrap();
    fs::write(dir.join("b"), "").unwrap();
    fs::write(dir.join("c"), "").unwrap();
    fs::write(dir.join("ab"), "").unwrap();
    let out = dir.join("o.txt");
    std::env::set_current_dir(&dir).unwrap();

    let (code, _, err) = run_script("printf '%s\\n' [ab] > o.txt\nprintf '%s\\n' ? >> o.txt\n");
    assert_eq!(code, 0, "err={err}");
    let text = fs::read_to_string(&out).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert!(lines.contains(&"a"));
    assert!(lines.contains(&"b"));
    assert!(lines.contains(&"c"));
    assert!(!lines.contains(&"ab"));

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn heredoc_redirect_to_file() {
    let dir = scratch("heredoc");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("h.txt");
    let script = format!("cat << END > {}\nhello\nworld\nEND\n", out.display());
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "hello\nworld\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn exit_in_pipe_does_not_kill_shell_then_status() {
    let dir = scratch("exitpipe");
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("o.txt");
    let script = format!(
        "exit 9 | true\n\
         printf '%s\\n' $? > {}\n\
         exit 3\n\
         printf '%s\\n' should_not_run > {}\n",
        out.display(),
        out.display()
    );
    let (code, _, err) = run_script(&script);
    assert_eq!(code, 3, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "0\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unquoted_var_value_can_glob() {
    let _guard = lock_cwd();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("varglob");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("x1"), "").unwrap();
    fs::write(dir.join("x2"), "").unwrap();
    let out = dir.join("o.txt");
    std::env::set_current_dir(&dir).unwrap();

    let (code, _, err) = run_script("set PAT=x?\nprintf '%s\\n' $PAT > o.txt\n");
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "x1\nx2\n");

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn double_quoted_var_does_not_glob() {
    let _guard = lock_cwd();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("dqglob");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("x1"), "").unwrap();
    let out = dir.join("o.txt");
    std::env::set_current_dir(&dir).unwrap();

    let (code, _, err) = run_script("set PAT=x?\nprintf '%s\\n' \"$PAT\" > o.txt\n");
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "x?\n");

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}
