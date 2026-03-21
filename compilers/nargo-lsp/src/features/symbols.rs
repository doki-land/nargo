//! Symbols feature.

use nargo_types::IRModule;

/// Symbol kind.
#[derive(Debug, Clone, Copy)]
pub enum SymbolKind {
    /// File.
    File,
    /// Module.
    Module,
    /// Function.
    Function,
    /// Variable.
    Variable,
}

/// Structure item.
#[derive(Debug, Clone)]
pub struct StructureItem {
    /// Name.
    pub name: String,
    /// Kind.
    pub kind: SymbolKind,
    /// Children.
    pub children: Vec<StructureItem>,
}

/// Find symbols.
pub fn find_symbols(_ir: &IRModule) -> Vec<StructureItem> {
    Vec::new()
}
