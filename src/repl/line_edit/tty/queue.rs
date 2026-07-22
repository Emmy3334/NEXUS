//! Byte queue so paste / multi-byte reads are never truncated.

use super::term::nix_err;
use nix::unistd::read;

use std::collections::VecDeque;
use std::io;
use std::os::fd::AsRawFd;

/// Pull more bytes from the terminal into `queue` when it is empty.
pub(super) fn fill(queue: &mut VecDeque<u8>) -> io::Result<bool> {
    if !queue.is_empty() {
        return Ok(true);
    }
    fill_more(queue)
}

/// Always append a fresh `read` chunk (used to complete UTF-8 / escapes).
pub(super) fn fill_more(queue: &mut VecDeque<u8>) -> io::Result<bool> {
    let mut buf = [0u8; 512];
    let n = read(io::stdin().as_raw_fd(), &mut buf).map_err(nix_err)?;
    if n == 0 {
        return Ok(false);
    }
    queue.extend(&buf[..n]);
    Ok(true)
}

/// Peel the first newline-terminated line from a paste buffer.
pub fn take_complete_line(queue: &mut VecDeque<u8>) -> Option<String> {
    let nl = queue.iter().position(|&b| b == b'\n' || b == b'\r')?;
    let mut bytes = Vec::with_capacity(nl);
    for _ in 0..nl {
        bytes.push(queue.pop_front()?);
    }
    match queue.front().copied() {
        Some(b'\n') => {
            let _ = queue.pop_front();
        }
        Some(b'\r') => {
            let _ = queue.pop_front();
            if queue.front() == Some(&b'\n') {
                let _ = queue.pop_front();
            }
        }
        _ => {}
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}
