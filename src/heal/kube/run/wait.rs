//! Poll Pod phase until Succeeded / Failed and read the exit code.

use crate::tokio_rt::io_other;
use k8s_openapi::api::core::v1::Pod;
use kube::api::Api;
use kube::Client;

use std::io;
use std::time::Duration;

const POLL: Duration = Duration::from_millis(200);
const MAX_WAIT: Duration = Duration::from_secs(90);

pub(super) async fn wait_status(client: &Client, name: &str) -> io::Result<u8> {
    let pods: Api<Pod> = Api::default_namespaced(client.clone());
    let deadline = tokio::time::Instant::now() + MAX_WAIT;
    loop {
        let pod = pods.get(name).await.map_err(io_other)?;
        if let Some(code) = terminal_exit(&pod) {
            return Ok(code);
        }
        if let Some(err) = pull_or_schedule_error(&pod) {
            return Err(io_other(err));
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(io_other(format!("kube heal timed out waiting for {name}")));
        }
        tokio::time::sleep(POLL).await;
    }
}

fn terminal_exit(pod: &Pod) -> Option<u8> {
    let status = pod.status.as_ref()?;
    let phase = status.phase.as_deref()?;
    if phase != "Succeeded" && phase != "Failed" {
        return None;
    }
    let exit = status
        .container_statuses
        .as_ref()?
        .first()?
        .state
        .as_ref()?
        .terminated
        .as_ref()?
        .exit_code;
    Some(exit.clamp(0, 255) as u8)
}

fn pull_or_schedule_error(pod: &Pod) -> Option<String> {
    let status = pod.status.as_ref()?;
    for cs in status.container_statuses.as_ref()?.iter() {
        let waiting = cs.state.as_ref()?.waiting.as_ref()?;
        let reason = waiting.reason.as_deref().unwrap_or("");
        if matches!(
            reason,
            "ErrImagePull" | "ImagePullBackOff" | "InvalidImageName"
        ) {
            return Some(waiting.message.clone().unwrap_or_else(|| reason.into()));
        }
    }
    None
}
