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

#[test]
fn local_outside_function_errors() {
    let (code, err) = run("local x=1\n");
    assert_eq!(code, 1);
    assert!(err.contains("not in a function"), "{err}");
}

#[test]
fn local_restores_outer_after_function() {
    let path = scratch("local_restore");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
set x=outer
f() {{
local x=inner
printf '%s\\n' $x
}}
f > {}
printf '%s\\n' $x >> {}
",
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "inner\nouter\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn local_bare_name_is_empty() {
    let path = scratch("local_bare");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
set x=outer
f() {{
local x
printf '%s\\n' \"empty:$x\"
}}
f > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "empty:\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn local_nested_frames_restore() {
    let path = scratch("local_nested");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
set x=global
inner() {{
local x=inner
printf '%s\\n' $x
}}
outer() {{
local x=outer
inner
printf '%s\\n' $x
}}
outer > {}
printf '%s\\n' $x >> {}
",
        path.display(),
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "inner\nouter\nglobal\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn local_restores_after_return() {
    let path = scratch("local_return");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
set x=outer
f() {{
local x=inner
return 0
printf '%s\\n' never
}}
f
printf '%s\\n' $x > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "outer\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn nested_foreach_in_function() {
    let path = scratch("foreach_fn");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
walk() {{
foreach i (a b)
printf '%s\\n' \"$i\"
end
}}
walk > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "a\nb\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn nested_while_if_in_function() {
    let path = scratch("while_if_fn");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
count() {{
set i=0
while ($i < 2)
if (1) then
printf '%s\\n' \"$i\"
endif
@ i++
end
}}
count > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "0\n1\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn return_from_nested_while_in_function() {
    let path = scratch("ret_while_fn");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
early() {{
set i=0
while ($i < 5)
if ($i == 1) then
return 7
endif
@ i++
end
printf '%s\\n' never
}}
early
printf '%s\\n' $? > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "7\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn nested_case_in_function() {
    let path = scratch("case_fn");
    let _ = fs::remove_file(&path);
    let script = format!(
        "\
pick() {{
case $1 in
a)
printf 'A\\n'
;;
*)
printf 'other\\n'
;;
esac
}}
pick a > {}
",
        path.display()
    );
    let (code, err) = run(&script);
    assert_eq!(code, 0, "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "A\n");
    let _ = fs::remove_file(&path);
}
