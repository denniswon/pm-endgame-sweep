//! PM Endgame Sweep - Ingestion service
//!
//! Discovers markets, polls quotes, and extracts rules from Polymarket.

pub mod client;
pub mod config;
pub mod orchestrator;
pub mod retry;

pub use client::{PolymarketClient, VenueClient};
pub use config::IngestConfig;
pub use orchestrator::IngestOrchestrator;
