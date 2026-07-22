//! Scripting: file execution, positionals, and `source`.

use nexus::env::ShellEnvironment;
use nexus::expand::expand_word_for_exec;
use nexus::repl;
use std::io::Cursor;
use std::process::Command;

fn write_temp_script(name: &str, body: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("nexus_script_{name}_{}.sh", std::process::id()));
    std::fs::write(&path, body).unwrap();
    path
}

fn nexus_bin() -> std::path::PathBuf {
    env!("CARGO_BIN_EXE_nexus").into()
}

fn expand(raw: &str, env: &ShellEnvironment) -> String {
    expand_word_for_exec(raw, env, 0).unwrap().into_string()
}

#[test]
fn positional_expansion_from_argv() {
    let mut env = ShellEnvironment::default();
    env.set_argv(vec!["script".into(), "one".into(), "two".into()]);
    assert_eq!(expand("$0", &env), "script");
    assert_eq!(expand("$1", &env), "one");
    assert_eq!(expand("$2", &env), "two");
    assert_eq!(expand("$#", &env), "2");
    assert_eq!(expand("$*", &env), "one two");
    assert_eq!(expand("$3", &env), "");
}

#[test]
fn run_script_file_with_args() {
    let out = std::env::temp_dir().join(format!("nexus_script_out_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let script = write_temp_script(
        "args",
        &format!("printf '%s\\n' \"$0\" \"$1\" \"$#\" > {}\n", out.display()),
    );
    let status = Command::new(nexus_bin())
        .arg(&script)
        .arg("hello")
        .status()
        .unwrap();
    assert!(status.success());
    let text = std::fs::read_to_string(&out).unwrap();
    assert_eq!(text, format!("{}\nhello\n1\n", script.display()));
    let _ = std::fs::remove_file(script);
    let _ = std::fs::remove_file(out);
}

#[test]
fn source_runs_in_current_environment() {
    let out = std::env::temp_dir().join(format!("nexus_source_out_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let script = write_temp_script("src", "set FOO=from_source\n");
    let input = format!(
        "source {}\nprintf '%s' \"$FOO\" > {}\n",
        script.display(),
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "from_source");
    let _ = std::fs::remove_file(script);
    let _ = std::fs::remove_file(out);
}

#[test]
fn source_exit_stops_shell() {
    let script = write_temp_script("exit", "exit 9\n");
    let input = format!("source {}\necho still\n", script.display());
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 9);
    assert!(!String::from_utf8(stdout).unwrap().contains("still"));
    let _ = std::fs::remove_file(script);
}

#[test]
fn missing_script_file_exits_one() {
    let status = Command::new(nexus_bin())
        .arg("/no/such/nexus_script_file")
        .output()
        .unwrap();
    assert_eq!(status.status.code(), Some(1));
    let err = String::from_utf8_lossy(&status.stderr);
    assert!(err.contains("No such file"));
}
