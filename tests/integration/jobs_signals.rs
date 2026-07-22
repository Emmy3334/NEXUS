//! Signal / job-control helper smoke tests (TTY-sensitive paths stay manual).

use nexus::jobs::job_control_active;

#[test]
fn job_control_inactive_in_piped_tests() {
    assert!(!job_control_active());
}
