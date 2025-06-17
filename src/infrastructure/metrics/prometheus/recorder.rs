use anyhow::Result;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the Prometheus recorder globally and store the handle.
/// This function is safe to call multiple times - it will only initialize once.
/// Returns true if initialization was successful, false if already initialized.
pub fn init_metrics() -> Result<bool> {
    // ---
    // Check if already initialized before attempting initialization
    if HANDLE.get().is_some() {
        return Ok(false); // Already initialized
    }

    // Try to initialize the recorder
    let handle = PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| anyhow::anyhow!("Failed to install Prometheus recorder: {}", e))?;

    // Store the handle - this will only succeed if we're the first to initialize
    match HANDLE.set(handle) {
        Ok(()) => Ok(true),  // Successfully initialized
        Err(_) => Ok(false), // Another thread beat us to it, but that's fine
    }
}

/// Render the current metrics in Prometheus text format.
pub fn render_metrics() -> Result<String> {
    // ---
    let handle = HANDLE.get().ok_or_else(|| {
        anyhow::anyhow!("Metrics recorder not initialized. Call init_metrics() first.")
    })?;

    Ok(handle.render())
}
