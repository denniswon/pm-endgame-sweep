//! Prometheus metrics

use std::sync::Arc;

use prometheus::{
    opts, register_int_counter_vec, register_int_gauge, Encoder, IntCounterVec, IntGauge, Registry,
    TextEncoder,
};

/// Metrics collector for API service
#[derive(Clone)]
pub struct Metrics {
    /// Total API requests by endpoint and status
    pub api_requests_total: IntCounterVec,

    /// Current number of active markets being tracked
    pub active_markets_gauge: IntGauge,

    /// Current number of recommendations available
    pub recommendations_gauge: IntGauge,

    /// Prometheus registry
    registry: Arc<Registry>,
}

impl Metrics {
    /// Create new metrics collector
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let api_requests_total = register_int_counter_vec!(
            opts!(
                "pm_api_requests_total",
                "Total number of API requests by endpoint and status"
            ),
            &["endpoint", "status"]
        )?;
        registry.register(Box::new(api_requests_total.clone()))?;

        let active_markets_gauge = register_int_gauge!(opts!(
            "pm_active_markets_total",
            "Current number of active markets being tracked"
        ))?;
        registry.register(Box::new(active_markets_gauge.clone()))?;

        let recommendations_gauge = register_int_gauge!(opts!(
            "pm_recommendations_total",
            "Current number of recommendations available"
        ))?;
        registry.register(Box::new(recommendations_gauge.clone()))?;

        Ok(Self {
            api_requests_total,
            active_markets_gauge,
            recommendations_gauge,
            registry: Arc::new(registry),
        })
    }

    /// Get metrics in Prometheus text format
    pub fn render(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer)
            .map_err(|e| prometheus::Error::Msg(format!("Failed to encode metrics: {}", e)))
    }

    /// Record an API request
    pub fn record_request(&self, endpoint: &str, status: u16) {
        self.api_requests_total
            .with_label_values(&[endpoint, &status.to_string()])
            .inc();
    }

    /// Update active markets count
    pub fn set_active_markets(&self, count: i64) {
        self.active_markets_gauge.set(count);
    }

    /// Update recommendations count
    pub fn set_recommendations(&self, count: i64) {
        self.recommendations_gauge.set(count);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}
