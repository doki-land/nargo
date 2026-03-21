use crate::ParseState;
use nargo_ir::{JsProgram, TemplateNodeIR};
use nargo_types::{NargoValue, Result};
use std::{collections::HashMap, sync::Arc};

pub trait TemplateParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<Vec<TemplateNodeIR>>;
}

pub trait ScriptParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<JsProgram>;
}

pub trait StyleParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<(String, nargo_ir::Trivia)>;
}

pub trait MetadataParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<(NargoValue, nargo_ir::Trivia)>;
}

#[derive(Default)]
pub struct ParserRegistry {
    template_parsers: HashMap<String, Arc<dyn TemplateParser>>,
    script_parsers: HashMap<String, Arc<dyn ScriptParser>>,
    style_parsers: HashMap<String, Arc<dyn StyleParser>>,
    metadata_parsers: HashMap<String, Arc<dyn MetadataParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();

        // Register default template parser
        registry.register_template_parser("nargo", std::sync::Arc::new(crate::oak_template_parser::OakVueTemplateParser));

        // Register default script parser
        registry.register_script_parser("typescript", std::sync::Arc::new(crate::oak_script_parser::OakTypeScriptParser));
        registry.register_script_parser("ts", std::sync::Arc::new(crate::oak_script_parser::OakTypeScriptParser));
        registry.register_script_parser("javascript", std::sync::Arc::new(crate::oak_script_parser::OakTypeScriptParser));
        registry.register_script_parser("js", std::sync::Arc::new(crate::oak_script_parser::OakTypeScriptParser));

        // Register default style parser
        registry.register_style_parser("css", std::sync::Arc::new(crate::oak_style_parser::OakCssParser));
        registry.register_style_parser("scss", std::sync::Arc::new(crate::oak_style_parser::OakScssParser));
        registry.register_style_parser("sass", std::sync::Arc::new(crate::oak_style_parser::OakScssParser));

        // Register default metadata parser for JSON
        // registry.register_metadata_parser("json", std::sync::Arc::new(JSONMetadataParser));

        registry
    }

    pub fn register_template_parser(&mut self, lang: &str, parser: Arc<dyn TemplateParser>) {
        self.template_parsers.insert(lang.to_string(), parser);
    }

    pub fn register_script_parser(&mut self, lang: &str, parser: Arc<dyn ScriptParser>) {
        self.script_parsers.insert(lang.to_string(), parser);
    }

    pub fn register_style_parser(&mut self, lang: &str, parser: Arc<dyn StyleParser>) {
        self.style_parsers.insert(lang.to_string(), parser);
    }

    pub fn register_metadata_parser(&mut self, lang: &str, parser: Arc<dyn MetadataParser>) {
        self.metadata_parsers.insert(lang.to_string(), parser);
    }

    pub fn get_template_parser(&self, lang: &str) -> Option<Arc<dyn TemplateParser>> {
        self.template_parsers.get(lang).cloned()
    }

    pub fn get_script_parser(&self, lang: &str) -> Option<Arc<dyn ScriptParser>> {
        self.script_parsers.get(lang).cloned()
    }

    pub fn get_style_parser(&self, lang: &str) -> Option<Arc<dyn StyleParser>> {
        self.style_parsers.get(lang).cloned()
    }

    pub fn get_metadata_parser(&self, lang: &str) -> Option<Arc<dyn MetadataParser>> {
        self.metadata_parsers.get(lang).cloned()
    }
}
