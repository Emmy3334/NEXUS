//! Observability init smoke tests.

use nexus::observability;

#[test]
fn init_is_idempotent_when_unset() {
    // Ensure neither filter forces a subscriber for this process if already set.
    observability::init();
    observability::init();
}
