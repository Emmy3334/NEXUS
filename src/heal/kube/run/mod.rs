//! Create Pod, wait, fetch logs, delete.

mod logs;
mod remove;
mod spawn;
mod wait;

use crate::tokio_rt::io_other;
use kube::Client;
use logs::copy_logs;
use remove::delete_pod;
use spawn::create_pod;
use wait::wait_status;

use std::io::{self, Write};

pub(super) async fn execute(
    image: &str,
    argv: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let client = Client::try_default().await.map_err(io_other)?;
    let name = create_pod(&client, image, argv).await?;
    let code = match wait_status(&client, &name).await {
        Ok(code) => code,
        Err(err) => {
            let _ = delete_pod(&client, &name).await;
            return Err(err);
        }
    };
    let _ = copy_logs(&client, &name, code, stdout, stderr).await;
    let _ = delete_pod(&client, &name).await;
    Ok(code)
}
