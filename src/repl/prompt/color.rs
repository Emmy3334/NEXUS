//! Minimal ANSI SGR helpers for themed prompt segments.

#[must_use]
pub fn fg(code: u8, text: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

#[must_use]
pub fn reset() -> &'static str {
    "\x1b[0m"
}

pub const GREEN: u8 = 32;
pub const YELLOW: u8 = 33;
pub const BLUE: u8 = 34;
pub const MAGENTA: u8 = 35;
pub const RED: u8 = 31;
