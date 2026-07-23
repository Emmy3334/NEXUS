//! Feed finite stdin bytes into a started heal container.

use super::super::client::{map_err, Docker};
use bollard::container::{AttachContainerOptions, AttachContainerResults};
use tokio::io::AsyncWriteExt;

use std::io;

pub(super) async fn feed(docker: &Docker, id: &str, bytes: &[u8]) -> io::Result<()> {
    let AttachContainerResults {
        mut input,
        output: _,
    } = docker
        .attach_container(
            id,
            Some(AttachContainerOptions::<String> {
                stdin: Some(true),
                stream: Some(true),
                ..Default::default()
            }),
        )
        .await
        .map_err(map_err)?;
    input.write_all(bytes).await?;
    input.shutdown().await?;
    Ok(())
}
