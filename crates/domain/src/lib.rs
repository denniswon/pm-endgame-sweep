//! PM Endgame Sweep - Core domain types and traits
//!
//! This crate defines the shared types used across all services.

pub mod market;
pub mod quote;
pub mod risk;
pub mod score;

pub use market::{Market, MarketStatus, Outcome};
pub use quote::Quote;
pub use risk::{RiskFlag, RuleSnapshot};
pub use score::{Recommendation, Score};
