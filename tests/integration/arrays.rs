//! Integration tests for `typeset -a` arrays and `${name[i]}` expansion.

use nexus::repl;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn test_env() -> nexus::env::ShellEnvironment {
    let path = std::env::var("PATH").unwrap_or_default();
    let mut map = BTreeMap::new();
    map.insert("PATH".into(), path);
    nexus::env::ShellEnvironment::from_map(map)
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
        false,
        &mut env,
    )
    .unwrap();
    (code, String::from_utf8(stderr).unwrap())
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_arr_{name}_{}", std::process::id()))
}

#[test]
fn typeset_array_index_and_join() {
    let path = scratch("basic");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -a fruits=apple:banana:cherry
printf '%s\\n' ${{fruits[1]}} > {}
printf '%s\\n' ${{fruits[2]}} >> {}
printf '%s\\n' ${{fruits[*]}} >> {}
printf '%s\\n' ${{#fruits}} >> {}
",
        path.display(),
        path.display(),
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "apple\nbanana\napple banana cherry\n3\n"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn at_splat_expands_to_separate_fields() {
    let path = scratch("splat");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -a fruits=apple:banana:cherry
printf '%s\\n' ${{fruits[@]}} > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "apple\nbanana\ncherry\n"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn quoted_at_splat_keeps_separate_fields() {
    let path = scratch("qsplat");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -a fruits=apple:banana
printf '%s\\n' \"${{fruits[@]}}\" > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "apple\nbanana\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn quoted_star_joins_to_one_field() {
    let path = scratch("qstar");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -a fruits=apple:banana
printf '%s\\n' \"${{fruits[*]}}\" > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "apple banana\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn empty_at_splat_contributes_no_fields() {
    let path = scratch("empty_at");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -a empty
printf 'X%sX\\n' ${{empty[@]}} > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    // No empty argv word: printf gets only the format → prints `XX` plus newline.
    assert_eq!(fs::read_to_string(&path).unwrap(), "XX\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn typeset_empty_array_and_out_of_range() {
    let path = scratch("empty");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -a empty
printf '%s\\n' ${{empty[1]}} > {}
printf '%s\\n' ${{#empty}} >> {}
",
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "\n0\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn typeset_array_outside_function() {
    let path = scratch("global");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -a items=one:two
printf '%s\\n' ${{items[2]}} > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "two\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn typeset_a_x_combined_errors() {
    let (code, err) = run("typeset -a -x combo=a:b\n");
    assert_eq!(code, 1);
    assert!(err.contains("combined -a -x unsupported"), "{err}");
}
