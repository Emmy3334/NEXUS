//! Tests for `repeat`.

use nexus::repl;
use std::io::Cursor;

#[test]
fn repeat_runs_command_n_times() {
    let out = std::env::temp_dir().join(format!("nexus_repeat_{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let input = format!(
        "set n=0\nrepeat 3 @ n++\nprintf '%s\\n' \"$n\" > {}\n",
        out.display()
    );
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "err={}", String::from_utf8_lossy(&stderr));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "3\n");
    let _ = std::fs::remove_file(out);
}
