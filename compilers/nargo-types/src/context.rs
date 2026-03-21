//! Context module for Nargo.

use crate::NargoValue;

/// Nargo execution context.
#[derive(Debug, Clone)]
pub struct NargoContext {
    /// Configuration.
    pub config: NargoValue,
}

impl NargoContext {
    /// Create a new context with the given configuration.
    pub fn new(config: NargoValue) -> Self {
        Self { config }
    }
}
