//! PM Endgame Sweep - API service
//!
//! Read-only REST API for opportunities and market detail.

pub mod config;
pub mod handlers;
pub mod metrics;
pub mod server;
pub mod state;

pub use config::ApiConfig;
pub use metrics::Metrics;
pub use server::ApiServer;
pub use state::AppState;
