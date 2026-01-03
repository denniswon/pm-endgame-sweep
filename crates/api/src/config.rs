//! API configuration

use serde::{Deserialize, Serialize};

/// API service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// HTTP server bind address
    pub bind_addr: String,

    /// HTTP server port
    pub port: u16,

    /// Maximum page size for paginated endpoints
    pub max_page_size: usize,

    /// Default page size
    pub default_page_size: usize,

    /// Request timeout in seconds
    pub request_timeout_sec: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0".to_string(),
            port: 3000,
            max_page_size: 100,
            default_page_size: 20,
            request_timeout_sec: 30,
        }
    }
}
