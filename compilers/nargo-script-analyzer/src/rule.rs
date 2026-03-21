use nargo_ir::{JsExpr, JsProgram, JsStmt};
use nargo_types::{NargoValue, Span};
use std::collections::HashSet;

use crate::types::{AnalysisIssue, AnalysisReport, IssueLevel, ScriptMetadata};

/// 规则接口
pub trait Rule {
    /// 获取规则代码
    fn code(&self) -> String;
    /// 获取规则描述
    fn description(&self) -> String;
    /// 执行规则检查
    fn check(&self, program: &JsProgram, meta: &ScriptMetadata, report: &mut AnalysisReport);
}

/// 规则引擎
#[derive(Default)]
pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleEngine {
    /// 创建新的规则引擎
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// 执行所有规则检查
    pub fn run(&self, program: &JsProgram, meta: &ScriptMetadata, report: &mut AnalysisReport) {
        for rule in &self.rules {
            rule.check(program, meta, report);
        }
    }
}
