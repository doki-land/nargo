use nargo_types::{NargoValue, Result, Span};
use std::collections::{HashMap, HashSet};

/// 脚本元数据，包含信号、计算属性、属性、事件、动作和依赖关系
#[derive(Debug, Clone, Default)]
pub struct ScriptMetadata {
    /// 信号集合
    pub signals: HashSet<String>,
    /// 计算属性集合
    pub computed: HashSet<String>,
    /// 属性集合
    pub props: HashSet<String>,
    /// 事件集合
    pub emits: HashSet<String>,
    /// 动作集合
    pub actions: HashSet<String>,
    /// 依赖关系映射（变量 -> 它依赖的变量）
    pub dependencies: HashMap<String, HashSet<String>>,
}

impl ScriptMetadata {
    /// 将脚本元数据转换为 NargoValue
    pub fn to_nargo_value(&self) -> NargoValue {
        let mut map = HashMap::new();

        let signals_arr = self.signals.iter().map(|s| NargoValue::String(s.clone())).collect();
        map.insert("signals".to_string(), NargoValue::Array(signals_arr));

        let computed_arr = self.computed.iter().map(|s| NargoValue::String(s.clone())).collect();
        map.insert("computed".to_string(), NargoValue::Array(computed_arr));

        let props_arr = self.props.iter().map(|s| NargoValue::String(s.clone())).collect();
        map.insert("props".to_string(), NargoValue::Array(props_arr));

        let emits_arr = self.emits.iter().map(|s| NargoValue::String(s.clone())).collect();
        map.insert("emits".to_string(), NargoValue::Array(emits_arr));

        let actions_arr = self.actions.iter().map(|s| NargoValue::String(s.clone())).collect();
        map.insert("actions".to_string(), NargoValue::Array(actions_arr));

        let mut deps_map = HashMap::new();
        for (k, v) in &self.dependencies {
            let deps_arr = v.iter().map(|s| NargoValue::String(s.clone())).collect();
            deps_map.insert(k.clone(), NargoValue::Array(deps_arr));
        }
        map.insert("dependencies".to_string(), NargoValue::Object(deps_map));

        NargoValue::Object(map)
    }
}

/// 分析报告中的问题级别
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueLevel {
    /// 错误
    Error,
    /// 警告
    Warning,
    /// 提示
    Info,
}

/// 分析报告中的问题
#[derive(Debug, Clone)]
pub struct AnalysisIssue {
    /// 问题级别
    pub level: IssueLevel,
    /// 问题代码
    pub code: String,
    /// 问题描述
    pub message: String,
    /// 问题位置
    pub span: Option<Span>,
}

/// 分析报告
#[derive(Debug, Clone, Default)]
pub struct AnalysisReport {
    /// 分析的文件路径
    pub file_path: Option<String>,
    /// 发现的问题
    pub issues: Vec<AnalysisIssue>,
    /// 分析耗时（毫秒）
    pub duration_ms: u64,
}

impl AnalysisReport {
    /// 创建新的分析报告
    pub fn new(file_path: Option<String>) -> Self {
        Self { file_path, issues: Vec::new(), duration_ms: 0 }
    }

    /// 添加问题
    pub fn add_issue(&mut self, level: IssueLevel, code: String, message: String, span: Option<Span>) {
        self.issues.push(AnalysisIssue { level, code, message, span });
    }

    /// 按级别排序问题
    pub fn sort_issues(&mut self) {
        self.issues.sort_by(|a, b| a.level.cmp(&b.level));
    }

    /// 按规则类型分组问题
    pub fn group_issues_by_rule(&self) -> HashMap<String, Vec<&AnalysisIssue>> {
        let mut grouped = HashMap::new();
        for issue in &self.issues {
            grouped.entry(issue.code.clone()).or_insert(Vec::new()).push(issue);
        }
        grouped
    }

    /// 生成控制台报告
    pub fn generate_console_report(&self) -> String {
        let mut report = String::new();

        // 报告标题
        if let Some(file_path) = &self.file_path {
            report.push_str(&format!("========================================\n"));
            report.push_str(&format!("ANALYSIS REPORT FOR: {}\n", file_path));
            report.push_str(&format!("========================================\n"));
        }
        else {
            report.push_str(&format!("========================================\n"));
            report.push_str("ANALYSIS REPORT\n");
            report.push_str(&format!("========================================\n"));
        }

        // 基本统计信息
        report.push_str(&format!("Duration: {}ms\n", self.duration_ms));
        report.push_str(&format!("Total issues: {}\n", self.issues.len()));

        let error_count = self.issues.iter().filter(|i| i.level == IssueLevel::Error).count();
        let warning_count = self.issues.iter().filter(|i| i.level == IssueLevel::Warning).count();
        let info_count = self.issues.iter().filter(|i| i.level == IssueLevel::Info).count();

        report.push_str(&format!("Errors: {}, Warnings: {}, Info: {}\n\n", error_count, warning_count, info_count));

        // 按规则类型分组显示
        let grouped_issues = self.group_issues_by_rule();
        if !grouped_issues.is_empty() {
            report.push_str("ISSUES BY RULE:\n");
            report.push_str("----------------------------------------\n");

            for (rule_code, issues) in grouped_issues {
                report.push_str(&format!("{} ({})\n", rule_code, issues.len()));
                for issue in issues {
                    let level_str = match issue.level {
                        IssueLevel::Error => "ERROR",
                        IssueLevel::Warning => "WARNING",
                        IssueLevel::Info => "INFO",
                    };

                    report.push_str(&format!("  - [{}] {}", level_str, issue.message));
                    if let Some(span) = &issue.span {
                        report.push_str(&format!(" (line {}, col {})", span.start.line, span.start.column));
                    }
                    report.push_str("\n");
                }
                report.push_str("\n");
            }
        }

        // 详细问题列表
        if !self.issues.is_empty() {
            report.push_str("DETAILED ISSUES LIST:\n");
            report.push_str("----------------------------------------\n");

            for (i, issue) in self.issues.iter().enumerate() {
                let level_str = match issue.level {
                    IssueLevel::Error => "ERROR",
                    IssueLevel::Warning => "WARNING",
                    IssueLevel::Info => "INFO",
                };

                report.push_str(&format!("{}. [{}] [{}] {}", i + 1, level_str, issue.code, issue.message));
                if let Some(span) = &issue.span {
                    report.push_str(&format!("\n   Location: line {}, column {} - line {}, column {}", span.start.line, span.start.column, span.end.line, span.end.column));
                }
                report.push_str("\n\n");
            }
        }

        report
    }

