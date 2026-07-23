//! Build and create an ephemeral heal Pod.

mod cwd;

use crate::tokio_rt::io_other;
use cwd::{cwd_string, host_cwd_volume, host_cwd_volume_mount};
use k8s_openapi::api::core::v1::{Container, EnvVar, Pod, PodSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Api, PostParams};
use kube::{Client, ResourceExt};

use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) async fn create_pod(
    client: &Client,
    image: &str,
    argv: &[String],
    env: Vec<EnvVar>,
) -> io::Result<String> {
    let pods: Api<Pod> = Api::default_namespaced(client.clone());
    let pod = heal_pod(image, argv, env);
    let name = pod.name_any();
    pods.create(&PostParams::default(), &pod)
        .await
        .map_err(io_other)?;
    Ok(name)
}

fn heal_pod(image: &str, argv: &[String], env: Vec<EnvVar>) -> Pod {
    let cwd = cwd_string();
    Pod {
        metadata: ObjectMeta {
            name: Some(unique_name()),
            ..Default::default()
        },
        spec: Some(PodSpec {
            restart_policy: Some("Never".into()),
            volumes: Some(vec![host_cwd_volume(&cwd)]),
            containers: vec![Container {
                name: "heal".into(),
                image: Some(image.to_owned()),
                command: Some(argv.to_vec()),
                env: Some(env),
                image_pull_policy: Some("IfNotPresent".into()),
                working_dir: Some(cwd.clone()),
                volume_mounts: Some(vec![host_cwd_volume_mount(&cwd)]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn unique_name() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("nexus-heal-{}-{}", std::process::id(), millis % 1_000_000)
}
