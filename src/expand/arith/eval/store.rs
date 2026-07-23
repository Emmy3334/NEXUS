//! Shared writeback for `$((…))` assignments and `++`/`--`.

use crate::env::ShellEnvironment;

pub(super) fn store(env: &mut ShellEnvironment, name: &str, value: i64) {
    let text = value.to_string();
    if env.get_local(name).is_some() {
        env.set_local(name, text);
    } else if env.contains(name) {
        env.set(name, text);
    } else {
        env.set_local(name, text);
    }
}
