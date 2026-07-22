//! Shared Tokio runtime for sync heal call sites.

use std::io;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

pub(super) fn block_on<F: std::future::Future>(fut: F) -> io::Result<F::Output> {
    Ok(runtime()?.block_on(fut))
}

pub(super) fn io_other(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    // `Error::other` needs Rust ≥ 1.83; MSRV is 1.75.
    #[allow(clippy::io_other_error)]
    {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

fn runtime() -> io::Result<&'static Runtime> {
    static RT: OnceLock<Runtime> = OnceLock::new();
    if let Some(rt) = RT.get() {
        return Ok(rt);
    }
    let built = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(io_other)?;
    let _ = RT.set(built);
    RT.get().ok_or_else(|| io_other("tokio runtime missing"))
}
