//! Post-collect enrichment: descriptions, resource tags, score bumps.

mod cloud;
mod nexus;

use super::context::{self, Kind};
use super::docker;
use super::kube;
use super::match_item::{Match, Tag};
use crate::builtins::NAMES;
use crate::env::CompRegistry;

const RESOURCE_BONUS: i32 = 10;

/// Apply static descriptions and live-resource tagging after collection.
pub(super) fn enrich(matches: &mut [Match], before: &str, registry: &CompRegistry) {
    let kind = context::classify(before, registry);
    for m in matches.iter_mut() {
        cloud::apply(&kind, m);
        nexus::apply(&kind, m);
        if live_resource(&kind, &m.value) {
            m.tag = Tag::Resources;
            m.score += RESOURCE_BONUS;
        }
    }
}

fn live_resource(kind: &Kind, value: &str) -> bool {
    if value.starts_with('-') || value.starts_with('$') || value.contains('/') {
        return false;
    }
    if NAMES.contains(&value) {
        return false;
    }
    matches!(
        kind,
        Kind::Docker(docker::Complete::Logs)
            | Kind::Kube(kube::Complete::Pod { .. })
            | Kind::Kube(kube::Complete::Namespace)
    )
}