    /// 生成 JSON 报告
    pub fn generate_json_report(&self) -> Result<NargoValue> {
        let mut issues = Vec::new();

        for issue in &self.issues {
            let mut issue_map = HashMap::new();
            issue_map.insert(
                "level".to_string(),
                NargoValue::String(match issue.level {
                    IssueLevel::Error => "error".to_string(),
                    IssueLevel::Warning => "warning".to_string(),
                    IssueLevel::Info => "info".to_string(),
                }),
            );
            issue_map.insert("code".to_string(), NargoValue::String(issue.code.clone()));
            issue_map.insert("message".to_string(), NargoValue::String(issue.message.clone()));

            if let Some(span) = &issue.span {
                let mut span_map = HashMap::new();
                span_map.insert("start_line".to_string(), NargoValue::Number(span.start.line as f64));
                span_map.insert("start_column".to_string(), NargoValue::Number(span.start.column as f64));
                span_map.insert("end_line".to_string(), NargoValue::Number(span.end.line as f64));
                span_map.insert("end_column".to_string(), NargoValue::Number(span.end.column as f64));
                issue_map.insert("span".to_string(), NargoValue::Object(span_map));
            }

            issues.push(NargoValue::Object(issue_map));
        }

        // 统计信息
        let error_count = self.issues.iter().filter(|i| i.level == IssueLevel::Error).count();
        let warning_count = self.issues.iter().filter(|i| i.level == IssueLevel::Warning).count();
        let info_count = self.issues.iter().filter(|i| i.level == IssueLevel::Info).count();

        // 按规则分组的统计
        let mut rule_stats = HashMap::new();
        for issue in &self.issues {
            let count = rule_stats.entry(issue.code.clone()).or_insert(0);
            *count += 1;
        }

        let mut rule_stats_nargo = HashMap::new();
        for (rule, count) in rule_stats {
            rule_stats_nargo.insert(rule, NargoValue::Number(count as f64));
        }

        let mut report_map = HashMap::new();
        if let Some(file_path) = &self.file_path {
            report_map.insert("file_path".to_string(), NargoValue::String(file_path.clone()));
        }
        report_map.insert("issues".to_string(), NargoValue::Array(issues));
        report_map.insert("duration_ms".to_string(), NargoValue::Number(self.duration_ms as f64));
        report_map.insert("total_issues".to_string(), NargoValue::Number(self.issues.len() as f64));
        report_map.insert("error_count".to_string(), NargoValue::Number(error_count as f64));
        report_map.insert("warning_count".to_string(), NargoValue::Number(warning_count as f64));
        report_map.insert("info_count".to_string(), NargoValue::Number(info_count as f64));
        report_map.insert("rule_stats".to_string(), NargoValue::Object(rule_stats_nargo));

        Ok(NargoValue::Object(report_map))
    }

    /// 保存报告到文件
    pub fn save_to_file(&self, output_path: &str) -> Result<()> {
        let json_report = self.generate_json_report()?;
        let report_str = serde_json::to_string_pretty(&json_report).map_err(|e| nargo_types::Error::external_error("serde_json".to_string(), e.to_string(), Span::default()))?;

        let path = std::path::Path::new(output_path);
        let mut file = std::fs::File::create(path)?;
        std::io::Write::write(&mut file, report_str.as_bytes())?;

        Ok(())
    }

    /// 生成摘要报告
    pub fn generate_summary(&self) -> String {
        let total_issues = self.issues.len();
        let error_count = self.issues.iter().filter(|i| i.level == IssueLevel::Error).count();
        let warning_count = self.issues.iter().filter(|i| i.level == IssueLevel::Warning).count();
        let info_count = self.issues.iter().filter(|i| i.level == IssueLevel::Info).count();

        format!("Analysis Summary: {total_issues} issues found ({error_count} errors, {warning_count} warnings, {info_count} info) in {duration}ms", total_issues = total_issues, error_count = error_count, warning_count = warning_count, info_count = info_count, duration = self.duration_ms)
    }
}
