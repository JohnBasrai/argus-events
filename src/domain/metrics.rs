use std::sync::Arc;
use std::time::Instant;

/// Abstraction for application metrics (counters, histograms).
pub trait Metrics: Send + Sync + 'static {
    // ---
    /// Render current metrics in Prometheus text format.
    fn render(&self) -> anyhow::Result<String>;

    /// Record a "event created" event.
    fn record_event_created(&self);

    /// Record HTTP request duration and labels.
    fn record_http_request(&self, start: Instant, path: &str, method: &str, status: u16);
}

/// Type alias for any backend that implements Metrics.
pub type MetricsPtr = Arc<dyn Metrics>;
