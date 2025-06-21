//! # Space Earth DTN (Delay Tolerant Network) Library
//!
//! This library implements a DTN (Delay Tolerant Network) system for space communications.
//! It provides bundle protocol v7 (BPv7) support, routing algorithms, and various
//! convergence layer adapters (CLAs).

// Core modules
pub mod api;
pub mod bpv7;
pub mod cla;
pub mod config;
pub mod routing;
pub mod store;

// Constants and utilities
pub mod consts;

// Re-export commonly used types for convenience
pub use api::{node::DtnNode, BundleStatus};
pub use bpv7::{bundle::Bundle, EndpointId};

// Re-export convenience functions for easy access
pub use api::convenience;
