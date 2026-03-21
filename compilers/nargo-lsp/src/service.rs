use crate::features;
use crate::types::NargoLanguage;
use futures::Future;
use nargo_config::NargoConfig;
use nargo_parser::NargoParser;
use nargo_types::NargoContext;
use oak_core::Range;
use oak_lsp::{LanguageService, MemoryVfs, Vfs};
use std::sync::Arc;

/// Language service implementation for Nargo.
///
/// This service provides IDE features like hover information, go-to-definition,
/// completion, references, and document symbols for Nargo source files.
///
/// # Features
///
/// - **Hover**: Shows type information and documentation for symbols
/// - **Go to Definition**: Navigates to symbol definitions
/// - **Completion**: Provides context-aware code completions
/// - **Find References**: Finds all references to a symbol
/// - **Document Symbols**: Shows the document outline
///
/// # Example
///
/// ```ignore
/// use nargo_lsp::NargoService;
///
/// let service = NargoService::new();
/// // Use with LSP server
/// ```
pub struct NargoService {
    /// Virtual file system for managing source files.
    vfs: MemoryVfs,
    /// Workspace manager for project-level features.
    workspace: oak_lsp::workspace::WorkspaceManager,
    /// Parser instance for analyzing source code.
    parser: Arc<NargoParser>,
}

impl NargoService {
    /// Creates a new Nargo language service instance.
    ///
    /// Initializes the service with default configuration and context,
    /// setting up the virtual file system and parser.
    ///
    /// # Returns
    ///
    /// A new `NargoService` instance ready to handle LSP requests.
    pub fn new() -> Self {
        let config = NargoConfig::default();
        let ctx = Arc::new(NargoContext::new(config));
        let parser = Arc::new(NargoParser::new(ctx));

        Self {
            vfs: MemoryVfs::new(),
            workspace: oak_lsp::workspace::WorkspaceManager::new(),
            parser,
        }
    }
}

impl Default for NargoService {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageService for NargoService {
    type Lang = NargoLanguage;
    type Vfs = MemoryVfs;

    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }

    fn workspace(&self) -> &oak_lsp::workspace::WorkspaceManager {
        &self.workspace
    }

    fn initialize(
        &self,
        params: oak_lsp::types::InitializeParams,
    ) -> impl Future<Output = ()> + Send + '_ {
        async move {
            self.workspace.initialize(&params);
        }
    }

    fn hover<'a>(
        &'a self,
        uri: &'a str,
        range: Range<usize>,
    ) -> impl Future<Output = Option<oak_lsp::types::Hover>> + Send + 'a {
        async move {
            let content = self.vfs().get_source(uri)?.text().to_string();
            let ir = self.parser.parse(uri, &content).ok()?;

            let hover_text = features::find_hover_text_in_ir(&ir, range.start);

            Some(oak_lsp::types::Hover {
                contents: hover_text.unwrap_or_else(|| "Nargo Hover: No details found".to_string()),
                range: None,
            })
        }
    }

    fn definition<'a>(
        &'a self,
        uri: &'a str,
        range: Range<usize>,
    ) -> impl Future<Output = Vec<oak_lsp::types::LocationRange>> + Send + 'a {
        async move {
            if let Some(source) = self.vfs().get_source(uri) {
                let content = source.text().to_string();
                if let Ok(ir) = self.parser.parse(uri, &content) {
                    return features::find_definition_in_ir(&ir, uri, range.start);
                }
            }
            vec![]
        }
    }

    fn completion<'a>(
        &'a self,
        uri: &'a str,
        position: usize,
    ) -> impl Future<Output = Vec<oak_lsp::types::CompletionItem>> + Send + 'a {
        async move {
            if let Some(source) = self.vfs().get_source(uri) {
                let content = source.text().to_string();
                if let Ok(ir) = self.parser.parse(uri, &content) {
                    return features::find_completion_in_ir(&ir, position);
                }
            }
            vec![]
        }
    }

    fn references<'a>(
        &'a self,
        uri: &'a str,
        range: Range<usize>,
    ) -> impl Future<Output = Vec<oak_lsp::types::LocationRange>> + Send + 'a {
        async move {
            if let Some(source) = self.vfs().get_source(uri) {
                let content = source.text().to_string();
                if let Ok(ir) = self.parser.parse(uri, &content) {
                    return features::find_references_in_ir(&ir, uri, range.start);
                }
            }
            vec![]
        }
    }

    fn document_symbols<'a>(
        &'a self,
        uri: &'a str,
    ) -> impl Future<Output = Vec<oak_lsp::types::StructureItem>> + Send + 'a {
        async move {
            if let Some(source) = self.vfs().get_source(uri) {
                let content = source.text().to_string();
                if let Ok(ir) = self.parser.parse(uri, &content) {
                    return features::find_document_symbols_in_ir(&ir);
                }
            }
            vec![]
        }
    }
}
