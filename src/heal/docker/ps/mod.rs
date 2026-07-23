//! Running-container table for `@docker ps`.

mod age;
mod fields;
mod format;
mod ports;

use super::client::{self, map_err, Docker};
use super::runtime::block_on;
use bollard::container::ListContainersOptions;

use std::collections::HashMap;
use std::io;

/// Lines for `@docker ps` (Docker CLI–style columns); daemon errors propagate.
pub fn list_ps_lines() -> io::Result<Vec<String>> {
    let docker = client::connect()?;
    match block_on(fetch(&docker)) {
        Ok(inner) => inner,
        Err(err) => Err(err),
    }
}

async fn fetch(docker: &Docker) -> io::Result<Vec<String>> {
    let options = Some(ListContainersOptions::<String> {
        all: false,
        filters: HashMap::new(),
        ..Default::default()
    });
    let containers = docker.list_containers(options).await.map_err(map_err)?;
    Ok(format::render(&containers))
}
