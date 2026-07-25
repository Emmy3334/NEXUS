//! Brace `{ cmd… }` conditions must preserve empty argv words.

use nexus::env::ShellEnvironment;
use nexus::if_block;
use nexus::lex;
use nexus::repl;
use nexus::while_loop;
use std::io::Cursor;

fn eval_if_line(line: &str) -> Option<bool> {
    let mut tokens = Vec::new();
    lex::tokenize_into(line, &mut tokens).unwrap();
    let header = if_block::parse_header(line, &tokens).expect("header");
    let mut env = ShellEnvironment::default();
    let mut stdin = Cursor::new(Vec::new());
    let mut stderr = Vec::new();
    while_loop::eval_condition(&header.expr, &mut env, 0, &mut stdin, &mut stderr)
}

#[test]
fn brace_test_n_empty_is_false() {
    let mut env = ShellEnvironment::default();
    let mut stdin = Cursor::new(Vec::new());
    let mut stderr = Vec::new();
    let expr = vec![
        "{".into(),
        "test".into(),
        "-n".into(),
        "".into(),
        "}".into(),
    ];
    let got = while_loop::eval_condition(&expr, &mut env, 0, &mut stdin, &mut stderr);
    assert_eq!(
        got,
        Some(false),
        "stderr={}",
        String::from_utf8_lossy(&stderr)
    );
}

#[test]
fn brace_from_if_header_quoted_unset_is_false() {
    assert_eq!(
        eval_if_line(r#"if ( { test -n "$omn_load_all_libs" } ) then"#),
        Some(false)
    );
}

#[test]
fn brace_test_n_nonempty_is_true() {
    let mut env = ShellEnvironment::default();
    let mut stdin = Cursor::new(Vec::new());
    let mut stderr = Vec::new();
    let expr = vec![
        "{".into(),
        "test".into(),
        "-n".into(),
        "x".into(),
        "}".into(),
    ];
    let got = while_loop::eval_condition(&expr, &mut env, 0, &mut stdin, &mut stderr);
    assert_eq!(
        got,
        Some(true),
        "stderr={}",
        String::from_utf8_lossy(&stderr)
    );
}

#[test]
fn omn_safe_libs_path_when_load_all_unset() {
    let input = "\
if ( { test -n \"$omn_load_all_libs\" } ) then
echo LOAD_ALL
else
echo LOAD_SAFE
endif
";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(String::from_utf8_lossy(&stdout), "LOAD_SAFE\n");
}
