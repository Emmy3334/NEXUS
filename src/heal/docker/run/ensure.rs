//! Ensure the heal image exists locally (pull if needed).

use super::super::client::{map_err, Docker};
use bollard::image::CreateImageOptions;
use futures_util::TryStreamExt;

use std::io;

pub(super) async fn ensure_image(docker: &Docker, image: &str) -> io::Result<()> {
    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: image,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await
        .map(|_| ())
        .map_err(map_err)
}
