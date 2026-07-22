//! foreach / end control structure.

use nexus::foreach::{self, parse_header};
use nexus::lex;
use nexus::repl;
use std::io::Cursor;

#[test]
fn parse_foreach_header_tokens() {
    let src = "foreach i (foo bar team plio)";
    let mut tokens = Vec::new();
    lex::tokenize_into(src, &mut tokens).unwrap();
    let kinds: Vec<_> = tokens
        .iter()
        .map(|t| format!("{:?}:{}", t.kind, t.lexeme(src)))
        .collect();
    assert!(parse_header(src, &tokens).is_some(), "tokens={kinds:?}");
    let h = parse_header(src, &tokens).unwrap();
    assert_eq!(h.var, "i");
    assert_eq!(h.items, vec!["foo", "bar", "team", "plio"]);
}

#[test]
fn foreach_runs_body_for_each_word() {
    let out = std::env::temp_dir().join(format!("nexus_foreach_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "foreach i (foo bar)\nprintf '%s\\n' \"$i\" >> {}\nend\n",
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "foo\nbar\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn is_end_line_trims() {
    assert!(foreach::is_end_line("end"));
    assert!(foreach::is_end_line("  end\t"));
    assert!(!foreach::is_end_line("ending"));
}

#[test]
fn bare_foreach_is_syntax_error_not_command_not_found() {
    let mut stdin = Cursor::new("foreach\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 1);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Words not parenthesized"), "{err}");
    assert!(!err.contains("Command not found"));
}

#[test]
fn stray_end_is_not_command_not_found() {
    let mut stdin = Cursor::new("end\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 1);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Not in while/foreach"), "{err}");
    assert!(!err.contains("Command not found"));
}
