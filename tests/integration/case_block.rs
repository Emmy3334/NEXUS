//! `case` / `esac` control structure.

use nexus::case_block::{self, parse_header};
use nexus::lex;
use nexus::repl;
use std::io::Cursor;

#[test]
fn parse_case_in_header() {
    let src = "case $x in";
    let mut tokens = Vec::new();
    lex::tokenize_into(src, &mut tokens).unwrap();
    let h = parse_header(src, &tokens).expect("header");
    assert_eq!(h.subject, "$x");
}

#[test]
fn case_matches_literal_and_star() {
    let out = std::env::temp_dir().join(format!("nexus_case_lit_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "\
case hello in
hello)
printf 'hit\\n' >> {0}
;;
*)
printf 'miss\\n' >> {0}
;;
esac
",
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "hit\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn case_alternation_and_default() {
    let out = std::env::temp_dir().join(format!("nexus_case_alt_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "\
case zed in
a|b)
printf 'ab\\n' >> {0}
;;
*)
printf 'def\\n' >> {0}
;;
esac
",
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "def\n");
    let _ = std::fs::remove_file(out);
}

#[test]
fn case_glob_question() {
    assert!(case_block::matches_any("ab", "a?"));
    assert!(!case_block::matches_any("abc", "a?"));
    assert!(case_block::matches_any("hello", "h*o"));
}

#[test]
fn stray_esac_errors() {
    let mut stdin = Cursor::new("esac\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 1);
    assert!(
        String::from_utf8_lossy(&stderr).contains("Not in case"),
        "{}",
        String::from_utf8_lossy(&stderr)
    );
}

#[test]
fn bare_case_is_syntax_error() {
    let mut stdin = Cursor::new("case\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 1);
    assert!(
        String::from_utf8_lossy(&stderr).contains("Syntax Error"),
        "{}",
        String::from_utf8_lossy(&stderr)
    );
}
