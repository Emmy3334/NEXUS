//! Raw-mode termios guard for the controlling terminal.

use nix::sys::termios::{tcgetattr, tcsetattr, InputFlags, LocalFlags, SetArg, Termios};
use nix::unistd::isatty;
use std::io;
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
        Ok(Self { original })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let stdin = io::stdin();
        let _ = tcsetattr(stdin.as_fd(), SetArg::TCSANOW, &self.original);
    }
}

pub(super) fn nix_err(err: nix::Error) -> io::Error {
    io::Error::from_raw_os_error(err as i32)
}
