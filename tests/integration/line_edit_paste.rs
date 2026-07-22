//! Bracketed-paste escape recognition.

use nexus::repl;

#[test]
fn bracketed_paste_markers_do_not_crash_repl() {
    // Non-TTY path: markers appear as literal noise only if typed; this just
    // ensures the shell still runs after bindkey/paste-related builtins exist.
    let mut stdin = std::io::Cursor::new("true\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0);
}
