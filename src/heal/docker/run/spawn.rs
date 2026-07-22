//! Create and start an ephemeral heal container.

use super::super::client::{map_err, Docker};
use bollard::container::{Config, RemoveContainerOptions};
use bollard::secret::HostConfig;

use std::io;
use std::path::PathBuf;

pub(super) async fn create_and_start(
    docker: &Docker,
    image: &str,
    argv: &[String],
) -> io::Result<String> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let cwd = cwd.to_string_lossy().into_owned();
    let config = container_config(image, argv, &cwd);
    let id = docker
        .create_container::<String, String>(None, config)
        .await
        .map_err(map_err)?
        .id;
    if let Err(err) = docker.start_container::<String>(&id, None).await {
        let _ = force_remove(docker, &id).await;
        return Err(map_err(err));
    }
    Ok(id)
}

fn container_config(image: &str, argv: &[String], cwd: &str) -> Config<String> {
    Config {
        image: Some(image.to_owned()),
        cmd: Some(argv.to_vec()),
        working_dir: Some(cwd.to_owned()),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        host_config: Some(HostConfig {
            binds: Some(vec![format!("{cwd}:{cwd}")]),
            auto_remove: Some(false),
            ..Default::default()
        }),
        ..Default::default()
    }
}

async fn force_remove(docker: &Docker, id: &str) -> io::Result<()> {
    docker
        .remove_container(
            id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await
        .map_err(map_err)
}
