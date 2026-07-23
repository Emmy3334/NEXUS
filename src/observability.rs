//! Optional stderr tracing (`NEXUS_LOG` / `RUST_LOG`).

use std::sync::Once;

static INIT: Once = Once::new();

/// Install a stderr subscriber once when `NEXUS_LOG` or `RUST_LOG` is set.
///
/// No-op when both are unset (zero noise for normal interactive use).
/// Safe to call repeatedly.
pub fn init() {
    INIT.call_once(install);
}

fn install() {
    let Ok(filter) = std::env::var("NEXUS_LOG").or_else(|_| std::env::var("RUST_LOG")) else {
        return;
    };
    if filter.is_empty() {
        return;
    }
    let env_filter = tracing_subscriber::EnvFilter::try_new(&filter)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .try_init();
}
