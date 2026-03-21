use nargo_template::TemplateEngineManager;
use nargo_types::NargoValue;
use std::collections::HashMap;

#[cfg(feature = "dejavu")]
use nargo_template::{DejaVuAdapter, HtmlAdapter};

fn main() {
    let mut manager = TemplateEngineManager::new();

    #[cfg(feature = "dejavu")]
    {
        manager.register_engine("html", Box::new(HtmlAdapter::new()));
        manager.register_engine("dejavu", Box::new(DejaVuAdapter::new()));

        let html_engine = manager.get_engine_mut(Some("html")).unwrap();
        html_engine.register_template("greeting", "Hello, <% name %>!").unwrap();

        let mut data = HashMap::new();
        data.insert("name".to_string(), NargoValue::String("World".to_string()));
        let context = NargoValue::Object(data);

        let result = manager.render("greeting", &context, Some("html")).unwrap();
        println!("{}", result);
    }

    #[cfg(not(feature = "dejavu"))]
    {
        println!("No template engine available. Enable a feature flag like --features dejavu");
    }
}
