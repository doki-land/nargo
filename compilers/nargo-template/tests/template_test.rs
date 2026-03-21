use nargo_template::{TemplateEngineManager, ToNargoValue, UnifiedTemplateEngine};
use nargo_types::NargoValue;
use std::collections::HashMap;

#[cfg(feature = "dejavu")]
use nargo_template::{DejaVuAdapter, DejaVuFrontend, HtmlAdapter};
#[cfg(feature = "jinja")]
use nargo_template::Jinja2Adapter;
#[cfg(feature = "liquid")]
use nargo_template::LiquidAdapter;

fn make_context() -> NargoValue {
    let mut data = HashMap::new();
    data.insert("name".to_string(), NargoValue::String("World".to_string()));
    NargoValue::Object(data)
}

#[test]
#[cfg(feature = "dejavu")]
fn test_html_adapter() {
    let mut adapter = HtmlAdapter::new();
    adapter.register_template("greeting", "Hello, <% name %>!").unwrap();

    let context = make_context();
    let result = adapter.render("greeting", &context).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
#[cfg(feature = "jinja")]
fn test_jinja2_adapter() {
    let mut adapter = Jinja2Adapter::new();
    adapter.register_template("greeting", "Hello, {{ name }}!").unwrap();

    let context = make_context();
    let result = adapter.render("greeting", &context).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
#[cfg(feature = "liquid")]
fn test_liquid_adapter() {
    let mut adapter = LiquidAdapter::new();
    adapter.register_template("greeting", "Hello, {{ name }}!").unwrap();

    let context = make_context();
    let result = adapter.render("greeting", &context).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
#[cfg(feature = "dejavu")]
fn test_template_engine_manager() {
    let mut manager = TemplateEngineManager::new();
    manager.register_engine("html", Box::new(HtmlAdapter::new()));
    manager.register_engine("dejavu", Box::new(DejaVuAdapter::new(DejaVuFrontend::new())));

    let html_engine = manager.get_engine_mut(Some("html")).unwrap();
    html_engine.register_template("test", "Hello, <% name %>!").unwrap();

    let context = make_context();
    let html_result = manager.render("test", &context, Some("html")).unwrap();
    assert_eq!(html_result, "Hello, World!");
}

#[test]
#[cfg(feature = "dejavu")]
fn test_template_not_found() {
    let mut manager = TemplateEngineManager::new();
    manager.register_engine("html", Box::new(HtmlAdapter::new()));

    let context = make_context();
    let result = manager.render("non_existent", &context, Some("html"));
    assert!(result.is_err());
}

#[test]
fn test_to_nargo_value() {
    let s = "test".to_string();
    assert_eq!(s.to_nargo_value(), NargoValue::String("test".to_string()));

    let i: i64 = 42;
    if let NargoValue::Number(n) = i.to_nargo_value() {
        assert!((n - 42.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected NargoValue::Number");
    }

    let b = true;
    assert_eq!(b.to_nargo_value(), NargoValue::Bool(true));
}

#[test]
#[cfg(feature = "jinja")]
fn test_jinja2_for_loop() {
    let mut adapter = Jinja2Adapter::new();
    adapter.register_template("list", "{% for item in items %}{{ item }}{% endfor %}").unwrap();

    let mut data = HashMap::new();
    data.insert("items".to_string(), NargoValue::Array(vec![
        NargoValue::String("a".to_string()),
        NargoValue::String("b".to_string()),
        NargoValue::String("c".to_string()),
    ]));
    let context = NargoValue::Object(data);

    let result = adapter.render("list", &context).unwrap();
    assert_eq!(result, "abc");
}

#[test]
#[cfg(feature = "jinja")]
fn test_jinja2_if_else() {
    let mut adapter = Jinja2Adapter::new();
    adapter.register_template("cond", "{% if show %}yes{% else %}no{% endif %}").unwrap();

    let mut data = HashMap::new();
    data.insert("show".to_string(), NargoValue::Bool(true));
    let context = NargoValue::Object(data);

    let result = adapter.render("cond", &context).unwrap();
    assert_eq!(result, "yes");

    let mut data2 = HashMap::new();
    data2.insert("show".to_string(), NargoValue::Bool(false));
    let context2 = NargoValue::Object(data2);

    let result2 = adapter.render("cond", &context2).unwrap();
    assert_eq!(result2, "no");
}

#[test]
#[cfg(feature = "jinja")]
fn test_jinja2_nested_object() {
    let mut adapter = Jinja2Adapter::new();
    adapter.register_template("nested", "{{ user.name }}").unwrap();

    let mut user = HashMap::new();
    user.insert("name".to_string(), NargoValue::String("Alice".to_string()));
    let mut data = HashMap::new();
    data.insert("user".to_string(), NargoValue::Object(user));
    let context = NargoValue::Object(data);

    let result = adapter.render("nested", &context).unwrap();
    assert_eq!(result, "Alice");
}

#[test]
#[cfg(feature = "liquid")]
fn test_liquid_for_loop() {
    let mut adapter = LiquidAdapter::new();
    adapter.register_template("list", "{% for item in items %}{{ item }}{% endfor %}").unwrap();

    let mut data = HashMap::new();
    data.insert("items".to_string(), NargoValue::Array(vec![
        NargoValue::String("x".to_string()),
        NargoValue::String("y".to_string()),
    ]));
    let context = NargoValue::Object(data);

    let result = adapter.render("list", &context).unwrap();
    assert_eq!(result, "xy");
}

#[test]
#[cfg(feature = "liquid")]
fn test_liquid_if_condition() {
    let mut adapter = LiquidAdapter::new();
    adapter.register_template("cond", "{% if active %}on{% else %}off{% endif %}").unwrap();

    let mut data = HashMap::new();
    data.insert("active".to_string(), NargoValue::Bool(true));
    let context = NargoValue::Object(data);

    let result = adapter.render("cond", &context).unwrap();
    assert_eq!(result, "on");
}

#[test]
#[cfg(feature = "dejavu")]
fn test_dejavu_variable_interpolation() {
    let mut adapter = DejaVuAdapter::new(DejaVuFrontend::new());
    adapter.register_template("greeting", "Hello, <% name %>!").unwrap();

    let context = make_context();
    let result = adapter.render("greeting", &context).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
#[cfg(feature = "dejavu")]
fn test_dejavu_if_else() {
    let mut adapter = DejaVuAdapter::new(DejaVuFrontend::new());
    adapter.register_template("cond", "<% if show %>yes<% else %>no<% end if %>").unwrap();

    let mut data_true = HashMap::new();
    data_true.insert("show".to_string(), NargoValue::Bool(true));
    let context_true = NargoValue::Object(data_true);
    let result_true = adapter.render("cond", &context_true).unwrap();
    assert_eq!(result_true, "yes");

    let mut data_false = HashMap::new();
    data_false.insert("show".to_string(), NargoValue::Bool(false));
    let context_false = NargoValue::Object(data_false);
    let result_false = adapter.render("cond", &context_false).unwrap();
    assert_eq!(result_false, "no");
}
