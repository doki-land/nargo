use crate::{ParseState, ParserRegistry};
use nargo_ir::{CustomBlockIR, IRModule, StyleIR, TemplateIR};
use nargo_types::{Cursor, Result, Span};
use std::{collections::HashMap, sync::Arc};

pub struct VocShell<'a> {
    name: String,
    state: ParseState<'a>,
    registry: Arc<ParserRegistry>,
}

impl<'a> VocShell<'a> {
    pub fn new(name: String, source: &'a str, registry: Arc<ParserRegistry>) -> Self {
        Self { name, state: ParseState::new(source), registry }
    }

    pub fn parse_all(&mut self) -> Result<IRModule> {
        self.parse_module()
    }

    pub fn parse_module(&mut self) -> Result<IRModule> {
        let mut template_nodes = Vec::new();
        let mut scripts = Vec::new();
        let mut scripts_server = Vec::new();
        let mut scripts_client = Vec::new();
        let mut ir_styles = Vec::new();
        let mut metadata = HashMap::new();
        let mut custom_blocks = Vec::new();

        while !self.state.cursor.is_eof() {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.is_eof() {
                break;
            }

            if self.state.cursor.peek_str("<template") {
                self.parse_template_block(&mut template_nodes)?;
            }
            else if self.state.cursor.peek_str("<script") {
                self.parse_script_block(&mut scripts, &mut scripts_server, &mut scripts_client)?;
            }
            else if self.state.cursor.peek_str("<style") {
                self.parse_style_block(&mut ir_styles)?;
            }
            else if self.state.cursor.peek_str("<metadata") {
                self.parse_metadata_block(&mut metadata)?;
            }
            else if self.state.cursor.peek_str("<") {
                self.parse_custom_block(&mut custom_blocks)?;
            }
            else {
                self.state.cursor.consume();
            }
        }

        let script = self.merge_scripts(scripts);
        let script_server = self.merge_scripts(scripts_server);
        let script_client = self.merge_scripts(scripts_client);

        // Analyze scripts to extract metadata (signals, props, emits)
        let mut script_meta = None;
        if let Some(s) = &script {
            let analyzer = nargo_script_analyzer::ScriptAnalyzer::new();
            if let Ok(meta) = analyzer.analyze(s) {
                script_meta = Some(meta.to_nargo_value());
            }
        }

        Ok(IRModule { name: self.name.clone(), metadata, script, script_server, script_client, script_meta, template: if template_nodes.is_empty() { None } else { Some(TemplateIR { nodes: template_nodes, span: Span::default() }) }, hoisted_nodes: HashMap::new(), styles: ir_styles, i18n: None, wasm: Vec::new(), custom_blocks, tests: Vec::new(), dependencies: Vec::new(), dependents: Vec::new(), span: Span::default() })
    }

    fn parse_template_block(&mut self, nodes: &mut Vec<nargo_ir::TemplateNodeIR>) -> Result<()> {
        let (content, attrs, start_pos) = self.parse_special_tag("template")?;
        let lang = attrs.get("lang").cloned().unwrap_or_else(|| "nargo".to_string());
        if let Some(template_parser) = self.registry.get_template_parser(&lang) {
            let mut sub_state = ParseState::with_cursor(Cursor::with_sliced_source(content, start_pos));
            nodes.extend(template_parser.parse(&mut sub_state, &lang)?);
        }
        Ok(())
    }

    fn parse_script_block(&mut self, scripts: &mut Vec<nargo_ir::JsProgram>, server: &mut Vec<nargo_ir::JsProgram>, client: &mut Vec<nargo_ir::JsProgram>) -> Result<()> {
        let (content, attrs, start_pos) = self.parse_special_tag("script")?;
        let lang = attrs.get("lang").cloned().unwrap_or_else(|| "ts".to_string());
        let is_server = attrs.contains_key("server");
        let is_client = attrs.contains_key("client");

        if let Some(script_parser) = self.registry.get_script_parser(&lang) {
            let mut sub_state = ParseState::with_cursor(Cursor::with_sliced_source(content, start_pos));
            let program = script_parser.parse(&mut sub_state, &lang)?;

            if is_server {
                server.push(program);
            }
            else if is_client {
                client.push(program);
            }
            else {
                scripts.push(program);
            }
        }
        Ok(())
    }

