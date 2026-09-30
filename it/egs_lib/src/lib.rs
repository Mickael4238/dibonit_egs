//! # EGS Library
//!
//! Foundational library for Dibonit Entreprise Global Softwares (EGS) ecosystem.
//! Provides core data types, utilities, and common functionality used across all EGS components.

pub mod config;
pub mod error;
pub mod prelude;
pub mod types;
pub mod utils;

/// Re-export commonly used items from the prelude
pub use prelude::*;
