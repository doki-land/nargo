use crate::{ParseState, TemplateParser};
use nargo_ir::{AttributeIR, Comment, ElementIR, ExpressionIR, ForIteratorIR, ForNodeIR, IfNodeIR, TemplateNodeIR, Trivia};
use nargo_types::{Error, Result, Span, is_void_element};

pub struct VueTemplateParser;

pub fn parse(source: &str) -> Result<Vec<TemplateNodeIR>> {
    let mut state = ParseState::new(source);
    let parser = VueTemplateParser;
    parser.parse(&mut state, "html")
}

impl TemplateParser for VueTemplateParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<Vec<TemplateNodeIR>> {
        let mut parser = TemplateParserImpl { state };
        parser.parse()
    }
}

struct TemplateParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> TemplateParserImpl<'a, 'b> {
    fn consume_trivia(&mut self) -> Trivia {
        let leading_whitespace = self.state.cursor.consume_whitespace();
        let mut leading_comments = Vec::new();

        while self.state.cursor.peek_str("<!--") {
            let start_pos = self.state.cursor.position();
            self.state.cursor.consume_n(4); // <!--
            let start = self.state.cursor.pos;
            while !self.state.cursor.is_eof() && !self.state.cursor.peek_str("-->") {
                self.state.cursor.consume();
            }
            let content = self.state.cursor.current_str(start).to_string();
            let _ = self.state.cursor.consume_n(3); // -->
            let end_pos = self.state.cursor.position();
            leading_comments.push(Comment { content, is_block: true, span: Span { start: start_pos, end: end_pos } });
            // After a comment, there might be more whitespace
            let _extra_ws = self.state.cursor.consume_whitespace();
            // We can append this to the leading_whitespace or ignore it for now
            // For simplicity, let's keep it simple.
        }

        Trivia { leading_whitespace, leading_comments, trailing_comments: Vec::new() }
    }

    pub fn parse(&mut self) -> Result<Vec<TemplateNodeIR>> {
        let mut nodes = Vec::new();
        while !self.state.cursor.is_eof() {
            if self.state.cursor.peek() == '{' {
                nodes.push(self.parse_interpolation()?);
            }
            else if self.state.cursor.peek() == '<' {
                if self.state.cursor.peek_str("<!--") {
                    nodes.push(self.parse_comment()?);
                }
                else if self.state.cursor.peek_str("</") {
                    break; // Closing tag, handle in parent
                }
                else {
                    nodes.push(self.parse_element()?);
                }
            }
            else {
                nodes.push(self.parse_text()?);
            }
        }
        Ok(nodes)
    }

    fn parse_element(&mut self) -> Result<TemplateNodeIR> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect('<')?;
        let tag = self.state.cursor.consume_while(|c| c.is_alphanumeric() || c == '-');
        if tag.is_empty() {
            return Err(Error::parse_error("Expected tag name".to_string(), self.state.cursor.span_at_current()));
        }
        println!("Parsing element: <{}>", tag);
        let mut attributes = Vec::new();

        let mut trivia = self.consume_trivia();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '>' && !self.state.cursor.peek_str("/>") {
            let attr_start = self.state.cursor.position();
            let attr_name = self.state.cursor.consume_while(|c| !c.is_whitespace() && c != '=' && c != '>' && c != '/');

            let mut attr_trivia = self.consume_trivia();
            let attr_value = if self.state.cursor.peek() == '=' {
                self.state.cursor.consume();
                attr_trivia.leading_whitespace.push('=');
                attr_trivia.leading_whitespace.push_str(&self.state.cursor.consume_whitespace());

                let peek = self.state.cursor.peek();
                if peek == '"' || peek == '\'' {
                    Some(self.state.cursor.consume_string()?)
                }
                else if peek == '{' {
                    // Dynamic attribute: attr={val}
                    let interpolation = self.parse_interpolation()?;
                    if let TemplateNodeIR::Interpolation(expr) = interpolation { Some(expr.code) } else { None }
                }
                else {
                    Some(self.state.cursor.consume_while(|c| !c.is_whitespace() && c != '>' && c != '/'))
                }
            }
            else {
                None
            };

            // VOC syntax uses colon for directives: on:click
            let is_directive = attr_name.contains(':');
            let is_dynamic = attr_value.as_ref().map(|v| v.starts_with('{') || is_directive).unwrap_or(false);

            let value_ast = None;
            let attr_end = self.state.cursor.position();
            attributes.push(AttributeIR { name: attr_name, value: attr_value, value_ast, argument: None, modifiers: Vec::new(), is_directive, is_dynamic, span: Span { start: attr_start, end: attr_end }, trivia: attr_trivia });
            trivia = self.consume_trivia();
        }

        let is_self_closing = if self.state.cursor.peek_str("/>") {
            self.state.cursor.consume_n(2);
            true
        }
        else {
            self.state.cursor.expect('>')?;
            false
        };

