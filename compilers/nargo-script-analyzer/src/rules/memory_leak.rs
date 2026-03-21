use nargo_ir::{JsExpr, JsStmt};
use std::collections::HashSet;

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 内存泄漏检测规则
pub struct MemoryLeakRule;

impl Rule for MemoryLeakRule {
    fn code(&self) -> String {
        "memory-leak".to_string()
    }

    fn description(&self) -> String {
        "检查可能的内存泄漏".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        let mut set_intervals = HashSet::new();
        let mut set_timeouts = HashSet::new();
        let mut event_listeners = HashSet::new();

        // 检查定时器和事件监听器
        fn check_memory_leaks(expr: &JsExpr, set_intervals: &mut HashSet<String>, set_timeouts: &mut HashSet<String>, event_listeners: &mut HashSet<String>, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Call { callee, args, span, .. } => {
                    // 检查 setInterval
                    if let JsExpr::Identifier(name, _, _) = &**callee {
                        if name == "setInterval" {
                            if let Some(id_expr) = args.get(2) {
                                if let JsExpr::Identifier(id, _, _) = id_expr {
                                    set_intervals.insert(id.clone());
                                }
                            }
                            else {
                                report.add_issue(crate::types::IssueLevel::Warning, "memory-leak".to_string(), "setInterval without an ID may cause memory leaks".to_string(), Some(*span));
                            }
                        }
                        // 检查 setTimeout
                        else if name == "setTimeout" {
                            if let Some(id_expr) = args.get(2) {
                                if let JsExpr::Identifier(id, _, _) = id_expr {
                                    set_timeouts.insert(id.clone());
                                }
                            }
                        }
                    }
                    // 检查事件监听器
                    if let JsExpr::Member { object, property, computed: false, .. } = &**callee {
                        if let JsExpr::Identifier(prop_name, _, _) = &**property {
                            if prop_name == "addEventListener" {
                                event_listeners.insert(format!("{:?}.addEventListener", object));
                            }
                        }
                    }
                    check_memory_leaks(callee, set_intervals, set_timeouts, event_listeners, report);
                    for arg in args {
                        check_memory_leaks(arg, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_memory_leaks(object, set_intervals, set_timeouts, event_listeners, report);
                    if *computed {
                        check_memory_leaks(property, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                JsExpr::Binary { left, right, .. } => {
                    check_memory_leaks(left, set_intervals, set_timeouts, event_listeners, report);
                    check_memory_leaks(right, set_intervals, set_timeouts, event_listeners, report);
                }
                JsExpr::Unary { argument, .. } => {
                    check_memory_leaks(argument, set_intervals, set_timeouts, event_listeners, report);
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_memory_leaks(el, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_memory_leaks(value, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_memory_leaks(body, set_intervals, set_timeouts, event_listeners, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_memory_leaks(test, set_intervals, set_timeouts, event_listeners, report);
                    check_memory_leaks(consequent, set_intervals, set_timeouts, event_listeners, report);
                    check_memory_leaks(alternate, set_intervals, set_timeouts, event_listeners, report);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_memory_leaks(e, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                _ => {}
            }
        }

        fn check_memory_leaks_in_stmt(stmt: &JsStmt, set_intervals: &mut HashSet<String>, set_timeouts: &mut HashSet<String>, event_listeners: &mut HashSet<String>, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::Expr(expr, _, _) => check_memory_leaks(expr, set_intervals, set_timeouts, event_listeners, report),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        check_memory_leaks(e, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        check_memory_leaks(e, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    check_memory_leaks(test, set_intervals, set_timeouts, event_listeners, report);
                    check_memory_leaks_in_stmt(consequent, set_intervals, set_timeouts, event_listeners, report);
                    if let Some(alt) = alternate {
                        check_memory_leaks_in_stmt(alt, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    check_memory_leaks(test, set_intervals, set_timeouts, event_listeners, report);
                    check_memory_leaks_in_stmt(body, set_intervals, set_timeouts, event_listeners, report);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        check_memory_leaks_in_stmt(i, set_intervals, set_timeouts, event_listeners, report);
                    }
                    if let Some(t) = test {
                        check_memory_leaks(t, set_intervals, set_timeouts, event_listeners, report);
                    }
                    if let Some(u) = update {
                        check_memory_leaks(u, set_intervals, set_timeouts, event_listeners, report);
                    }
                    check_memory_leaks_in_stmt(body, set_intervals, set_timeouts, event_listeners, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_memory_leaks_in_stmt(s, set_intervals, set_timeouts, event_listeners, report);
                    }
                }
                _ => {}
            }
        }

        for stmt in &program.body {
            check_memory_leaks_in_stmt(stmt, &mut set_intervals, &mut set_timeouts, &mut event_listeners, report);
        }

        // 检查是否有对应的 clearInterval 调用
        let mut clear_intervals = HashSet::new();
        let mut clear_timeouts = HashSet::new();
        let mut remove_event_listeners = HashSet::new();

        fn check_clear_functions(expr: &JsExpr, clear_intervals: &mut HashSet<String>, clear_timeouts: &mut HashSet<String>, remove_event_listeners: &mut HashSet<String>) {
            match expr {
                JsExpr::Call { callee, args, .. } => {
                    if let JsExpr::Identifier(name, _, _) = &**callee {
                        if name == "clearInterval" {
                            if let Some(id_expr) = args.get(0) {
                                if let JsExpr::Identifier(id, _, _) = id_expr {
                                    clear_intervals.insert(id.clone());
                                }
                            }
                        }
                        else if name == "clearTimeout" {
                            if let Some(id_expr) = args.get(0) {
                                if let JsExpr::Identifier(id, _, _) = id_expr {
                                    clear_timeouts.insert(id.clone());
                                }
                            }
                        }
                    }
                    if let JsExpr::Member { property, computed: false, .. } = &**callee {
                        if let JsExpr::Identifier(prop_name, _, _) = &**property {
                            if prop_name == "removeEventListener" {
                                remove_event_listeners.insert("removeEventListener".to_string());
                            }
                        }
                    }
                    check_clear_functions(callee, clear_intervals, clear_timeouts, remove_event_listeners);
                    for arg in args {
                        check_clear_functions(arg, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_clear_functions(object, clear_intervals, clear_timeouts, remove_event_listeners);
                    if *computed {
                        check_clear_functions(property, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                JsExpr::Binary { left, right, .. } => {
                    check_clear_functions(left, clear_intervals, clear_timeouts, remove_event_listeners);
                    check_clear_functions(right, clear_intervals, clear_timeouts, remove_event_listeners);
                }
                JsExpr::Unary { argument, .. } => {
                    check_clear_functions(argument, clear_intervals, clear_timeouts, remove_event_listeners);
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_clear_functions(el, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_clear_functions(value, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_clear_functions(body, clear_intervals, clear_timeouts, remove_event_listeners);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_clear_functions(test, clear_intervals, clear_timeouts, remove_event_listeners);
                    check_clear_functions(consequent, clear_intervals, clear_timeouts, remove_event_listeners);
                    check_clear_functions(alternate, clear_intervals, clear_timeouts, remove_event_listeners);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_clear_functions(e, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                _ => {}
            }
        }

        fn check_clear_functions_in_stmt(stmt: &JsStmt, clear_intervals: &mut HashSet<String>, clear_timeouts: &mut HashSet<String>, remove_event_listeners: &mut HashSet<String>) {
            match stmt {
                JsStmt::Expr(expr, _, _) => check_clear_functions(expr, clear_intervals, clear_timeouts, remove_event_listeners),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        check_clear_functions(e, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        check_clear_functions(e, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    check_clear_functions(test, clear_intervals, clear_timeouts, remove_event_listeners);
                    check_clear_functions_in_stmt(consequent, clear_intervals, clear_timeouts, remove_event_listeners);
                    if let Some(alt) = alternate {
                        check_clear_functions_in_stmt(alt, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    check_clear_functions(test, clear_intervals, clear_timeouts, remove_event_listeners);
                    check_clear_functions_in_stmt(body, clear_intervals, clear_timeouts, remove_event_listeners);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        check_clear_functions_in_stmt(i, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                    if let Some(t) = test {
                        check_clear_functions(t, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                    if let Some(u) = update {
                        check_clear_functions(u, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                    check_clear_functions_in_stmt(body, clear_intervals, clear_timeouts, remove_event_listeners);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_clear_functions_in_stmt(s, clear_intervals, clear_timeouts, remove_event_listeners);
                    }
                }
                _ => {}
            }
        }

        for stmt in &program.body {
            check_clear_functions_in_stmt(stmt, &mut clear_intervals, &mut clear_timeouts, &mut remove_event_listeners);
        }

        // 检查未清理的定时器
        for interval_id in &set_intervals {
            if !clear_intervals.contains(interval_id) {
                report.add_issue(crate::types::IssueLevel::Warning, self.code(), format!("setInterval with ID '{}' may not be cleared, potential memory leak", interval_id), None);
            }
        }

        // 检查事件监听器是否有对应的移除调用
        if !event_listeners.is_empty() && remove_event_listeners.is_empty() {
            report.add_issue(crate::types::IssueLevel::Warning, self.code(), "Event listeners added but not removed, potential memory leak".to_string(), None);
        }
    }
}
