//! List namespace names (Tab complete / helpers).

use super::client::try_client;
use crate::tokio_rt::{block_on, io_other};
use k8s_openapi::api::core::v1::Namespace;
use kube::api::Api;
use kube::ResourceExt;

use std::io;

/// Namespace names matching `prefix` (empty on API failure).
pub fn list_namespace_names(prefix: &str) -> Vec<String> {
    let Ok(client) = try_client() else {
        return Vec::new();
    };
    match block_on(names_for(client, prefix)) {
        Ok(Ok(names)) => names,
        _ => Vec::new(),
    }
}

async fn names_for(client: kube::Client, prefix: &str) -> io::Result<Vec<String>> {
    let list = Api::<Namespace>::all(client)
        .list(&Default::default())
        .await
        .map_err(io_other)?;
    Ok(list
        .iter()
        .map(ResourceExt::name_any)
        .filter(|name| name.starts_with(prefix))
        .collect())
}
