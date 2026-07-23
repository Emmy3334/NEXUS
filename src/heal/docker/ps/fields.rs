//! Cell values for `@docker ps` columns.

use bollard::models::ContainerSummary;

pub(super) fn short_id(id: &str) -> String {
    id.chars().take(12).collect()
}

pub(super) fn display_image(image: &str) -> String {
    image
        .split_once("@sha256:")
        .map(|(name, _)| name.to_owned())
        .unwrap_or_else(|| image.to_owned())
}

pub(super) fn quote_cmd(cmd: &str) -> String {
    let max = 18_usize;
    let count = cmd.chars().count();
    let inner = if count <= max {
        cmd.to_owned()
    } else {
        let mut out: String = cmd.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    };
    format!("\"{inner}\"")
}

pub(super) fn container_name(container: &ContainerSummary) -> String {
    container
        .names
        .as_ref()
        .and_then(|ns| ns.first())
        .map(|n| n.trim_start_matches('/').to_owned())
        .or_else(|| container.id.as_ref().map(|id| short_id(id)))
        .unwrap_or_else(|| "-".into())
}