        let mut children = Vec::new();
        if !is_self_closing && !is_void_element(&tag) {
            if tag == "script" || tag == "style" {
                let start = self.state.cursor.pos;
                let end_tag = format!("</{}>", tag);
                while !self.state.cursor.is_eof() && !self.state.cursor.peek_str(&end_tag) {
                    self.state.cursor.consume();
                }
                let content = self.state.cursor.current_str(start).to_string();
                if !content.is_empty() {
                    let text_span = self.state.cursor.span_from(start_pos);
                    children.push(TemplateNodeIR::Text(content, text_span, Trivia::default()));
                }
                self.state.cursor.expect_str(&end_tag)?;
            }
            else if tag == "if" {
                // 处理条件指令
                children = self.parse()?;
                self.state.cursor.expect_str("</if>")?;
                let end_pos = self.state.cursor.position();

                // 查找条件属性
                let mut condition = ExpressionIR::default();
                for attr in &attributes {
                    if attr.name == "test" || attr.name == "condition" {
                        if let Some(ref value) = attr.value {
                            condition = ExpressionIR { code: value.clone(), ast: attr.value_ast.clone(), is_static: false, span: attr.span, trivia: attr.trivia.clone() };
                        }
                        break;
                    }
                }

                return Ok(TemplateNodeIR::If(nargo_ir::IfNodeIR { condition, consequent: children, alternate: None, else_ifs: Vec::new(), span: Span { start: start_pos, end: end_pos } }));
            }
            else if tag == "for" {
                // 处理循环指令
                children = self.parse()?;
                self.state.cursor.expect_str("</for>")?;
                let end_pos = self.state.cursor.position();

                // 查找循环属性
                let mut iterator = nargo_ir::ForIteratorIR::default();
                for attr in &attributes {
                    if attr.name == "each" {
                        if let Some(ref value) = attr.value {
                            // 处理格式: item in items 或 (item, index) in items
                            if let Some((left, right)) = value.split_once(" in ") {
                                let left = left.trim();
                                if let Some((item, index)) = left.split_once(',') {
                                    iterator.item = item.trim().to_string();
                                    iterator.index = Some(index.trim().to_string());
                                }
                                else {
                                    iterator.item = left.to_string();
                                }
                                iterator.collection = ExpressionIR { code: right.trim().to_string(), ast: None, is_static: false, span: attr.span, trivia: attr.trivia.clone() };
                            }
                        }
                        break;
                    }
                }

                return Ok(TemplateNodeIR::For(nargo_ir::ForNodeIR { iterator, body: children, span: Span { start: start_pos, end: end_pos } }));
            }
            else {
                children = self.parse()?;
                println!("Element <{}> found {} children", tag, children.len());
                self.state.cursor.expect_str(&format!("</{}>", tag))?;
            }
        }

        let end_pos = self.state.cursor.position();

        Ok(TemplateNodeIR::Element(ElementIR { tag, attributes, children, is_static: false, span: Span { start: start_pos, end: end_pos }, trivia }))
    }

    fn parse_interpolation(&mut self) -> Result<TemplateNodeIR> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect('{')?;
        let start = self.state.cursor.pos;
        let mut depth = 1;
        while !self.state.cursor.is_eof() && depth > 0 {
            let c = self.state.cursor.consume();
            if c == '{' {
                depth += 1;
            }
            else if c == '}' {
                depth -= 1;
            }
        }
        let content = self.state.cursor.current_str(start);
        let content = &content[..content.len() - 1]; // Remove trailing }
        let content = content.trim().to_string();
        // Remove surrounding braces if present
        let content = if content.starts_with('{') && content.ends_with('}') { content[1..content.len() - 1].trim().to_string() } else { content };
        let end_pos = self.state.cursor.position();

        let ast = None;

        Ok(TemplateNodeIR::Interpolation(ExpressionIR { code: content, ast, is_static: false, span: Span { start: start_pos, end: end_pos }, trivia: Trivia::default() }))
    }

    fn parse_text(&mut self) -> Result<TemplateNodeIR> {
        let start_pos = self.state.cursor.position();
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '<' && self.state.cursor.peek() != '{' {
            self.state.cursor.consume();
        }
        let end_pos = self.state.cursor.position();
        Ok(TemplateNodeIR::Text(self.state.cursor.current_str(start).to_string(), Span { start: start_pos, end: end_pos }, Trivia::default()))
    }

    fn parse_comment(&mut self) -> Result<TemplateNodeIR> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect_str("<!--")?;
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && !self.state.cursor.peek_str("-->") {
            self.state.cursor.consume();
        }
        let content = self.state.cursor.current_str(start).to_string();
        self.state.cursor.expect_str("-->")?;
        let end_pos = self.state.cursor.position();
        Ok(TemplateNodeIR::Comment(content, Span { start: start_pos, end: end_pos }, Trivia::default()))
    }
}
