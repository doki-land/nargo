#![warn(missing_docs)]

pub mod analyzer;
pub mod rule;
pub mod rules;
pub mod types;

pub use analyzer::ScriptAnalyzer;
pub use rule::{Rule, RuleEngine};
pub use rules::*;
pub use types::*;

/// 导出默认规则引擎配置
pub fn default_rule_engine() -> RuleEngine {
    let mut engine = RuleEngine::new();
    engine.add_rule(Box::new(UnusedVariableRule));
    engine.add_rule(Box::new(UndefinedVariableRule));
    engine.add_rule(Box::new(UnsafeOperationRule));
    engine.add_rule(Box::new(UnusedImportRule));
    engine.add_rule(Box::new(MemoryLeakRule));
    engine.add_rule(Box::new(PerformanceBottleneckRule));
    engine.add_rule(Box::new(SecurityRule));
    engine.add_rule(Box::new(CodeStyleRule));
    engine.add_rule(Box::new(CodeComplexityRule));
    engine.add_rule(Box::new(OptimizationSuggestionRule));
    engine.add_rule(Box::new(RefactoringSuggestionRule));
    engine
}
