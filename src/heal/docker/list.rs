//! List running container names for Tab completion.

use super::client::{self, map_err};
use super::runtime::block_on;
use bollard::container::ListContainersOptions;

use std::collections::HashMap;

/// Names (or short IDs) of running containers matching `prefix`.
///
/// Returns an empty list when the Docker daemon is unreachable or the API
/// call fails — Tab completion stays quiet instead of erroring the editor.
#[must_use]
pub fn running_names(prefix: &str) -> Vec<String> {
    let Ok(docker) = client::connect() else {
        return Vec::new();
    };
    match block_on(list_names(&docker, prefix)) {
        Ok(Ok(names)) => names,
        _ => Vec::new(),
    }
}

async fn list_names(docker: &client::Docker, prefix: &str) -> std::io::Result<Vec<String>> {
    let options = Some(ListContainersOptions::<String> {
        all: false,
        filters: HashMap::new(),
        ..Default::default()
    });
    let containers = docker.list_containers(options).await.map_err(map_err)?;
    let mut out = Vec::new();
    for container in containers {
        push_matches(
            container.names.as_deref(),
            container.id.as_deref(),
            prefix,
            &mut out,
        );
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn push_matches(names: Option<&[String]>, id: Option<&str>, prefix: &str, out: &mut Vec<String>) {
    if let Some(names) = names {
        for name in names {
            let bare = name.trim_start_matches('/');
            if bare.starts_with(prefix) {
                out.push(bare.to_owned());
            }
        }
    } else if let Some(id) = id {
        let short = &id[..id.len().min(12)];
        if short.starts_with(prefix) {
            out.push(short.to_owned());
        }
    }
}
