use nargo_ir::{IRModule, JsExpr, JsProgram, JsStmt};
use nargo_types::Result;

pub struct TsAdapter;

impl TsAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn transform(&self, ir: &mut IRModule) -> Result<()> {
        if let Some(script) = &mut ir.script {
            self.transform_script(script)?;
        }
        if let Some(script) = &mut ir.script_server {
            self.transform_script(script)?;
        }
        if let Some(script) = &mut ir.script_client {
            self.transform_script(script)?;
        }
        for test in &mut ir.tests {
            self.transform_script(&mut test.body)?;
        }
        Ok(())
    }

    fn transform_script(&self, script: &mut JsProgram) -> Result<()> {
        // 1. Identify state variables and computed macros
        let mut reactive_vars = Vec::new();
        let mut computed_vars = Vec::new();

        // 预分配空间，避免频繁扩容
        reactive_vars.reserve(10);
        computed_vars.reserve(5);

        for stmt in &mut script.body {
            if let JsStmt::VariableDecl { kind, id, init, .. } = stmt {
                if kind == "let" || kind == "var" {
                    // Check if it's already a reactive call
                    let is_already_reactive = if let Some(JsExpr::Call { callee, .. }) = init { if let JsExpr::Identifier(name, _, _) = &**callee { name == "ref" || name == "signal" || name == "createSignal" || name == "reactive" || name == "computed" || name == "createComputed" } else { false } } else { false };

                    if !is_already_reactive {
                        reactive_vars.push(id.clone());
                    }
                }
                else if kind == "const" {
                    if let Some(JsExpr::Call { callee, args, span, trivia }) = init {
                        let mut should_transform_effect = false;
                        if let JsExpr::Identifier(name, _, _) = &**callee {
                            if name == "$computed" || name == "computed" || name == "createComputed" {
                                computed_vars.push(id.clone());
                            }
                            else if name == "$effect" || name == "effect" || name == "createEffect" {
                                should_transform_effect = true;
                            }
                        }

                        if should_transform_effect {
                            *callee = Box::new(JsExpr::Identifier("effect".to_string(), *span, trivia.clone()));
                            // If it's $effect(expr), wrap in arrow function
                            if args.len() == 1 && !matches!(args[0], JsExpr::ArrowFunction { .. }) {
                                let expr = args.pop().unwrap();
                                args.push(JsExpr::ArrowFunction { params: Vec::new(), body: Box::new(expr), span: *span, trivia: trivia.clone() });
                            }
                        }
                    }
                }
            }
        }

        // 2. Transform declarations
        for stmt in &mut script.body {
            if let JsStmt::VariableDecl { kind, id, init, span, trivia } = stmt {
                if (kind == "let" || kind == "var") && reactive_vars.contains(id) {
                    *kind = "const".to_string();
                    let initial_value = init.take().unwrap_or(JsExpr::Literal(nargo_types::NargoValue::Null, *span, trivia.clone()));

                    *init = Some(JsExpr::Call { callee: Box::new(JsExpr::Identifier("ref".to_string(), *span, trivia.clone())), args: vec![initial_value], span: *span, trivia: trivia.clone() });
                }
                else if kind == "const" && computed_vars.contains(id) {
                    if let Some(JsExpr::Call { callee, args, span, trivia }) = init {
                        if let JsExpr::Identifier(name, _, _) = &**callee {
                            if name == "$computed" || name == "computed" || name == "createComputed" {
                                *callee = Box::new(JsExpr::Identifier("computed".to_string(), *span, trivia.clone()));
                                // Transform args (wrap in arrow function if not already)
                                if args.len() == 1 {
                                    if !matches!(args[0], JsExpr::ArrowFunction { .. }) {
                                        let expr = args.pop().unwrap();
                                        args.push(JsExpr::ArrowFunction { params: Vec::new(), body: Box::new(expr), span: *span, trivia: trivia.clone() });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Transform usages: x -> x.value
        let mut all_reactive = reactive_vars;
        all_reactive.extend(computed_vars);

        for stmt in &mut script.body {
            self.transform_stmt(stmt, &all_reactive);
        }

        Ok(())
    }

    fn transform_stmt(&self, stmt: &mut JsStmt, reactive_vars: &[String]) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.transform_expr(expr, reactive_vars),
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.transform_expr(expr, reactive_vars);
                }
            }
            JsStmt::Return(expr, _, _) => {
                if let Some(e) = expr {
                    self.transform_expr(e, reactive_vars);
                }
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                self.transform_expr(test, reactive_vars);
                self.transform_stmt(consequent, reactive_vars);
                if let Some(alt) = alternate {
                    self.transform_stmt(alt, reactive_vars);
                }
            }
            JsStmt::While { test, body, .. } => {
                self.transform_expr(test, reactive_vars);
                self.transform_stmt(body, reactive_vars);
            }
            JsStmt::For { init, test, update, body, .. } => {
                if let Some(i) = init {
                    self.transform_stmt(i, reactive_vars);
                }
                if let Some(t) = test {
                    self.transform_expr(t, reactive_vars);
                }
                if let Some(u) = update {
                    self.transform_expr(u, reactive_vars);
                }
                self.transform_stmt(body, reactive_vars);
            }
            JsStmt::Block(stmts, _, _) => {
                for s in stmts {
                    self.transform_stmt(s, reactive_vars);
                }
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.transform_stmt(s, reactive_vars);
                }
            }
            _ => {}
        }
    }

    fn transform_expr(&self, expr: &mut JsExpr, reactive_vars: &[String]) {
        match expr {
            JsExpr::Identifier(name, span, trivia) => {
                if reactive_vars.contains(name) {
                    // 直接使用引用，避免不必要的克隆
                    let name_clone = name.clone();
                    let span_clone = *span;
                    let trivia_clone = trivia.clone();

                    *expr = JsExpr::Member { object: Box::new(JsExpr::Identifier(name_clone, span_clone, trivia_clone.clone())), property: Box::new(JsExpr::Identifier("value".to_string(), span_clone, trivia_clone.clone())), computed: false, span: span_clone, trivia: trivia_clone };
                }
            }
            JsExpr::Unary { argument, .. } => self.transform_expr(argument, reactive_vars),
            JsExpr::Binary { left, right, .. } => {
                self.transform_expr(left, reactive_vars);
                self.transform_expr(right, reactive_vars);
            }
            JsExpr::Call { callee, args, span, trivia } => {
                // Handle macros
                if let JsExpr::Identifier(name, _, _) = &**callee {
                    if name == "$effect" || name == "effect" || name == "createEffect" {
                        *callee = Box::new(JsExpr::Identifier("effect".to_string(), *span, trivia.clone()));
                        if args.len() == 1 && !matches!(args[0], JsExpr::ArrowFunction { .. }) {
                            let expr = args.pop().unwrap();
                            args.push(JsExpr::ArrowFunction { params: Vec::new(), body: Box::new(expr), span: *span, trivia: trivia.clone() });
                        }
                    }
                }

                self.transform_expr(callee, reactive_vars);
                for arg in args {
                    self.transform_expr(arg, reactive_vars);
                }
            }
            JsExpr::Member { object, property, computed, .. } => {
                self.transform_expr(object, reactive_vars);
                if *computed {
                    self.transform_expr(property, reactive_vars);
                }
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.transform_expr(el, reactive_vars);
                }
            }
            JsExpr::Object(properties, _, _) => {
                for value in properties.values_mut() {
                    self.transform_expr(value, reactive_vars);
                }
            }
            JsExpr::ArrowFunction { body, .. } => {
                self.transform_expr(body, reactive_vars);
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                self.transform_expr(test, reactive_vars);
                self.transform_expr(consequent, reactive_vars);
                self.transform_expr(alternate, reactive_vars);
            }
            JsExpr::TemplateLiteral { expressions, .. } => {
                for e in expressions {
                    self.transform_expr(e, reactive_vars);
                }
            }
            JsExpr::TseElement { attributes, children, .. } => {
                for attr in attributes {
                    if let Some(val) = &mut attr.value {
                        self.transform_expr(val, reactive_vars);
                    }
                }
                for child in children {
                    self.transform_expr(child, reactive_vars);
                }
            }
            _ => {}
        }
    }
}
