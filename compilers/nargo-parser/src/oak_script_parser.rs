use crate::{ParseState, ScriptParser};
use nargo_ir::{JsExpr, JsProgram, JsStmt, Trivia};
use nargo_types::{Position, Result, Span};
use oak_core::{Parser, SourceText, parser::ParseSession, tree::RedTree};
use oak_typescript::{TypeScriptElementType, TypeScriptLanguage, TypeScriptParser};

/// 基于 oak-typescript 的脚本解析器
pub struct OakTypeScriptParser;

impl ScriptParser for OakTypeScriptParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<JsProgram> {
        let source = SourceText::new(state.cursor.source.to_string());
        let language = TypeScriptLanguage::default();
        let parser = TypeScriptParser::new(&language);
        let mut session = ParseSession::default();
        let parse_output = parser.parse(&source, &[], &mut session);

        // 将 green 树转换为 red 树，获取包含位置信息的 AST
        let green_node = parse_output.result.map_err(|e| nargo_types::Error::parse_error(format!("TypeScript parsing error: {:?}", e), Span::default()))?;
        let red_tree = RedTree::new(green_node);

        // 转换 oak-typescript 的解析结果为项目的 IR 格式
        let program = self.convert_to_ir(&red_tree, &source);
        Ok(program)
    }
}

impl OakTypeScriptParser {
    /// 将 oak-typescript 的解析结果转换为项目的 IR 格式
    fn convert_to_ir(&self, tree: &RedTree<'_, TypeScriptLanguage>, source: &SourceText) -> JsProgram {
        let mut body = Vec::new();
        self.convert_node(tree, source, &mut body);

        let span = if let RedTree::Node(node) = tree { Span { start: Position::new(1, 1, node.span().start as u32), end: Position::new(1, 1, node.span().end as u32) } } else { Span { start: Position::new(1, 1, 0), end: Position::new(1, 1, 0) } };

        JsProgram { body, span, trivia: Trivia::default() }
    }

