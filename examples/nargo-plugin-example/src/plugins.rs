use nargo_plugin::Plugin;
use nargo_types::{NargoContext, Result};
use std::sync::Arc;

pub struct LoggingPlugin {
    name: String,
}

impl LoggingPlugin {
    pub fn new() -> Self {
        Self { name: "logging-plugin".to_string() }
    }
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_init(&self, _ctx: Arc<NargoContext>) -> Result<()> {
        tracing::info!("LoggingPlugin initialized");
        Ok(())
    }

    fn on_parse(&self, source: &str) -> Result<Option<String>> {
        tracing::debug!("Parsing source: {} bytes", source.len());
        Ok(None)
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        tracing::debug!("Transforming code: {} bytes", code.len());
        Ok(None)
    }

    fn on_bundle(&self, bundle: &str) -> Result<Option<String>> {
        tracing::debug!("Bundling: {} bytes", bundle.len());
        Ok(None)
    }
}

pub struct TransformPlugin {
    name: String,
}

impl TransformPlugin {
    pub fn new() -> Self {
        Self { name: "transform-plugin".to_string() }
    }
}

impl Plugin for TransformPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let transformed = format!("// Transformed by TransformPlugin\n{}", code);
        Ok(Some(transformed))
    }
}

pub struct MinifyPlugin {
    name: String,
}

impl MinifyPlugin {
    pub fn new() -> Self {
        Self { name: "minify-plugin".to_string() }
    }

    fn minify_code(&self, code: &str) -> String {
        code.lines().map(|line| line.trim()).filter(|line| !line.is_empty() && !line.starts_with("//")).collect::<Vec<_>>().join(" ").replace("  ", " ").replace("( ", "(").replace(" )", ")").replace("{ ", "{").replace(" }", "}").replace("; ", ";").replace(" ,", ",").replace(" + ", "+").replace(" = ", "=")
    }
}

impl Plugin for MinifyPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let minified = self.minify_code(code);
        Ok(Some(minified))
    }
}

pub struct CustomPlugin {
    name: String,
    prefix: String,
}

impl CustomPlugin {
    pub fn new(name: &str, prefix: &str) -> Self {
        Self { name: name.to_string(), prefix: prefix.to_string() }
    }
}

impl Plugin for CustomPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_init(&self, _ctx: Arc<NargoContext>) -> Result<()> {
        println!("CustomPlugin '{}' initialized with prefix: {}", self.name, self.prefix);
        Ok(())
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let transformed = format!("/* {} */\n{}", self.prefix, code);
        Ok(Some(transformed))
    }
}
