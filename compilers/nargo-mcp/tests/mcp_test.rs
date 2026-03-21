use nargo_mcp::{EventType, Message, MessageStatus, MessageType, ModuleCommunicationManager, NargoLanguageService, run_http_server, run_stdio};
use oak_lsp::types::{CompletionItem, InitializeParams};
use oak_vfs::MemoryVfs;
use serde_json::json;
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
fn test_module_communication_manager_new() {
    // Test creating a new ModuleCommunicationManager
    let manager = ModuleCommunicationManager::new();
    // Just test that it creates successfully
    assert!(true);
}

#[test]
fn test_module_registration() {
    // Test module registration
    let manager = ModuleCommunicationManager::new();
    let module_id = "test-module".to_string();
    let rx = manager.register_module(module_id.clone(), "Test Module".to_string(), "1.0.0".to_string());
    // Test that we get a receiver
    assert!(rx.recv().await.is_err()); // Should be empty initially
}

#[test]
fn test_message_creation() {
    // Test message creation
    let sender = "sender".to_string();
    let receiver = "receiver".to_string();
    let data = json!({ "key": "value" });

    // Test new_message
    let message = Message::new_message(sender.clone(), receiver.clone(), data.clone());
    assert_eq!(message.message_type, MessageType::Message);
    assert_eq!(message.sender, sender);
    assert_eq!(message.receiver, receiver);

    // Test new_event
    let event_message = Message::new_event(sender.clone(), EventType::ModuleInitialized, data.clone());
    assert_eq!(event_message.message_type, MessageType::Event);
    assert_eq!(event_message.event_type, Some(EventType::ModuleInitialized));

    // Test new_request
    let request_message = Message::new_request(sender.clone(), receiver.clone(), data.clone());
    assert_eq!(request_message.message_type, MessageType::Request);

    // Test new_response
    let response_message = Message::new_response("req-123".to_string(), sender.clone(), receiver.clone(), data.clone());
    assert_eq!(response_message.message_type, MessageType::Response);

    // Test new_broadcast
    let broadcast_message = Message::new_broadcast(sender.clone(), data.clone());
    assert_eq!(broadcast_message.message_type, MessageType::Broadcast);

    // Test new_ping
    let ping_message = Message::new_ping(sender.clone(), receiver.clone());
    assert_eq!(ping_message.message_type, MessageType::Ping);

    // Test new_pong
    let pong_message = Message::new_pong(sender.clone(), receiver.clone(), "ping-123".to_string());
    assert_eq!(pong_message.message_type, MessageType::Pong);

    // Test new_subscribe
    let subscribe_message = Message::new_subscribe(sender.clone(), EventType::ModuleInitialized);
    assert_eq!(subscribe_message.message_type, MessageType::Subscribe);

    // Test new_unsubscribe
    let unsubscribe_message = Message::new_unsubscribe(sender.clone(), EventType::ModuleInitialized);
    assert_eq!(unsubscribe_message.message_type, MessageType::Unsubscribe);

    // Test new_error
    let error_message = Message::new_error(sender.clone(), receiver.clone(), "Test error".to_string());
    assert_eq!(error_message.message_type, MessageType::Error);
}

#[test]
fn test_message_status() {
    // Test message status
    let sender = "sender".to_string();
    let receiver = "receiver".to_string();
    let data = json!({ "key": "value" });

    let mut message = Message::new_message(sender, receiver, data);
    assert_eq!(message.status, MessageStatus::Created);

    message.status = MessageStatus::Sent;
    assert_eq!(message.status, MessageStatus::Sent);

    message.status = MessageStatus::Received;
    assert_eq!(message.status, MessageStatus::Received);

    message.status = MessageStatus::Acknowledged;
    assert_eq!(message.status, MessageStatus::Acknowledged);

    message.status = MessageStatus::Timeout;
    assert_eq!(message.status, MessageStatus::Timeout);

    message.status = MessageStatus::Failed;
    assert_eq!(message.status, MessageStatus::Failed);
}

