//! Global cache management for Nargo package manager.
//!
//! This crate provides caching functionality for downloaded packages,
//! including integrity verification and cache cleanup.

#![warn(missing_docs)]

pub mod cache;
pub mod integrity;
pub mod storage;

pub use cache::{Cache, CacheEntry, CacheStats};
pub use integrity::{HashAlgorithm, Integrity};
pub use storage::{Storage, StorageLocation};
