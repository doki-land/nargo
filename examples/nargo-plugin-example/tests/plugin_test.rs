use nargo_plugin::Plugin;
use nargo_plugin_example::*;
use plugins::*;

#[test]
fn test_plugin_name() {
    let logging = LoggingPlugin::new();
    assert_eq!(logging.name(), "logging-plugin");

    let transform = TransformPlugin::new();
    assert_eq!(transform.name(), "transform-plugin");

    let minify = MinifyPlugin::new();
    assert_eq!(minify.name(), "minify-plugin");
}

#[tokio::test]
async fn test_transform_plugin() {
    let plugin = TransformPlugin::new();

    let input = "console.log('test');";
    let result = plugin.on_transform(input);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_some());
    let transformed = result.unwrap();
    assert!(transformed.contains("// Transformed by TransformPlugin"));
}

#[test]
fn test_logging_plugin_name() {
    let plugin = LoggingPlugin::new();
    assert_eq!(plugin.name(), "logging-plugin");
}

#[test]
fn test_transform_plugin_2() {
    let plugin = TransformPlugin::new();
    let input = "test code";
    let result = plugin.on_transform(input);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_some());
    let transformed = result.unwrap();
    assert!(transformed.contains("Transformed by TransformPlugin"));
    assert!(transformed.contains("test code"));
}

#[test]
fn test_minify_plugin() {
    let plugin = MinifyPlugin::new();

    let input = r#"
        // This is a comment
        function   test(   a,   b   )   {
            return   a   +   b;
        }
        "#;

    let result = plugin.on_transform(input);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_some());

    let minified = result.unwrap();
    assert!(!minified.contains("// This is a comment"));
    assert!(!minified.contains("   "));
}

#[test]
fn test_custom_plugin() {
    let plugin = CustomPlugin::new("my-plugin", "My Custom Header");

    assert_eq!(plugin.name(), "my-plugin");

    let input = "test content";
    let result = plugin.on_transform(input);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_some());
    let transformed = result.unwrap();
    assert!(transformed.contains("My Custom Header"));
    assert!(transformed.contains("test content"));
}
