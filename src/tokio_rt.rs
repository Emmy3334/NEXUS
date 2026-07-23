//! Shared Tokio runtime for sync call sites (Docker heal, Kubernetes).

use std::io;
use std::sync::{Mutex, OnceLock};
use tokio::runtime::Runtime;

pub(crate) fn block_on<F: std::future::Future>(fut: F) -> io::Result<F::Output> {
    Ok(runtime()?.block_on(fut))
}

pub(crate) fn io_other(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::other(err)
}

fn runtime() -> io::Result<&'static Runtime> {
    static RT: OnceLock<Runtime> = OnceLock::new();
    if let Some(rt) = RT.get() {
        return Ok(rt);
    }
    // Serialize first-time build (`get_or_try_init` is still unstable).
    static INIT: Mutex<()> = Mutex::new(());
    let _guard = INIT
        .lock()
        .map_err(|_| io_other("tokio runtime init poisoned"))?;
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
