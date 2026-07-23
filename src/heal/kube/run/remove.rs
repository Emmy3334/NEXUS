//! Best-effort Pod cleanup after heal.

use crate::tokio_rt::io_other;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, DeleteParams};
use kube::Client;

use std::io;

pub(super) async fn delete_pod(client: &Client, name: &str) -> io::Result<()> {
    let pods: Api<Pod> = Api::default_namespaced(client.clone());
    match pods.delete(name, &DeleteParams::default()).await {
        Ok(_) => Ok(()),
        Err(err) if err.to_string().contains("NotFound") => Ok(()),
        Err(err) => Err(io_other(err)),
    }
}
