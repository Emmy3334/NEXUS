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

#[test]
fn assoc_key_lookup_and_count() {
    let path = scratch("assoc_lookup");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -A colors=red:ff0000,green:00ff00,blue:0000ff
printf '%s\\n' ${{colors[green]}} > {}
printf '%s\\n' ${{colors[missing]}} >> {}
printf '%s\\n' ${{#colors}} >> {}
",
        path.display(),
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "00ff00\n\n3\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn assoc_keys_and_values_flags() {
    let path = scratch("assoc_kv");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -A colors=red:ff0000,green:00ff00,blue:0000ff
printf '%s\\n' ${{(k)colors}} > {}
printf '%s\\n' ${{(v)colors}} >> {}
",
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    // Keys are stored sorted (BTreeMap); values follow key order.
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "blue\ngreen\nred\n0000ff\n00ff00\nff0000\n"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn assoc_at_splat_and_star_join() {
    let path = scratch("assoc_at");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -A pair=a:1,b:2
printf '%s\\n' ${{pair[@]}} > {}
printf '%s\\n' \"${{pair[*]}}\" >> {}
",
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "1\n2\n1 2\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn assoc_empty_and_reassign_wins() {
    let path = scratch("assoc_empty");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
typeset -A h
printf 'X%sX\\n' ${{(k)h}} > {}
typeset -A h=k:first,k:second
printf '%s\\n' ${{h[k]}} >> {}
",
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    // Empty assoc contributes no fields; later duplicate key wins.
    assert_eq!(fs::read_to_string(&path).unwrap(), "XX\nsecond\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn typeset_a_capital_a_combined_errors() {
    let (code, err) = run("typeset -a -A combo=a:b\n");
    assert_eq!(code, 1);
    assert!(err.contains("combined -a -A unsupported"), "{err}");
}
