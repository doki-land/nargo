//! Hover feature.

use nargo_types::IRModule;

/// Hover result.
#[derive(Debug, Clone)]
pub struct Hover {
    /// Contents.
    pub contents: String,
}

/// Find hover.
pub fn find_hover(_ir: &IRModule, _offset: usize) -> Option<Hover> {
    None
}
