//! Completion feature.

use nargo_types::IRModule;

/// Completion item.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// Label.
    pub label: String,
    /// Kind.
    pub kind: CompletionItemKind,
}

/// Completion item kind.
#[derive(Debug, Clone, Copy)]
pub enum CompletionItemKind {
    /// Text.
    Text,
    /// Method.
    Method,
    /// Function.
    Function,
    /// Variable.
    Variable,
}

/// Find completions.
pub fn find_completions(_ir: &IRModule, _offset: usize) -> Vec<CompletionItem> {
    Vec::new()
}
