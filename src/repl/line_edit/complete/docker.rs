//! Running Docker container names for `@docker logs`.

use crate::heal;

pub(super) fn collect(prefix: &str, out: &mut Vec<String>) {
    out.extend(heal::running_names(prefix));
}
