//! PM Endgame Sweep - Scoring service
//!
//! Computes opportunity scores and risk decomposition.

pub mod config;
pub mod engine;
pub mod orchestrator;

pub use config::ScoringConfig;
pub use engine::ScoringEngine;
pub use orchestrator::ScoringOrchestrator;
