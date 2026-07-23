//! Function call depth and `return` request flag.

use super::ShellEnvironment;

impl ShellEnvironment {
    /// Current function call nesting depth.
    #[must_use]
    pub fn func_depth(&self) -> u32 {
        self.func_depth
    }

    pub(crate) fn enter_function(&mut self) {
        self.func_depth = self.func_depth.saturating_add(1);
    }

    pub(crate) fn leave_function(&mut self) {
        self.func_depth = self.func_depth.saturating_sub(1);
    }

    pub(crate) fn request_return(&mut self, code: u8) {
        self.pending_return = Some(code);
    }

    #[must_use]
    pub(crate) fn return_requested(&self) -> bool {
        self.pending_return.is_some()
    }

    pub(crate) fn take_return(&mut self) -> Option<u8> {
        self.pending_return.take()
    }
}