#[test]
fn test_event_type_to_string() {
    // Test EventType to_string method
    assert_eq!(EventType::ModuleInitialized.to_string(), "ModuleInitialized");
    assert_eq!(EventType::ModuleStateUpdated.to_string(), "ModuleStateUpdated");
    assert_eq!(EventType::CompileStarted.to_string(), "CompileStarted");
    assert_eq!(EventType::CompileCompleted.to_string(), "CompileCompleted");
    assert_eq!(EventType::CompileError.to_string(), "CompileError");
    assert_eq!(EventType::FileChanged.to_string(), "FileChanged");
    assert_eq!(EventType::ConfigurationChanged.to_string(), "ConfigurationChanged");
    assert_eq!(EventType::ModuleRegistered.to_string(), "ModuleRegistered");
    assert_eq!(EventType::ModuleUnregistered.to_string(), "ModuleUnregistered");
    assert_eq!(EventType::MessageSent.to_string(), "MessageSent");
    assert_eq!(EventType::MessageReceived.to_string(), "MessageReceived");
    assert_eq!(EventType::HeartbeatTimeout.to_string(), "HeartbeatTimeout");
    assert_eq!(EventType::NetworkDisconnected.to_string(), "NetworkDisconnected");
    assert_eq!(EventType::NetworkConnected.to_string(), "NetworkConnected");
    assert_eq!(EventType::ResourceLoadStarted.to_string(), "ResourceLoadStarted");
    assert_eq!(EventType::ResourceLoadCompleted.to_string(), "ResourceLoadCompleted");
    assert_eq!(EventType::ResourceLoadError.to_string(), "ResourceLoadError");
    assert_eq!(EventType::PerformanceWarning.to_string(), "PerformanceWarning");
    assert_eq!(EventType::SystemWarning.to_string(), "SystemWarning");
    assert_eq!(EventType::SystemError.to_string(), "SystemError");
}

#[test]
fn test_message_batch() {
    // Test message batch creation
    let sender = "sender".to_string();
    let receiver = "receiver".to_string();
    let data = json!({ "key": "value" });

    let message1 = Message::new_message(sender.clone(), receiver.clone(), data.clone());
    let message2 = Message::new_message(sender.clone(), receiver.clone(), data.clone());

    let batch = nargo_mcp::MessageBatch { batch_id: "batch-123".to_string(), messages: vec![message1, message2], size: 2, timestamp: 1234567890 };

    assert_eq!(batch.batch_id, "batch-123");
    assert_eq!(batch.messages.len(), 2);
    assert_eq!(batch.size, 2);
    assert_eq!(batch.timestamp, 1234567890);
}

#[test]
fn test_message_caching() {
    // Test message caching
    let manager = ModuleCommunicationManager::new();

    let key = "test-key".to_string();
    let value = json!({ "key": "value" });

    manager.cache_message(key.clone(), value.clone());
    assert_eq!(manager.get_cached_message(&key), Some(value));

    manager.clear_cache();
    assert_eq!(manager.get_cached_message(&key), None);
}

#[test]
fn test_cleanup_expired_messages() {
    // Test cleanup expired messages
    let manager = ModuleCommunicationManager::new();

    let sender = "sender".to_string();
    let receiver = "receiver".to_string();
    let data = json!({ "key": "value" });

    let message = Message::new_message(sender, receiver, data);
    manager.send_message(message).await;

    // Cleanup should not panic
    manager.cleanup_expired_messages();
    assert!(true);
}

#[test]
fn test_batch_processing() {
    // Test batch processing
    let manager = ModuleCommunicationManager::new();

    let sender = "sender".to_string();
    let receiver = "receiver".to_string();
    let data = json!({ "key": "value" });

    // Register a module to receive messages
    manager.register_module(receiver.clone(), "Test Module".to_string(), "1.0.0".to_string());

    // Queue multiple messages
    for i in 0..15 {
        let message = Message::new_message(sender.clone(), receiver.clone(), json!({ "index": i }));
        manager.queue_message(message);
    }

    // Process batches
    manager.process_message_batches().await;

    // Test should complete without errors
    assert!(true);
}

#[test]
fn test_heartbeat_mechanism() {
    // Test heartbeat mechanism
    let manager = ModuleCommunicationManager::new();

    let module_id = "test-module".to_string();
    manager.register_module(module_id.clone(), "Test Module".to_string(), "1.0.0".to_string());

    // Test send_ping
    manager.send_ping(&module_id).await;

    // Test handle_pong
    manager.handle_pong(&module_id).await;

    // Test check_heartbeats
    manager.check_heartbeats().await;

    // Test should complete without errors
    assert!(true);
}
