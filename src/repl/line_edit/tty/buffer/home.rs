//! Beginning / end of line cursor motion.

use super::EditBuffer;

pub(in crate::repl::line_edit::tty) fn move_home(edit: &mut EditBuffer) {
    edit.cursor = 0;
}

pub(in crate::repl::line_edit::tty) fn move_end(edit: &mut EditBuffer) {
    edit.cursor = edit.text.len();
}
