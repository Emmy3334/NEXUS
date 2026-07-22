//! Execute a WASI Preview1 command module with Wasmtime.

use super::{engine, exit_code};
use wasmtime::{Linker, Module, Store};
use wasmtime_wasi::pipe::MemoryOutputPipe;
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

use std::io::{self, Write};
use std::path::Path;

/// Load `path` and run `_start` with WASI args; copy guest stdio to writers.
pub fn run_module(
    path: &Path,
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let engine = engine::shared();
    let module = Module::from_file(engine, path).map_err(io_err)?;
    let mut linker: Linker<WasiP1Ctx> = Linker::new(engine);
    preview1::add_to_linker_sync(&mut linker, |cx| cx).map_err(io_err)?;

    let out_pipe = MemoryOutputPipe::new(1 << 20);
    let err_pipe = MemoryOutputPipe::new(1 << 20);
    let wasi_args = build_args(path, args);
    let wasi = WasiCtxBuilder::new()
        .stdout(out_pipe.clone())
        .stderr(err_pipe.clone())
        .args(&wasi_args)
        .build_p1();

    let mut store = Store::new(engine, wasi);
    let instance = linker.instantiate(&mut store, &module).map_err(io_err)?;
    let start = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .map_err(io_err)?;
    let status = exit_code::from_call_result(start.call(&mut store, ()), stderr)?;
    stdout.write_all(&out_pipe.contents())?;
    stderr.write_all(&err_pipe.contents())?;
    Ok(status)
}

fn build_args(path: &Path, args: &[String]) -> Vec<String> {
    let mut full = Vec::with_capacity(args.len() + 1);
    full.push(path.to_string_lossy().into_owned());
    full.extend(args.iter().cloned());
    full
}

fn io_err(err: impl std::fmt::Display) -> io::Error {
    // MSRV 1.78: `Error::other` requires 1.83+.
    #[allow(clippy::io_other_error)]
    {
        io::Error::new(io::ErrorKind::Other, err.to_string())
    }
}