    fn parse_style_block(&mut self, styles: &mut Vec<StyleIR>) -> Result<()> {
        let (content, attrs, start_pos) = self.parse_special_tag("style")?;
        let lang = attrs.get("lang").cloned().unwrap_or_else(|| "css".to_string());
        let scoped = attrs.contains_key("scoped");
        let span = self.state.cursor.span_from(start_pos);

        if let Some(style_parser) = self.registry.get_style_parser(&lang) {
            let mut sub_state = ParseState::with_cursor(Cursor::with_sliced_source(content, start_pos));
            if let Ok((code, trivia)) = style_parser.parse(&mut sub_state, &lang) {
                styles.push(StyleIR { code, lang, scoped, span, trivia });
            }
        }
        Ok(())
    }

    fn parse_metadata_block(&mut self, metadata: &mut HashMap<String, nargo_types::NargoValue>) -> Result<()> {
        let (content, attrs, start_pos) = self.parse_special_tag("metadata")?;
        let lang = attrs.get("lang").cloned().unwrap_or_else(|| "json".to_string());
        if let Some(metadata_parser) = self.registry.get_metadata_parser(&lang) {
            let mut sub_state = ParseState::with_cursor(Cursor::with_sliced_source(content, start_pos));
            if let Ok((nargo_types::NargoValue::Object(map), _trivia)) = metadata_parser.parse(&mut sub_state, &lang) {
                metadata.extend(map);
            }
        }
        Ok(())
    }

    fn parse_custom_block(&mut self, blocks: &mut Vec<CustomBlockIR>) -> Result<()> {
        self.state.cursor.consume(); // consume '<'
        let block_name = self.state.cursor.consume_while(|c| c.is_alphanumeric() || c == '-');
        let attrs = self.state.parse_tag_attributes();
        self.state.cursor.expect('>')?;

        let start_pos = self.state.cursor.position();
        let start_offset = self.state.cursor.pos;
        let end_tag = format!("</{}>", block_name);

        // 优化：使用更高效的字符串查找算法，而不是逐字符消耗
        let remaining_source = &self.state.cursor.source[start_offset..];
        if let Some(end_offset) = remaining_source.find(&end_tag) {
            self.state.cursor.pos = start_offset + end_offset;
        }
        else {
            // 如果没有找到结束标签，继续消耗直到EOF
            while !self.state.cursor.is_eof() {
                self.state.cursor.consume();
            }
        }

        let content = self.state.cursor.current_str(start_offset).to_string();
        let span = self.state.cursor.span_from(start_pos);
        self.state.cursor.consume_str(&end_tag);

        blocks.push(CustomBlockIR { name: block_name, content, attributes: attrs, span, trivia: nargo_ir::Trivia::default() });
        Ok(())
    }

    fn parse_special_tag(&mut self, tag: &str) -> Result<(&'a str, HashMap<String, String>, nargo_types::Position)> {
        let start_tag = format!("<{}", tag);
        let end_tag = format!("</{}>", tag);

        self.state.cursor.consume_str(&start_tag);
        let attrs = self.state.parse_tag_attributes();
        self.state.cursor.expect('>')?;

        let start_pos = self.state.cursor.position();
        let start_offset = self.state.cursor.pos;

        // 优化：使用更高效的字符串查找算法，而不是逐字符消耗
        let remaining_source = &self.state.cursor.source[start_offset..];
        if let Some(end_offset) = remaining_source.find(&end_tag) {
            self.state.cursor.pos = start_offset + end_offset;
        }
        else {
            // 如果没有找到结束标签，继续消耗直到EOF
            while !self.state.cursor.is_eof() {
                self.state.cursor.consume();
            }
        }

        let content = &self.state.cursor.source[start_offset..self.state.cursor.pos];
        self.state.cursor.consume_str(&end_tag);

        Ok((content, attrs, start_pos))
    }

    fn merge_scripts(&self, mut scripts: Vec<nargo_ir::JsProgram>) -> Option<nargo_ir::JsProgram> {
        if scripts.is_empty() {
            return None;
        }
        let mut first = scripts.remove(0);
        for mut script in scripts {
            first.body.append(&mut script.body);
        }
        Some(first)
    }
}
