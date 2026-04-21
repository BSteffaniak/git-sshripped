//! Optional tracing subscriber for the `profiling` crate.
//!
//! When the `profile-trace` Cargo feature is enabled **and** the
//! `GIT_SSHRIPPED_TRACE` environment variable is set to `1`, `true`, or
//! `on`, a [`tracing_subscriber`] fmt layer is attached to stderr that
//! prints a timing line every time a [`profiling::scope!`] or
//! `#[profiling::function]` scope opens and closes.
//!
//! When the feature is disabled this module compiles to a pair of no-op
//! functions.

/// Initialise the tracing subscriber if `GIT_SSHRIPPED_TRACE` is set.
///
/// Safe to call multiple times; subsequent calls are no-ops because
/// [`tracing::subscriber::set_global_default`] only succeeds once.
#[cfg(feature = "profile-trace")]
pub fn init_trace() {
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::{EnvFilter, fmt};

    let enabled = std::env::var("GIT_SSHRIPPED_TRACE")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "on" | "yes"));

    if !enabled {
        return;
    }

    // Allow users to narrow which spans are emitted by also accepting a
    // standard `RUST_LOG`-style filter. Default to `trace` so every
    // instrumented span is captured.
    let filter = std::env::var("RUST_LOG")
        .ok()
        .map_or_else(|| EnvFilter::new("trace"), EnvFilter::new);

    let _ = fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_target(false)
        .with_ansi(false)
        .try_init();
}

/// No-op when the `profile-trace` feature is disabled.
#[cfg(not(feature = "profile-trace"))]
pub const fn init_trace() {}
