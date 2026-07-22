//! Docker daemon connect / health check.

pub(super) use bollard::Docker;

use std::io;

pub(super) fn connect() -> io::Result<Docker> {
    Docker::connect_with_socket_defaults().map_err(map_err)
}

pub(super) async fn ping(docker: &Docker) -> io::Result<()> {
    docker.ping().await.map(|_| ()).map_err(map_err)
}

pub(super) fn map_err(err: bollard::errors::Error) -> io::Error {
    // `Error::other` needs Rust ≥ 1.83; MSRV is 1.75.
    #[allow(clippy::io_other_error)]
    {
        io::Error::new(io::ErrorKind::Other, err.to_string())
    }
}
