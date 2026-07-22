//! if / then / else / endif control structure.

use nexus::if_block::{self, parse_header};
use nexus::lex;
use nexus::repl;
use std::io::Cursor;

#[test]
fn parse_if_then_header() {
    let src = "if ($i < 3) then";
    let mut tokens = Vec::new();
    lex::tokenize_into(src, &mut tokens).unwrap();
    let h = parse_header(src, &tokens).expect("header");
    assert_eq!(h.expr, vec!["$i", "<", "3"]);
}

#[test]
fn if_then_branch_runs() {
    let out = std::env::temp_dir().join(format!("nexus_if_then_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "if (1) then\nprintf 'yes\\n' >> {}\nelse\nprintf 'no\\n' >> {}\nendif\n",
        out.display(),
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "yes\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn if_else_branch_runs() {
    let out = std::env::temp_dir().join(format!("nexus_if_else_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "if (0) then\nprintf 'yes\\n' >> {}\nelse\nprintf 'no\\n' >> {}\nendif\n",
        out.display(),
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "no\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn else_if_branch_runs() {
    let out = std::env::temp_dir().join(format!("nexus_elseif_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "if (0) then\nprintf 'a\\n' >> {}\nelse if (1) then\nprintf 'b\\n' >> {}\nelse\nprintf 'c\\n' >> {}\nendif\n",
        out.display(),
        out.display(),
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "b\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn bare_if_is_syntax_error() {
    let mut stdin = Cursor::new("if\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 1);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Expression Syntax"), "{err}");
    assert!(!err.contains("Command not found"));
}

#[test]
fn stray_endif_is_not_command_not_found() {
    let mut stdin = Cursor::new("endif\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 1);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Not in if"), "{err}");
    assert!(!err.contains("Command not found"));
}

#[test]
fn is_endif_trims() {
    assert!(if_block::is_endif_line("endif"));
    assert!(if_block::is_endif_line("  endif\t"));
    assert!(!if_block::is_endif_line("endifx"));
}
