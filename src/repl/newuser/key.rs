//! Read a single keypress (raw on TTY; first non-newline byte otherwise).

use std::io::{self, BufRead, Write};

/// Prompt and read one key. On a TTY this is a single keypress; in tests, first byte.
pub(super) fn read_key(stdin: &mut impl BufRead, stdout: &mut impl Write) -> io::Result<char> {
    write!(stdout, "--- Type one of the keys in parentheses --- ")?;
    stdout.flush()?;
    #[cfg(unix)]
    let _guard = cbreak::enter()?;
    let mut buf = [0u8; 1];
    loop {
        if stdin.read(&mut buf)? == 0 {
            return Ok('q');
        }
        match buf[0] {
            b'\n' | b'\r' => continue,
            b => {
                writeln!(stdout)?;
                return Ok(b as char);
            }
        }
    }
}

#[cfg(unix)]
mod cbreak {
    use nix::sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg};
    use nix::unistd::isatty;
    use std::io;
    use std::os::fd::{AsFd, AsRawFd};

    pub(super) struct Guard {
        original: nix::sys::termios::Termios,
    }

    pub(super) fn enter() -> io::Result<Option<Guard>> {
        if !isatty(io::stdin().as_raw_fd()).unwrap_or(false) {
            return Ok(None);
        }
        let stdin = io::stdin();
        let original = tcgetattr(stdin.as_fd()).map_err(nix_err)?;
        let mut raw = original.clone();
        raw.local_flags
            .remove(LocalFlags::ICANON | LocalFlags::ECHO);
        tcsetattr(stdin.as_fd(), SetArg::TCSANOW, &raw).map_err(nix_err)?;
        Ok(Some(Guard { original }))
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            let stdin = io::stdin();
            let _ = tcsetattr(stdin.as_fd(), SetArg::TCSANOW, &self.original);
        }
    }

    fn nix_err(err: nix::Error) -> io::Error {
        io::Error::from_raw_os_error(err as i32)
    }
}
