pub use nargo_types::{Cursor, Error, ErrorKind, Position, Span};
use std::collections::HashMap;

pub struct ParseState<'a> {
    pub cursor: Cursor<'a>,
}

impl<'a> ParseState<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { cursor: Cursor::new(source) }
    }

    pub fn new_with_pos(source: &'a str, pos: Position) -> Self {
        Self { cursor: Cursor::with_position(source, pos) }
    }

    pub fn with_cursor(cursor: Cursor<'a>) -> Self {
        Self { cursor }
    }

    /// 获取当前位置的上下文信息
    pub fn get_context(&self, radius: usize) -> String {
        let start = if self.cursor.pos > radius { self.cursor.pos - radius } else { 0 };
        let end = if self.cursor.pos + radius < self.cursor.source.len() { self.cursor.pos + radius } else { self.cursor.source.len() };
        self.cursor.source[start..end].to_string()
    }

    /// 生成带有上下文的错误消息
    pub fn format_error_message(&self, message: &str) -> String {
        let line = self.cursor.line;
        let column = self.cursor.column;
        let context = self.get_context(20);
        let pointer = " ".repeat(20 - (self.cursor.pos % 20)) + "^";

        format!("Error at line {}, column {}:\n{}\n{}\n{}", line, column, message, context, pointer)
    }

    pub fn unexpected_char(&self) -> Error {
        let message = format!("Unexpected character '{}'", self.cursor.peek());
        let formatted_message = self.format_error_message(&message);
        Error::new(ErrorKind::UnexpectedChar { character: self.cursor.peek(), span: self.cursor.span_at_current() })
    }

    pub fn expected_char(&self, expected: char) -> Error {
        let message = format!("Expected '{}', found '{}'", expected, self.cursor.peek());
        let formatted_message = self.format_error_message(&message);
        Error::new(ErrorKind::ExpectedChar { expected, found: self.cursor.peek(), span: self.cursor.span_at_current() })
    }

    pub fn expected_string(&self, expected: &str) -> Error {
        let message = format!("Expected '{}', found '{}'", expected, self.cursor.peek());
        let formatted_message = self.format_error_message(&message);
        Error::new(ErrorKind::ExpectedString { expected: expected.to_string(), found: self.cursor.peek().to_string(), span: self.cursor.span_at_current() })
    }

    pub fn error(&self, message: impl Into<String>) -> Error {
        let message = message.into();
        let formatted_message = self.format_error_message(&message);
        Error::new(ErrorKind::Parse { message: formatted_message, span: self.cursor.span_at_current() })
    }

    /// 期望特定的标记
    pub fn expected_token(&self, expected: &str) -> Error {
        let message = format!("Expected '{}' token", expected);
        let formatted_message = self.format_error_message(&message);
        Error::new(ErrorKind::Parse { message: formatted_message, span: self.cursor.span_at_current() })
    }

    /// 语法错误
    pub fn syntax_error(&self, message: impl Into<String>) -> Error {
        let message = format!("Syntax error: {}", message.into());
        let formatted_message = self.format_error_message(&message);
        Error::new(ErrorKind::Parse { message: formatted_message, span: self.cursor.span_at_current() })
    }

    /// 类型错误
    pub fn type_error(&self, message: impl Into<String>) -> Error {
        let message = format!("Type error: {}", message.into());
        let formatted_message = self.format_error_message(&message);
        Error::new(ErrorKind::Parse { message: formatted_message, span: self.cursor.span_at_current() })
    }

    pub fn parse_tag_attributes(&mut self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        self.cursor.skip_whitespace();
        while !self.cursor.is_eof() && self.cursor.peek() != '>' {
            let start = self.cursor.pos;
            while !self.cursor.is_eof() && !self.cursor.peek().is_whitespace() && self.cursor.peek() != '=' && self.cursor.peek() != '>' {
                self.cursor.consume();
            }
            let key = self.cursor.current_str(start).to_string();
            self.cursor.skip_whitespace();
            if self.cursor.peek() == '=' {
                self.cursor.consume();
                self.cursor.skip_whitespace();
                let quote = self.cursor.peek();
                if quote == '"' || quote == '\'' {
                    self.cursor.consume();
                    let val_start = self.cursor.pos;
                    while !self.cursor.is_eof() && self.cursor.peek() != quote {
                        self.cursor.consume();
                    }
                    let val = self.cursor.current_str(val_start).to_string();
                    let _ = self.cursor.consume(); // consume quote
                    attrs.insert(key, val);
                }
                else {
                    let val_start = self.cursor.pos;
                    while !self.cursor.is_eof() && !self.cursor.peek().is_whitespace() && self.cursor.peek() != '>' {
                        self.cursor.consume();
                    }
                    let val = self.cursor.current_str(val_start).to_string();
                    attrs.insert(key, val);
                }
            }
            else {
                attrs.insert(key, "true".to_string());
            }
            self.cursor.skip_whitespace();
        }
        attrs
    }
}
