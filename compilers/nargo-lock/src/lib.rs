//! Lock file management for Nargo package manager.
//!
//! This crate provides lock file generation, parsing, and verification
//! to ensure reproducible builds.

#![warn(missing_docs)]

pub mod lock;
pub mod verify;

pub use lock::{LockEntry, LockFile, LockMetadata};
pub use verify::{LockDiff, Verifier, VerifyResult};
