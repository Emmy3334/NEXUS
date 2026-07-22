//! Vi insert / command map switching.

use super::KeyBindings;

impl KeyBindings {
    pub fn enter_command_map(&mut self) {
        self.alternate_active = true;
    }

    pub fn enter_insert_map(&mut self) {
        self.alternate_active = false;
    }

    #[must_use]
    pub fn alternate_active(&self) -> bool {
        self.alternate_active
    }
}
