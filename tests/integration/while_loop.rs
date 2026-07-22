//! while / end control structure and `@` counters.

use nexus::lex;
use nexus::repl;
use nexus::while_loop::parse_header;
use std::io::Cursor;

#[test]
fn parse_while_header_comparison() {
    let src = "while ($i < 3)";
    let mut tokens = Vec::new();
    lex::tokenize_into(src, &mut tokens).unwrap();
    let h = parse_header(src, &tokens).expect("header");
    assert_eq!(h.expr, vec!["$i", "<", "3"]);
}

#[test]
fn while_runs_until_condition_false() {
    let out = std::env::temp_dir().join(format!("nexus_while_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "set i=0\nwhile ($i < 3)\nprintf '%s\\n' \"$i\" >> {}\n@ i++\nend\n",
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "0\n1\n2\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn while_zero_skips_body() {
    let mut stdin = Cursor::new("while (0)\necho no\nend\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert!(
        stdout.is_empty(),
        "stdout={}",
        String::from_utf8_lossy(&stdout)
    );
}

#[test]
fn bare_while_is_syntax_error() {
    let mut stdin = Cursor::new("while\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 1);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Expression Syntax"), "{err}");
    assert!(!err.contains("Command not found"));
}

#[test]
fn at_plusplus_updates_local() {
    let out = std::env::temp_dir().join(format!("nexus_at_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "set n=1\n@ n++\nprintf '%s\\n' \"$n\" > {}\n",
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "2\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn while_not_and_or_ops() {
    let out = std::env::temp_dir().join(format!("nexus_while_logic_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "set i=0\nwhile ( ! 0 && $i < 2 || 0 )\nprintf 'x\\n' >> {}\n@ i++\nend\n",
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "x\nx\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn while_brace_command_status() {
    let out = std::env::temp_dir().join(format!("nexus_while_brace_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "set n=0\nwhile ( {{ true }} && $n < 1 )\nprintf 'ok\\n' >> {}\n@ n++\nend\n",
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "ok\n");
    let _ = std::fs::remove_file(out);
}
