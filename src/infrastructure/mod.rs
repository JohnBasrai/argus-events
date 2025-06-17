mod metrics;

// Re-export the factory functions for easy access
use anyhow::{anyhow, Result};
use metrics::{noop::create as create_noop_metrics, prometheus::create as create_prom_metrics};
use std::env;

use crate::domain::MetricsPtr;

pub fn create_metrics() -> Result<MetricsPtr> {
    // --
    // Determine metrics implementation from environment
    let metrics_type = env::var("ARGUS_METRICS_TYPE").unwrap_or_else(|_| "noop".to_string());

    if metrics_type == "prom" {
        create_prom_metrics()
    } else if metrics_type == "noop" {
        create_noop_metrics()
    } else {
        Err(anyhow!(
            "Invalid value for ARGUS_METRICS_TYPE: {}",
            metrics_type
        ))
    }
}
