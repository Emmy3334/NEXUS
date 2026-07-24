//! Container table for `@docker ps` (running, or all with `-a`; IDs with `-q`).

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
///
/// `all` matches Docker’s `-a` / `--all` (include exited containers).
/// `quiet` matches `-q` / `--quiet` (short IDs only, no header).
pub fn list_ps_lines(all: bool, quiet: bool) -> io::Result<Vec<String>> {
    let docker = client::connect()?;
    match block_on(fetch(&docker, all, quiet)) {
        Ok(inner) => inner,
        Err(err) => Err(err),
    }
}

async fn fetch(docker: &Docker, all: bool, quiet: bool) -> io::Result<Vec<String>> {
    let options = Some(ListContainersOptions::<String> {
        all,
        filters: HashMap::new(),
        ..Default::default()
    });
    let containers = docker.list_containers(options).await.map_err(map_err)?;
    Ok(if quiet {
        format::ids(&containers)
    } else {
        format::render(&containers)
    })
}
