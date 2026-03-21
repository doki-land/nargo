use nargo_lsp::{NargoLanguageService, run_http_server, run_stdio};
use oak_lsp::types::{CompletionItem, InitializeParams};
use oak_vfs::MemoryVfs;
use std::sync::Arc;

#[test]
fn test_nargo_language_service_new() {
    // Test creating a new NargoLanguageService
    let service = NargoLanguageService::new();
    assert!(service.vfs().is_empty());
}

#[test]
fn test_nargo_language_service_vfs() {
    // Test VFS functionality
    let service = NargoLanguageService::new();
    let vfs = service.vfs();
    assert!(vfs.is_empty());
}

#[test]
fn test_nargo_language_service_workspace() {
    // Test workspace functionality
    let service = NargoLanguageService::new();
    let workspace = service.workspace();
    // Just test that we can get the workspace
    assert!(workspace.list_folders().is_empty());
}

#[test]
fn test_run_stdio() {
    // Test that run_stdio doesn't panic
    // Note: This is a non-blocking test since run_stdio is async
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // This will not actually run since it expects stdio input
        // We just test that it doesn't panic when called
        let result = run_stdio().await;
        // We expect this to fail since we're not providing stdio input
        assert!(result.is_err());
    });
}

#[test]
fn test_run_http_server() {
    // Test that run_http_server doesn't panic
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // This will not actually run a server
        // We just test that it doesn't panic when called
        let result = run_http_server(8080).await;
        // We expect this to fail since it's not fully implemented
        assert!(result.is_err());
    });
}

#[test]
fn test_initialization() {
    // Test initialization
    let service = Arc::new(NargoLanguageService::new());
    let params = InitializeParams { process_id: None, root_uri: None, root_path: None, initialization_options: None, capabilities: Default::default(), client_info: None, locale: None, workspace_folders: None, trace: None };

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        service.initialize(params).await;
        // Just test that initialization doesn't panic
    });
}

#[test]
fn test_completion() {
    // Test completion functionality
    let service = Arc::new(NargoLanguageService::new());

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // This will return an empty list since there's no content
        let items: Vec<CompletionItem> = service.completion("file:///test.ts", 0).await;
        assert!(!items.is_empty());
    });
}

#[test]
fn test_diagnostics() {
    // Test diagnostics functionality
    let service = Arc::new(NargoLanguageService::new());

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // This will return an empty list since there's no content
        let diags = service.diagnostics("file:///test.ts").await;
        assert!(diags.is_empty());
    });
}

#[test]
fn test_definition() {
    // Test definition functionality
    let service = Arc::new(NargoLanguageService::new());

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // This will return an empty list since there's no content
        let locs = service.definition("file:///test.ts", oak_core::Range { start: 0, end: 0 }).await;
        assert!(locs.is_empty());
    });
}

#[test]
fn test_semantic_tokens() {
    // Test semantic tokens functionality
    let service = Arc::new(NargoLanguageService::new());

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // This will return None since there's no content
        let tokens = service.semantic_tokens("file:///test.ts").await;
        assert!(tokens.is_none());
    });
}
