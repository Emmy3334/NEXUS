//! Copy container stdout/stderr into the shell writers.

use super::super::client::{map_err, Docker};
use bollard::container::{LogOutput, LogsOptions};
use futures_util::StreamExt;

use std::io::{self, Write};

pub(in crate::heal::docker) async fn copy_logs(
    docker: &Docker,
    id: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<()> {
    let mut stream = docker.logs(
        id,
        Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        }),
    );
    while let Some(item) = stream.next().await {
        write_chunk(item.map_err(map_err)?, stdout, stderr)?;
    }
    Ok(())
}

fn write_chunk(chunk: LogOutput, stdout: &mut dyn Write, stderr: &mut dyn Write) -> io::Result<()> {
    match chunk {
        LogOutput::StdOut { message } | LogOutput::Console { message } => {
            stdout.write_all(&message)
        }
        LogOutput::StdErr { message } => stderr.write_all(&message),
        LogOutput::StdIn { .. } => Ok(()),
    }
}
