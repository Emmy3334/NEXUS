//! Wait for container exit status.

use super::super::client::{map_err, Docker};
use bollard::container::WaitContainerOptions;
use futures_util::StreamExt;

use std::io;

pub(super) async fn wait_status(docker: &Docker, id: &str) -> io::Result<u8> {
    let mut stream = docker.wait_container(id, None::<WaitContainerOptions<String>>);
    let response = stream
        .next()
        .await
        .ok_or_else(|| super::super::runtime::io_other("docker wait ended early"))?
        .map_err(map_err)?;
    Ok(response.status_code.clamp(0, 255) as u8)
}
