//! Shell functions: define, call, positionals, and `return`.

use nexus::env::ShellEnvironment;
use nexus::repl;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn test_env() -> ShellEnvironment {
    let path = std::env::var("PATH").unwrap_or_default();
    let mut map = BTreeMap::new();
    map.insert("PATH".into(), path);
    ShellEnvironment::from_map(map)
}

fn run(script: &str) -> (u8, String) {
    let mut env = test_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run_with_env(
        Cursor::new(script),
        &mut stdout,
        &mut stderr,
        false,
        &mut env,
    )
    .unwrap();
    (code, String::from_utf8(stderr).unwrap())
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_fn_{name}_{}", std::process::id()))
}

#[test]
fn define_and_call_one_line() {
    let path = scratch("one");
    let _ = fs::remove_file(&path);
    let script = format!(
        "greet() {{ printf '%s\\n' hi; }}\ngreet > {}\n",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "hi\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn function_keyword_form() {
    let path = scratch("kw");
    let _ = fs::remove_file(&path);
    let script = format!(
        "function greet {{ printf '%s\\n' yo; }}\ngreet > {}\n",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "yo\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn multiline_body_and_positionals() {
    let path = scratch("multi");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
greet() {{
printf '%s\\n' $1
printf '%s\\n' $2
}}
greet a b > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "a\nb\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn return_stops_body_with_status() {
    let path = scratch("ret");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
f() {{
printf '%s\\n' one
return 7
printf '%s\\n' two
}}
f > {}
printf '%s\\n' done >> {}
",
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "one\ndone\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn return_outside_function_errors() {
    let (code, err) = run("return 1\n");
    assert_eq!(code, 1);
    assert!(err.contains("not in a function"), "{err}");
}

#[test]
fn argc_inside_function() {
    let path = scratch("argc");
    let _ = fs::remove_file(&path);
    let script = format!(
        "count() {{ printf '%s\\n' $#; }}\ncount a b c > {}\n",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "3\n");
    let _ = fs::remove_file(&path);
}