    /// 转换单个节点
    fn convert_node(&self, tree: &RedTree<'_, TypeScriptLanguage>, source_text: &SourceText, result: &mut Vec<JsStmt>) {
        match tree {
            RedTree::Node(node) => {
                match node.element_type() {
                    TypeScriptElementType::SourceFile => {
                        // 处理程序根节点
                        for child in node.children() {
                            self.convert_node(&child, source_text, result);
                        }
                    }
                    TypeScriptElementType::VariableDeclaration => {
                        // 处理变量声明
                        let mut declarations = Vec::new();
                        let mut kind = "var".to_string();

                        for child in node.children() {
                            if let RedTree::Node(child_node) = &child {
                                match child_node.element_type() {
                                    TypeScriptElementType::VariableDeclaration => {
                                        for decl_child in child_node.children() {
                                            if let RedTree::Node(decl_node) = &decl_child {
                                                if decl_node.element_type() == TypeScriptElementType::VariableDeclaration {
                                                    declarations.push(self.convert_variable_declaration(&decl_child, source_text));
                                                }
                                            }
                                        }
                                    }
                                    TypeScriptElementType::Var => kind = "var".to_string(),
                                    TypeScriptElementType::Let => kind = "let".to_string(),
                                    TypeScriptElementType::Const => kind = "const".to_string(),
                                    _ => {}
                                }
                            }
                        }

                        let span = Span { start: Position::new(1, 1, node.span().start as u32), end: Position::new(1, 1, node.span().end as u32) };
                        for (id, init) in declarations {
                            result.push(JsStmt::VariableDecl { kind: kind.clone(), id, init, span, trivia: Trivia::default() });
                        }
                    }
                    TypeScriptElementType::FunctionDeclaration => {
                        // 处理函数声明
                        let mut id = None;
                        let mut params = Vec::new();
                        let mut body = Vec::new();

                        for child in node.children() {
                            if let RedTree::Node(child_node) = &child {
                                match child_node.element_type() {
                                    TypeScriptElementType::IdentifierName => {
                                        id = Some(self.get_node_text(child_node, source_text));
                                    }
                                    TypeScriptElementType::OpenParen => {
                                        for param_child in child_node.children() {
                                            if let RedTree::Node(param_node) = &param_child {
                                                if param_node.element_type() == TypeScriptElementType::Parameter {
                                                    params.push(self.convert_parameter(&param_child, source_text));
                                                }
                                            }
                                        }
                                    }
                                    TypeScriptElementType::OpenBrace => {
                                        for body_child in child_node.children() {
                                            self.convert_node(&body_child, source_text, &mut body);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        let span = Span { start: Position::new(1, 1, node.span().start as u32), end: Position::new(1, 1, node.span().end as u32) };
                        result.push(JsStmt::FunctionDecl { id: id.unwrap_or_default(), params, body, is_async: false, span, trivia: Trivia::default() });
                    }
                    TypeScriptElementType::ImportDeclaration => {
                        // 处理导入声明
                        let mut specifiers = Vec::new();
                        let mut import_source = None;

                        for child in node.children() {
                            if let RedTree::Node(child_node) = &child {
                                match child_node.element_type() {
                                    TypeScriptElementType::ImportSpecifier => {
                                        specifiers.push(self.convert_import_specifier(&child, source_text));
                                    }
                                    TypeScriptElementType::StringLiteral => {
                                        import_source = Some(self.get_node_text(child_node, source_text));
                                    }
                                    _ => {}
                                }
                            }
                        }

                        let span = Span { start: Position::new(1, 1, node.span().start as u32), end: Position::new(1, 1, node.span().end as u32) };
                        // 转换 specifiers 为 Vec<String>
                        let import_specifiers: Vec<String> = specifiers.into_iter().map(|(local, imported)| if let Some(local) = local { format!("{} as {}", imported, local) } else { imported }).collect();
                        result.push(JsStmt::Import { source: import_source.unwrap_or_default(), specifiers: import_specifiers, span, trivia: Trivia::default() });
                    }
                    TypeScriptElementType::ExpressionStatement => {
                        // 处理表达式语句
                        let mut expr = JsExpr::Other("".to_string(), Span::default(), Trivia::default());

                        for child in node.children() {
                            if let RedTree::Node(child_node) = &child {
                                expr = self.convert_expression(&child, source_text);
                            }
                        }

                        let span = Span { start: Position::new(1, 1, node.span().start as u32), end: Position::new(1, 1, node.span().end as u32) };
                        result.push(JsStmt::Expr(expr, span, Trivia::default()));
                    }
                    TypeScriptElementType::ClassDeclaration => {
                        // 处理类声明
                        let mut id = None;
                        let mut body = Vec::new();

                        for child in node.children() {
                            if let RedTree::Node(child_node) = &child {
                                match child_node.element_type() {
                                    TypeScriptElementType::IdentifierName => {
                                        id = Some(self.get_node_text(child_node, source_text));
                                    }
                                    TypeScriptElementType::ClassBody => {
                                        for body_child in child_node.children() {
                                            if let RedTree::Node(body_node) = &body_child {
                                                if body_node.element_type() == TypeScriptElementType::MethodDeclaration {
                                                    body.push(self.convert_method_definition(&body_child, source_text));
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        let span = Span { start: Position::new(1, 1, node.span().start as u32), end: Position::new(1, 1, node.span().end as u32) };
                        // 暂时使用 Other 变体来表示类声明，因为 JsStmt 中没有 ClassDecl 变体
                        let class_code = format!("class {}", id.unwrap_or_default());
                        result.push(JsStmt::Other(class_code, span, Trivia::default()));
                    }
                    _ => {
                        // 处理其他节点类型
                        for child in node.children() {
                            self.convert_node(&child, source_text, result);
                        }
                    }
                }
            }
            RedTree::Leaf(_) => {
                // 叶子节点已经在父节点中处理
            }
        }
    }

    /// 转换变量声明
    fn convert_variable_declaration(&self, tree: &RedTree<'_, TypeScriptLanguage>, source_text: &SourceText) -> (String, Option<JsExpr>) {
        let mut id = "".to_string();
        let mut init = None;

        if let RedTree::Node(node) = tree {
            for child in node.children() {
                if let RedTree::Node(child_node) = &child {
                    match child_node.element_type() {
                        TypeScriptElementType::IdentifierName => {
                            id = self.get_node_text(child_node, source_text);
                        }
                        TypeScriptElementType::Equal => {
                            for init_child in child_node.children() {
                                init = Some(self.convert_expression(&init_child, source_text));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        (id, init)
    }

    /// 转换参数
    fn convert_parameter(&self, tree: &RedTree<'_, TypeScriptLanguage>, source_text: &SourceText) -> String {
        if let RedTree::Node(node) = tree {
            for child in node.children() {
                if let RedTree::Node(child_node) = &child {
                    if child_node.element_type() == TypeScriptElementType::IdentifierName {
                        return self.get_node_text(child_node, source_text);
                    }
                }
            }
        }
        "".to_string()
    }

    /// 转换导入说明符
    fn convert_import_specifier(&self, tree: &RedTree<'_, TypeScriptLanguage>, source_text: &SourceText) -> (Option<String>, String) {
        let mut local = None;
        let mut imported = "".to_string();

        if let RedTree::Node(node) = tree {
            for child in node.children() {
                if let RedTree::Node(child_node) = &child {
                    match child_node.element_type() {
                        TypeScriptElementType::IdentifierName => {
                            if local.is_none() {
                                local = Some(self.get_node_text(child_node, source_text));
                            }
                            else {
                                imported = self.get_node_text(child_node, source_text);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        (local, imported)
    }

    /// 转换表达式
    fn convert_expression(&self, tree: &RedTree<'_, TypeScriptLanguage>, source_text: &SourceText) -> JsExpr {
        if let RedTree::Node(node) = tree { JsExpr::Other(self.get_node_text(node, source_text), Span::default(), Trivia::default()) } else { JsExpr::Other("".to_string(), Span::default(), Trivia::default()) }
    }

    /// 转换方法定义
    fn convert_method_definition(&self, tree: &RedTree<'_, TypeScriptLanguage>, source_text: &SourceText) -> (String, Vec<String>, Vec<JsStmt>) {
        let mut id = "".to_string();
        let mut params = Vec::new();
        let mut body = Vec::new();

        if let RedTree::Node(node) = tree {
            for child in node.children() {
                if let RedTree::Node(child_node) = &child {
                    // 简化处理，直接获取文本内容
                    let text = self.get_node_text(child_node, source_text);
                    if id.is_empty() {
                        id = text;
                    }
                }
            }
        }

        (id, params, body)
    }

    /// 获取节点的文本内容
    fn get_node_text(&self, node: &oak_core::tree::RedNode<'_, TypeScriptLanguage>, source_text: &SourceText) -> String {
        node.text(source_text).to_string()
    }
}
