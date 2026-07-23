//! Host cwd as a `hostPath` volume (Docker-heal parity when the node can see it).

use k8s_openapi::api::core::v1::{HostPathVolumeSource, Volume, VolumeMount};

use std::path::PathBuf;

pub(super) fn cwd_string() -> String {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("/"))
        .to_string_lossy()
        .into_owned()
}

pub(super) fn host_cwd_volume(cwd: &str) -> Volume {
    Volume {
        name: "nexus-cwd".into(),
        host_path: Some(HostPathVolumeSource {
            path: cwd.to_owned(),
            type_: Some("DirectoryOrCreate".into()),
        }),
        ..Default::default()
    }
}

pub(super) fn host_cwd_volume_mount(cwd: &str) -> VolumeMount {
    VolumeMount {
        name: "nexus-cwd".into(),
        mount_path: cwd.to_owned(),
        ..Default::default()
    }
}
