use nargo_plugin::*;
use nargo_types::{NargoContext, Result};
use std::sync::Arc;

struct LoggingPlugin {
    name: String,
    logs: std::sync::Mutex<Vec<String>>,
}

impl LoggingPlugin {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), logs: std::sync::Mutex::new(Vec::new()) }
    }

    fn get_logs(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "A plugin that logs all lifecycle events"
    }

    fn on_init(&self, _ctx: Arc<NargoContext>, _config: &PluginConfig) -> Result<()> {
        self.logs.lock().unwrap().push("on_init".to_string());
        Ok(())
    }

    fn on_pre_parse(&self, _source: &str) -> Result<Option<String>> {
        self.logs.lock().unwrap().push("on_pre_parse".to_string());
        Ok(None)
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        self.logs.lock().unwrap().push("on_transform".to_string());
        Ok(Some(format!("// Logged by {}\n{}", self.name, code)))
    }

    fn on_cleanup(&self) -> Result<()> {
        self.logs.lock().unwrap().push("on_cleanup".to_string());
        Ok(())
    }
}

struct PriorityPlugin {
    name: String,
    priority: i32,
}

impl PriorityPlugin {
    fn new(name: &str, priority: i32) -> Self {
        Self { name: name.to_string(), priority }
    }
}

impl Plugin for PriorityPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        Ok(Some(format!("// {} (priority {})\n{}", self.name, self.priority, code)))
    }
}

#[test]
fn test_plugin_lifecycle() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = LoggingPlugin::new("logging-plugin");
    manager.register(Box::new(plugin));

    assert!(manager.init_all().is_ok());
    assert!(manager.cleanup_all().is_ok());
}

#[tokio::test]
async fn test_plugin_chain() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin1 = PriorityPlugin::new("plugin-1", 10);
    let plugin2 = PriorityPlugin::new("plugin-2", 1);
    let plugin3 = PriorityPlugin::new("plugin-3", 5);

    manager.register(Box::new(plugin1));
    manager.register(Box::new(plugin2));
    manager.register(Box::new(plugin3));

    let input = "console.log('test');";
    let result = manager.transform(input.to_string()).await;

    assert!(result.is_ok());
    let transformed = result.unwrap();

    // Check that plugins are applied in priority order
    // Priority 1 (plugin-2) should run first, then 5 (plugin-3), then 10 (plugin-1)
    let plugin2_pos = transformed.find("plugin-2").unwrap();
    let plugin3_pos = transformed.find("plugin-3").unwrap();
    let plugin1_pos = transformed.find("plugin-1").unwrap();

    // Higher priority (lower number) plugins apply first,
    // so their comments appear later in the output
    assert!(plugin2_pos > plugin3_pos);
    assert!(plugin3_pos > plugin1_pos);
}

#[test]
fn test_plugin_config() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = LoggingPlugin::new("test-config-plugin");
    let mut config = PluginConfig::default();
    config.name = "test-config-plugin".to_string();
    config.version = "2.0.0".to_string();
    config.description = "Test plugin with custom config".to_string();
    config.author = Some("Test Author".to_string());
    config.homepage = Some("https://example.com".to_string());
    config.priority = 100;
    config.enabled = true;

    let mut custom_config = std::collections::HashMap::new();
    custom_config.insert("key1".to_string(), nargo_types::NargoValue::String("value1".to_string()));
    custom_config.insert("key2".to_string(), nargo_types::NargoValue::Number(42.0));
    config.config = custom_config;

    manager.register_with_config(Box::new(plugin), config);

    let info = manager.get_plugins_info();
    assert_eq!(info.len(), 1);
    let (name, cfg) = info[0];
    assert_eq!(name, "test-config-plugin");
    assert_eq!(cfg.version, "2.0.0");
    assert_eq!(cfg.description, "Test plugin with custom config");
    assert_eq!(cfg.author, Some("Test Author".to_string()));
    assert_eq!(cfg.homepage, Some("https://example.com".to_string()));
    assert_eq!(cfg.priority, 100);
    assert!(cfg.enabled);
    assert_eq!(cfg.config.len(), 2);
}

#[tokio::test]
async fn test_disabled_plugin() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let enabled_plugin = PriorityPlugin::new("enabled", 1);
    let disabled_plugin = PriorityPlugin::new("disabled", 2);

    manager.register(Box::new(enabled_plugin));
    manager.register(Box::new(disabled_plugin));

    manager.set_plugin_enabled("disabled", false).unwrap();

    let input = "test";
    let result = manager.transform(input.to_string()).await.unwrap();

    assert!(result.contains("enabled"));
    assert!(!result.contains("disabled"));
}

