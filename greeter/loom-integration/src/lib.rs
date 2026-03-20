//! loom-integration — gRPC client wrapper for the loom backend.
//!
//! Provides two clients for use by the greeter:
//! - [`MapClient`]: fetches the map for the current contest.
//! - [`StationClient`]: manages the persistent bidirectional station stream.

pub mod map;
pub mod station;

pub use map::MapClient;
pub use station::StationClient;
