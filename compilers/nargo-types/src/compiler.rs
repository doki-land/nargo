//! Compiler module for Nargo.

use crate::errors::Result;
use std::path::Path;

/// Nargo compiler.
pub struct NargoCompiler;

impl NargoCompiler {
    /// Create a new compiler instance.
    pub fn new() -> Self {
        Self
    }

    /// Compile source code.
    pub fn compile(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }
}

impl Default for NargoCompiler {
    fn default() -> Self {
        Self::new()
    }
}
