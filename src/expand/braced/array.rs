//! `${name[i]}`, `${name[@]}`, and `${name[*]}` expansion.

use super::value;
use super::ExpandedWord;
use crate::env::ShellEnvironment;

pub(super) fn index(
    name: &str,
    index: &str,
    env: &ShellEnvironment,
    out: &mut ExpandedWord,
    globable: bool,
) {
    let Some(elements) = env.array_get(name) else {
        return;
    };
    let text = match index {
        "@" | "*" => elements.join(" "),
        digits => {
            let Ok(n) = digits.parse::<usize>() else {
                return;
            };
            if n == 0 {
                return;
            }
            elements.get(n - 1).cloned().unwrap_or_default()
        }
    };
    value::push_text(&text, out, globable);
}
