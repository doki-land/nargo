use nargo_template::{TemplateEngineManager, UnifiedTemplateEngine};
use nargo_types::NargoValue;
use std::collections::HashMap;

#[cfg(feature = "dejavu")]
use nargo_template::{DejaVuAdapter, DejaVuFrontend, HtmlAdapter};
#[cfg(feature = "jinja")]
use nargo_template::Jinja2Adapter;
#[cfg(feature = "liquid")]
use nargo_template::LiquidAdapter;

fn make_context(name: &str) -> NargoValue {
    let mut data = HashMap::new();
    data.insert("name".to_string(), NargoValue::String(name.to_string()));
    NargoValue::Object(data)
}

#[test]
#[cfg(all(feature = "dejavu", feature = "jinja", feature = "liquid"))]
fn test_html_jinja_liquid_adapters() {
    let mut manager = TemplateEngineManager::new();

    manager.register_engine("html", Box::new(HtmlAdapter::new()));
    manager.register_engine("jinja2", Box::new(Jinja2Adapter::new()));
    manager.register_engine("liquid", Box::new(LiquidAdapter::new()));

    let html_engine = manager.get_engine_mut(Some("html")).unwrap();
    html_engine.register_template("test", "Hello, <% name %>!").unwrap();

    let jinja_engine = manager.get_engine_mut(Some("jinja2")).unwrap();
    jinja_engine.register_template("test", "Hello, {{ name }}!").unwrap();

    let liquid_engine = manager.get_engine_mut(Some("liquid")).unwrap();
    liquid_engine.register_template("test", "Hello, {{ name }}!").unwrap();

    let context = make_context("World");

    let html_result = manager.render("test", &context, Some("html")).unwrap();
    assert_eq!(html_result, "Hello, World!");

    let jinja_result = manager.render("test", &context, Some("jinja2")).unwrap();
    assert_eq!(jinja_result, "Hello, World!");

    let liquid_result = manager.render("test", &context, Some("liquid")).unwrap();
    assert_eq!(liquid_result, "Hello, World!");
}

#[test]
#[cfg(feature = "dejavu")]
fn test_dejavu_adapter_register() {
    let mut adapter = DejaVuAdapter::new(DejaVuFrontend::new());
    adapter.register_template("hello", "Hello World").unwrap();
    let context = NargoValue::Object(HashMap::new());
    let result = adapter.render("hello", &context).unwrap();
    assert_eq!(result, "Hello World");
}

#[test]
#[cfg(feature = "dejavu")]
fn test_template_not_found() {
    let mut manager = TemplateEngineManager::new();

    manager.register_engine("html", Box::new(HtmlAdapter::new()));

    let context = make_context("World");

    let result = manager.render("non_existent", &context, Some("html"));
    assert!(result.is_err());
}
