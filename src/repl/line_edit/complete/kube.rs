//! Running Kubernetes pod names for `@kube logs`.

use crate::kube;

pub(super) fn collect(prefix: &str, out: &mut Vec<String>) {
    out.extend(kube::list_pod_names(prefix));
}