#[test]
fn test_plugin_metadata() {
    struct MetadataPlugin;

    impl Plugin for MetadataPlugin {
        fn name(&self) -> &str {
            "metadata-plugin"
        }

        fn version(&self) -> &str {
            "1.2.3"
        }

        fn description(&self) -> &str {
            "A plugin with metadata"
        }

        fn author(&self) -> Option<&str> {
            Some("Jane Doe")
        }

        fn homepage(&self) -> Option<&str> {
            Some("https://metadata-plugin.example")
        }

        fn priority(&self) -> i32 {
            42
        }
    }

    let plugin = MetadataPlugin;
    assert_eq!(plugin.name(), "metadata-plugin");
    assert_eq!(plugin.version(), "1.2.3");
    assert_eq!(plugin.description(), "A plugin with metadata");
    assert_eq!(plugin.author(), Some("Jane Doe"));
    assert_eq!(plugin.homepage(), Some("https://metadata-plugin.example"));
    assert_eq!(plugin.priority(), 42);
}

#[tokio::test]
async fn test_parse_phase() {
    struct ParsePlugin {
        called: std::sync::atomic::AtomicBool,
    }

    impl ParsePlugin {
        fn new() -> Self {
            Self { called: std::sync::atomic::AtomicBool::new(false) }
        }
    }

    impl Plugin for ParsePlugin {
        fn name(&self) -> &str {
            "parse-plugin"
        }

        fn on_pre_parse(&self, source: &str) -> Result<Option<String>> {
            self.called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(format!("// Pre-parsed\n{}", source)))
        }
    }

    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = ParsePlugin::new();
    manager.register(Box::new(plugin));

    let input = "source code";
    let result = manager.parse(input.to_string()).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(parsed.contains("// Pre-parsed"));
    assert!(parsed.contains("source code"));
}

#[tokio::test]
async fn test_bundle_phase() {
    struct BundlePlugin;

    impl Plugin for BundlePlugin {
        fn name(&self) -> &str {
            "bundle-plugin"
        }

        fn on_bundle(&self, bundle: &str) -> Result<Option<String>> {
            Ok(Some(format!("// Bundled\n{}", bundle)))
        }
    }

    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    manager.register(Box::new(BundlePlugin));

    let input = "bundle content";
    let result = manager.bundle(input.to_string()).await;

    assert!(result.is_ok());
    let bundled = result.unwrap();
    assert!(bundled.contains("// Bundled"));
    assert!(bundled.contains("bundle content"));
}

#[test]
fn test_get_plugin() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin1 = PriorityPlugin::new("plugin-a", 1);
    let plugin2 = PriorityPlugin::new("plugin-b", 2);

    manager.register(Box::new(plugin1));
    manager.register(Box::new(plugin2));

    assert!(manager.get_plugin("plugin-a").is_some());
    assert!(manager.get_plugin("plugin-b").is_some());
    assert!(manager.get_plugin("non-existent").is_none());
}

struct TestPlugin {
    name: String,
    transform_called: std::sync::atomic::AtomicBool,
}

impl TestPlugin {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), transform_called: std::sync::atomic::AtomicBool::new(false) }
    }
}

impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        self.transform_called.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(Some(format!("// Transformed by {}\n{}", self.name, code)))
    }
}

#[test]
fn test_plugin_manager_creation() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let manager = PluginManager::new(ctx);
    assert!(manager.plugins().is_empty());
}

#[test]
fn test_plugin_registration() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = TestPlugin::new("test-plugin");
    manager.register(Box::new(plugin));

    assert_eq!(manager.plugins().len(), 1);
    assert!(manager.get_plugin("test-plugin").is_some());
}

#[tokio::test]
async fn test_plugin_transform() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = TestPlugin::new("test-plugin");
    manager.register(Box::new(plugin));

    let input = "console.log('test');";
    let result = manager.transform(input.to_string()).await;

    assert!(result.is_ok());
    let transformed = result.unwrap();
    assert!(transformed.contains("// Transformed by test-plugin"));
    assert!(transformed.contains("console.log('test');"));
}

#[test]
fn test_plugin_config() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = TestPlugin::new("test-plugin");
    let mut config = PluginConfig::default();
    config.name = "test-plugin".to_string();
    config.priority = 10;
    config.enabled = false;

    manager.register_with_config(Box::new(plugin), config);

    let info = manager.get_plugins_info();
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].0, "test-plugin");
    assert!(!info[0].1.enabled);
}

#[test]
fn test_plugin_enable_disable() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = TestPlugin::new("test-plugin");
    manager.register(Box::new(plugin));

    assert!(manager.get_config("test-plugin").unwrap().enabled);

    let result = manager.set_plugin_enabled("test-plugin", false);
    assert!(result.is_ok());
    assert!(!manager.get_config("test-plugin").unwrap().enabled);

    let result = manager.set_plugin_enabled("test-plugin", true);
    assert!(result.is_ok());
    assert!(manager.get_config("test-plugin").unwrap().enabled);
}
