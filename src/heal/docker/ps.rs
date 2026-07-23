//! Running-container table for `@docker ps`.

use super::client::{self, map_err, Docker};
use super::runtime::block_on;
use bollard::container::ListContainersOptions;
use bollard::models::ContainerSummary;

use std::collections::HashMap;
use std::io;

/// Lines for `@docker ps` (`NAME ID IMAGE STATUS`), daemon errors propagate.
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
    let mut lines = Vec::with_capacity(containers.len() + 1);
    lines.push("NAME\tID\tIMAGE\tSTATUS".to_owned());
    for container in containers {
        lines.push(format_row(&container));
    }
    Ok(lines)
}

fn format_row(container: &ContainerSummary) -> String {
    let name = container
        .names
        .as_ref()
        .and_then(|ns| ns.first())
        .map(|n| n.trim_start_matches('/').to_owned())
        .or_else(|| container.id.as_ref().map(|id| short_id(id)))
        .unwrap_or_else(|| "-".to_owned());
    let id = container
        .id
        .as_deref()
        .map(short_id)
        .unwrap_or_else(|| "-".to_owned());
    let image = container.image.as_deref().unwrap_or("-");
    let status = container.status.as_deref().unwrap_or("-");
    format!("{name}\t{id}\t{image}\t{status}")
}

fn short_id(id: &str) -> String {
    id.chars().take(12).collect()
}
