//! Format `@docker ps` rows like the Docker CLI table.

use super::{age, fields, ports};
use bollard::models::ContainerSummary;

const HEADER: [&str; 7] = [
    "CONTAINER ID",
    "IMAGE",
    "COMMAND",
    "CREATED",
    "STATUS",
    "PORTS",
    "NAMES",
];

/// Build header + data lines with space-padded columns.
pub(super) fn render(containers: &[ContainerSummary]) -> Vec<String> {
    let mut rows = Vec::with_capacity(containers.len() + 1);
    rows.push(HEADER.map(str::to_owned));
    for container in containers {
        rows.push(cells(container));
    }
    pad_rows(&rows)
}

/// One short container ID per line (`docker ps -q`).
pub(super) fn ids(containers: &[ContainerSummary]) -> Vec<String> {
    containers
        .iter()
        .map(|container| {
            container
                .id
                .as_deref()
                .map(fields::short_id)
                .unwrap_or_else(|| "-".into())
        })
        .collect()
}

fn cells(container: &ContainerSummary) -> [String; 7] {
    [
        container
            .id
            .as_deref()
            .map(fields::short_id)
            .unwrap_or_else(|| "-".into()),
        fields::display_image(container.image.as_deref().unwrap_or("-")),
        fields::quote_cmd(container.command.as_deref().unwrap_or("-")),
        age::created_ago(container.created),
        container.status.clone().unwrap_or_else(|| "-".into()),
        ports::format_ports(container.ports.as_deref().unwrap_or(&[])),
        fields::container_name(container),
    ]
}

fn pad_rows(rows: &[[String; 7]]) -> Vec<String> {
    let mut widths = [0_usize; 7];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }
    rows.iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(i, cell)| {
                    if i + 1 == row.len() {
                        cell.clone()
                    } else {
                        format!("{cell:<width$}", width = widths[i] + 2)
                    }
                })
                .collect()
        })
        .collect()
}
