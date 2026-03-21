use nargo_ir::*;
use nargo_types::{NargoValue, Result, Span};

/// 死代码消除优化插件
pub struct DeadCodeEliminationPass;

impl DeadCodeEliminationPass {
    /// 创建新的死代码消除优化插件
    pub fn new() -> Self {
        Self
    }

    fn transform_stmts(&self, stmts: &mut Vec<JsStmt>) {
        let mut i = 0;
        while i < stmts.len() {
            let stmt = &stmts[i];
            if matches!(stmt, JsStmt::Return(_, _, _) | JsStmt::Break(_, _) | JsStmt::Continue(_, _)) {
                // 移除不可达代码
                stmts.truncate(i + 1);
                break;
            }
            i += 1;
        }

        for stmt in stmts {
            match stmt {
                JsStmt::FunctionDecl { body, .. } => {
                    self.transform_stmts(body);
                }
                JsStmt::If { consequent, alternate, .. } => {
                    let mut cons_stmt = std::mem::replace(consequent, Box::new(JsStmt::Expr(JsExpr::Literal(NargoValue::Null, Span::unknown(), nargo_ir::Trivia::default()), Span::unknown(), nargo_ir::Trivia::default())));
                    let mut cons_vec = vec![*cons_stmt];
                    self.transform_stmts(&mut cons_vec);
                    if let Some(stmt) = cons_vec.pop() {
                        *consequent = Box::new(stmt);
                    }
                    else {
                        *consequent = Box::new(JsStmt::Expr(JsExpr::Literal(NargoValue::Null, Span::unknown(), nargo_ir::Trivia::default()), Span::unknown(), nargo_ir::Trivia::default()));
                    }

                    if let Some(alt) = alternate {
                        let mut alt_stmt = std::mem::replace(alt, Box::new(JsStmt::Expr(JsExpr::Literal(NargoValue::Null, Span::unknown(), nargo_ir::Trivia::default()), Span::unknown(), nargo_ir::Trivia::default())));
                        let mut alt_vec = vec![*alt_stmt];
                        self.transform_stmts(&mut alt_vec);
                        if let Some(stmt) = alt_vec.pop() {
                            *alt = Box::new(stmt);
                        }
                        else {
                            *alt = Box::new(JsStmt::Expr(JsExpr::Literal(NargoValue::Null, Span::unknown(), nargo_ir::Trivia::default()), Span::unknown(), nargo_ir::Trivia::default()));
                        }
                    }
                }
                JsStmt::Block(body, _, _) => {
                    self.transform_stmts(body);
                }
                _ => {}
            }
        }
    }
}

impl crate::TransformPass for DeadCodeEliminationPass {
    fn name(&self) -> String {
        "DeadCodeEliminationPass".to_string()
    }

    fn description(&self) -> String {
        "Removes unreachable code and unused declarations".to_string()
    }

    fn transform(&mut self, ir: &mut IRModule) -> Result<()> {
        if let Some(script) = &mut ir.script {
            self.transform_stmts(&mut script.body);
        }
        Ok(())
    }
}
