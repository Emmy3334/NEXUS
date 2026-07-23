//! Build a Kubernetes client from the local kubeconfig.

use crate::tokio_rt::{block_on, io_other};
use kube::Client;

use std::io;

pub(super) fn try_client() -> io::Result<Client> {
    tracing::debug!("kube try_client");
    block_on(Client::try_default())?.map_err(io_other)
}

/// True when the default kubeconfig yields a usable client (API ping).
#[must_use]
pub fn cluster_reachable() -> bool {
    tracing::debug!("kube cluster_reachable");
    match try_client() {
        Ok(client) => block_on(client.apiserver_version())
            .ok()
            .and_then(Result::ok)
            .is_some(),
        Err(_) => false,
    }
}
