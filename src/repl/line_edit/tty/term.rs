//! Raw-mode termios guard for the controlling terminal.

use nix::sys::termios::{tcgetattr, tcsetattr, InputFlags, LocalFlags, SetArg, Termios};
use nix::unistd::isatty;
use std::io::{self, Write};
use std::os::fd::{AsFd, AsRawFd};

pub(super) fn stdin_is_tty() -> bool {
    isatty(io::stdin().as_raw_fd()).unwrap_or(false)
}

pub(super) struct RawMode {
    original: Termios,
}

impl RawMode {
    pub(super) fn enter() -> io::Result<Self> {
        if !stdin_is_tty() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "line editor requires a terminal on stdin",
            ));
        }
        let stdin = io::stdin();
        let original = tcgetattr(stdin.as_fd()).map_err(nix_err)?;
        let mut raw = original.clone();
        raw.local_flags
            .remove(LocalFlags::ICANON | LocalFlags::ECHO | LocalFlags::ISIG);
        raw.input_flags.remove(InputFlags::IXON | InputFlags::ICRNL);
        tcsetattr(stdin.as_fd(), SetArg::TCSANOW, &raw).map_err(nix_err)?;
        // Enable bracketed paste so pasted newlines are not treated as Accept.
        let _ = io::stdout().write_all(b"\x1b[?2004h");
        let _ = io::stdout().flush();
        Ok(Self { original })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = io::stdout().write_all(b"\x1b[?2004l");
        let _ = io::stdout().flush();
        let stdin = io::stdin();
        let _ = tcsetattr(stdin.as_fd(), SetArg::TCSANOW, &self.original);
    }
}

pub(super) fn nix_err(err: nix::Error) -> io::Error {
    io::Error::from_raw_os_error(err as i32)
}
