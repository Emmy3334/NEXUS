//! Pull image, run one-shot container, stream logs, return exit status.

mod ensure;
mod logs;
mod spawn;
mod stdin;
mod wait;

use super::client::Docker;
use ensure::ensure_image;
pub(super) use logs::copy_logs;
use spawn::create_and_start;
use wait::wait_status;

use std::io::{self, Write};

pub(super) async fn execute(
    docker: &Docker,
    image: &str,
    argv: &[String],
    env: &[String],
    stdin: Option<&[u8]>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<(u8, String)> {
    ensure_image(docker, image).await?;
    let want_stdin = stdin.map(|b| !b.is_empty()).unwrap_or(false);
    let id = create_and_start(docker, image, argv, env, want_stdin).await?;
    if let Some(bytes) = stdin.filter(|b| !b.is_empty()) {
        stdin::feed(docker, &id, bytes).await?;
    }
    let code = wait_status(docker, &id).await?;
    copy_logs(docker, &id, stdout, stderr).await?;
    let _ = remove(docker, &id).await;
    Ok((code, short_id(&id)))
}

fn short_id(id: &str) -> String {
    id.chars().take(12).collect()
}

async fn remove(docker: &Docker, id: &str) -> io::Result<()> {
    use super::client::map_err;
    use bollard::container::RemoveContainerOptions;
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
