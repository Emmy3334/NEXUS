//! Integration tests for `$` / `$?` word expansion at exec time.

use nexus::env::ShellEnvironment;
use nexus::exec::{execute_list, CommandResult};
use nexus::expand::expand_word_for_exec;
use nexus::lex::LexError;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn expand(raw: &str, env: &mut ShellEnvironment, status: u8) -> String {
    expand_word_for_exec(raw, env, status)
        .unwrap()
        .into_string()
}

fn env_with(pairs: &[(&str, &str)]) -> ShellEnvironment {
    let mut map = BTreeMap::new();
    for &(k, v) in pairs {
        map.insert(k.into(), v.into());
    }
    // Keep PATH so external helpers (`printf`, `cat`, …) resolve.
    if let Ok(path) = std::env::var("PATH") {
        map.entry("PATH".into()).or_insert(path);
    }
    ShellEnvironment::from_map(map)
}

fn parse_list(source: &str) -> nexus::parse::CommandList<'_> {
    let mut tokens = Vec::new();
    nexus::lex::tokenize_into(source, &mut tokens).expect("lex ok");
    nexus::parse::parse_line(source, &tokens)
        .expect("parse ok")
        .expect("non-empty list")
}

fn run(source: &str, env: &mut ShellEnvironment, last_status: u8) -> (CommandResult, String) {
    let list = parse_list(source);
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        env,
        last_status,
        Vec::new(),
        &mut std::io::empty(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    (result, String::from_utf8(stderr).unwrap())
}

fn temp_out(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_expand_{name}_{}", std::process::id()))
}

#[test]
fn expand_home_plain_and_braced() {
    let mut env = env_with(&[("HOME", "/tmp/home")]);
    assert_eq!(expand("$HOME", &mut env, 0), "/tmp/home");
    assert_eq!(expand("${HOME}", &mut env, 0), "/tmp/home");
}

#[test]
fn expand_status_question() {
    let mut env = ShellEnvironment::default();
    assert_eq!(expand("$?", &mut env, 42), "42");
    assert_eq!(expand("x$?y", &mut env, 7), "x7y");
}

#[test]
fn expand_status_name_like_question() {
    let mut env = ShellEnvironment::default();
    assert_eq!(expand("$status", &mut env, 3), "3");
    assert_eq!(expand("${status}", &mut env, 11), "11");
}

#[test]
fn expand_cwd_local() {
    let mut env = ShellEnvironment::default();
    env.set_local("cwd", "/tmp/nexus_cwd");
    assert_eq!(expand("$cwd", &mut env, 0), "/tmp/nexus_cwd");
}

#[test]
fn no_expand_inside_single_quotes() {
    let mut env = env_with(&[("HOME", "/tmp/home")]);
    assert_eq!(expand("'$HOME'", &mut env, 0), "$HOME");
    assert_eq!(expand("'$?'", &mut env, 3), "$?");
}

#[test]
fn expand_inside_double_quotes() {
    let mut env = env_with(&[("HOME", "/tmp/home")]);
    assert_eq!(expand("\"$HOME\"", &mut env, 0), "/tmp/home");
    assert_eq!(expand("\"status=$?\"", &mut env, 9), "status=9");
}

#[test]
fn unknown_var_expands_to_empty() {
    let mut env = ShellEnvironment::default();
    assert_eq!(expand("$NEXUS_NO_SUCH", &mut env, 0), "");
    assert_eq!(expand("${NEXUS_NO_SUCH}", &mut env, 0), "");
}

#[test]
fn escaped_dollar_stays_literal() {
    let mut env = env_with(&[("HOME", "/tmp")]);
    assert_eq!(expand(r"\$HOME", &mut env, 0), "$HOME");
    assert_eq!(expand(r#""\$HOME""#, &mut env, 0), "$HOME");
}

#[test]
fn unclosed_quote_still_errors() {
    let mut env = ShellEnvironment::default();
    assert_eq!(
        expand_word_for_exec("\"$HOME", &mut env, 0).unwrap_err(),
        LexError::UnclosedQuote
    );
}

#[test]
fn exec_printf_home_and_status_to_file() {
    let out = temp_out("home_status");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("HOME", "/tmp/nexus_home"), ("OUT", out_s)]);

    let (result, err) = run("printf '%s\\n' $HOME > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "/tmp/nexus_home\n");

    let (result, err) = run("false ; printf '%s\\n' $? > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "1\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn exec_no_expand_in_single_quotes() {
    let out = temp_out("sq");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("HOME", "/tmp/home"), ("OUT", out_s)]);
    let (result, err) = run("printf '%s\\n' '$HOME' > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "$HOME\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn exec_expand_in_double_quotes() {
    let out = temp_out("dq");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("HOME", "/tmp/home"), ("OUT", out_s)]);
    let (result, err) = run(r#"printf '%s\n' "$HOME" > $OUT"#, &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "/tmp/home\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn redirect_path_expands_var() {
    let path = temp_out("redir");
    let _ = fs::remove_file(&path);
    let mut env = env_with(&[("OUTFILE", path.to_str().expect("utf8 path"))]);
    let (result, err) = run("printf 'ok\\n' > $OUTFILE", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "ok\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn pipe_argv_expands_var() {
    let out = temp_out("pipe");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("MSG", "piped"), ("OUT", out_s)]);
    let (result, err) = run("printf '%s\\n' $MSG | cat > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "piped\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn local_shadows_exported_in_expansion() {
    let mut env = env_with(&[("FOO", "exported")]);
    env.set_local("FOO", "local");
    assert_eq!(expand("$FOO", &mut env, 0), "local");
}

#[test]
fn redirect_path_uses_braced_form() {
    let path = temp_out("braced");
    let _ = fs::remove_file(&path);
    let mut env = env_with(&[("OUTFILE", path.to_str().expect("utf8 path"))]);
    let (result, err) = run("printf 'x\\n' > ${OUTFILE}", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "x\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn backtick_unquoted_splits_on_whitespace() {
    let out = temp_out("bt_split");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("OUT", out_s)]);
    let (result, err) = run("printf '%s\\n' `printf 'a b c'` > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "a\nb\nc\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn backtick_single_quoted_is_literal() {
    let out = temp_out("bt_sq");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("OUT", out_s)]);
    let (result, err) = run("printf '%s\\n' '`echo hi`' > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "`echo hi`\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn backtick_double_quoted_keeps_blanks() {
    let out = temp_out("bt_dq");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("OUT", out_s)]);
    let (result, err) = run(r#"printf '%s\n' "`printf 'a b'`" > $OUT"#, &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "a b\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn backtick_in_redirect_path() {
    let path = temp_out("bt_redir");
    let _ = fs::remove_file(&path);
    let path_s = path.to_str().expect("utf8 path");
    let mut env = env_with(&[]);
    let cmd = format!("printf 'ok\\n' > `printf '{path_s}'`");
    let (result, err) = run(&cmd, &mut env, 0);
    assert_eq!(result, CommandResult::Status(0), "{err}");
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "ok\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn backtick_unclosed_errors() {
    let mut tokens = Vec::new();
    assert_eq!(
        nexus::lex::tokenize_into("echo `hi", &mut tokens),
        Err(LexError::UnclosedQuote)
    );
    assert_eq!(
        expand_word_for_exec("`hi", &mut ShellEnvironment::default(), 0).unwrap_err(),
        LexError::UnclosedQuote
    );
}

#[test]
fn backtick_heredoc_reads_body_from_stdin() {
    let mut env = env_with(&[]);
    let mut argv = Vec::new();
    let mut stderr = Vec::new();
    let mut stdin = std::io::Cursor::new("hello from heredoc\nEND\n");
    // Double-quoted so heredoc blanks are kept as one field.
    nexus::parse::fill_argv(
        &["\"`cat <<END`\""],
        &mut argv,
        &mut env,
        0,
        &mut stdin,
        &mut stderr,
    )
    .unwrap();
    assert!(stderr.is_empty(), "{:?}", String::from_utf8_lossy(&stderr));
    assert_eq!(argv, ["hello from heredoc"]);
}
