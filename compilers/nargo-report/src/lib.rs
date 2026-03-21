#![warn(missing_docs)]

use nargo_types::{Error, NargoValue, Result};
use serde::Serialize;
use std::path::Path;

use nargo_audit::AuditIssue;
use nargo_linter::Diagnostic;
use nargo_template::{TemplateEngine, TemplateManager};

/// 报告数据结构
///
/// 包含所有类型的报告结果，用于生成 HTML 报告。
#[derive(Serialize)]
pub struct ReportData {
    /// 报告标题
    pub title: String,
    /// 生成时间戳
    pub timestamp: String,
    ///  lint 结果
    pub lint_results: Vec<LintFileResult>,
    /// 测试结果
    pub test_results: Vec<TestResultData>,
    /// 格式化结果
    pub format_results: Vec<FormatFileResult>,
    /// 安全审计结果
    pub audit_results: Vec<AuditIssue>,
    /// 覆盖率摘要
    pub coverage_summary: Option<CoverageSummary>,
    /// 构建产物分析
    pub artifact_analysis: Option<NargoValue>,
}

/// 单个文件的 lint 结果
#[derive(Serialize)]
pub struct LintFileResult {
    /// 文件路径
    pub file: String,
    /// 诊断信息
    pub diagnostics: Vec<Diagnostic>,
}

/// 测试结果数据
#[derive(Serialize)]
pub struct TestResultData {
    /// 测试文件路径
    pub file: String,
    /// 是否测试通过
    pub success: bool,
    /// 测试消息（如果有）
    pub message: Option<String>,
}

/// 格式化结果
#[derive(Serialize)]
pub struct FormatFileResult {
    /// 文件路径
    pub file: String,
    /// 是否已格式化
    pub formatted: bool,
}

/// 覆盖率摘要
#[derive(Serialize)]
pub struct CoverageSummary {
    /// 总语句数
    pub total_statements: usize,
    /// 覆盖的语句数
    pub covered_statements: usize,
    /// 覆盖率百分比
    pub percentage: f64,
}

/// Nargo 报告生成器
///
/// 用于生成 HTML 格式的报告，包含各种分析结果。
pub struct NargoReporter {
    template_manager: TemplateManager,
}

impl NargoReporter {
    /// 创建一个新的报告生成器
    pub fn new() -> Self {
        let template_manager = TemplateManager::new();
        Self { template_manager }
    }

    /// 生成 HTML 报告
    ///
    /// # Arguments
    ///
    /// * `data` - 报告数据
    /// * `output_path` - 输出文件路径
    ///
    /// # Returns
    ///
    /// 生成结果
    pub fn generate_html(&self, data: &ReportData, output_path: &Path) -> Result<()> {
        // Create HTML template manually
        let html = self.generate_html_template(data);
        std::fs::write(output_path, html)?;
        Ok(())
    }

    /// 生成 HTML 模板内容
    fn generate_html_template(&self, data: &ReportData) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #333; max-width: 1200px; margin: 0 auto; padding: 20px; background: #f4f7f6; }}
        .card {{ background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); }}
        h1 {{ color: #2c3e50; border-bottom: 2px solid #eee; padding-bottom: 10px; }}
        h2 {{ color: #34495e; margin-top: 30px; }}
        .summary {{ display: flex; gap: 20px; flex-wrap: wrap; }}
        .stat {{ flex: 1; min-width: 150px; text-align: center; padding: 15px; border-radius: 8px; color: white; }}
        .stat-success {{ background: #27ae60; }}
        .stat-error {{ background: #e74c3c; }}
        .stat-info {{ background: #3498db; }}
        .stat-value {{ font-size: 24px; font-weight: bold; display: block; }}
        .stat-label {{ font-size: 14px; opacity: 0.9; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 10px; }}
        th, td {{ text-align: left; padding: 12px; border-bottom: 1px solid #eee; }}
        th {{ background: #f8f9fa; font-weight: 600; }}
        .success {{ color: #27ae60; font-weight: bold; }}
        .error {{ color: #e74c3c; font-weight: bold; }}
        .warning {{ color: #f39c12; font-weight: bold; }}
        pre {{ background: #f8f9fa; padding: 10px; border-radius: 4px; overflow-x: auto; font-size: 13px; }}
        .progress-bar {{ height: 10px; background: #eee; border-radius: 5px; overflow: hidden; margin-top: 5px; }}
        .progress-fill {{ height: 100%; background: #27ae60; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>Generated at: {}</p>

    {}

    {}

    {}

    {}

    {}

    {}
</body>
</html>
"#,
            data.title,
            data.title,
            data.timestamp,
            self.render_artifact_analysis(data.artifact_analysis.as_ref()),
            self.render_test_results(&data.test_results),
            self.render_coverage_summary(data.coverage_summary.as_ref()),
            self.render_lint_results(&data.lint_results),
            self.render_format_results(&data.format_results),
            self.render_audit_results(&data.audit_results)
        )
    }

    /// 渲染构建产物分析
    fn render_artifact_analysis(&self, artifact_analysis: Option<&NargoValue>) -> String {
        match artifact_analysis {
            Some(analysis) => {
                let file_count = analysis.get("file_count").and_then(|v| v.as_number()).map(|n| n as u64).unwrap_or(0);
                let total_size = analysis.get("total_size").and_then(|v| v.as_number()).map(|n| n as u64).unwrap_or(0);

                let empty_files = vec![];
                let files = analysis.get("files").and_then(|v| v.as_array()).unwrap_or(&empty_files);
                let file_rows = files
                    .iter()
                    .map(|file| {
                        let path = file.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        let size = file.get("size").and_then(|v| v.as_number()).map(|n| n as u64).unwrap_or(0);
                        let file_type = file.get("file_type").and_then(|v| v.as_str()).unwrap_or("");
                        format!(
                            "                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>",
                            path, size, file_type
                        )
                    })
                    .collect::<String>();

                format!(
                    r#"
    <div class="card">
        <h2>🏗️ Artifact Analysis</h2>
        <div class="summary">
            <div class="stat stat-info">
                <span class="stat-value">{}</span>
                <span class="stat-label">Total Files</span>
            </div>
            <div class="stat stat-info">
                <span class="stat-value{} B</span>
                <span class="stat-label">Total Size</span>
            </div>
        </div>
        <table>
            <thead>
                <tr>
                    <th>File Path</th>
                    <th>Size (Bytes)</th>
                    <th>Type</th>
                </tr>
            </thead>
            <tbody>
{}
            </tbody>
        </table>
    </div>
"#,
                    file_count, total_size, file_rows
                )
            }
            None => "".to_string(),
        }
    }

    /// 渲染测试结果
    fn render_test_results(&self, test_results: &[TestResultData]) -> String {
        if test_results.is_empty() {
            return "".to_string();
        }

        let test_rows = test_results
            .iter()
            .map(|test| {
                let status_class = if test.success { "success" } else { "error" };
                let status_text = if test.success { "PASSED" } else { "FAILED" };
                let message = test.message.as_ref().map(|msg| format!("<pre>{}</pre>", msg)).unwrap_or("".to_string());
                format!(
                    "                <tr>
                    <td>{}</td>
                    <td><span class='{}'>{}</span></td>
                    <td>{}</td>
                </tr>",
                    test.file, status_class, status_text, message
                )
            })
            .collect::<String>();

        format!(
            r#"
    <div class="card">
        <h2>🧪 Test Results</h2>
        <table>
            <thead>
                <tr>
                    <th>Test File</th>
                    <th>Status</th>
                    <th>Message</th>
                </tr>
            </thead>
            <tbody>
{}
            </tbody>
        </table>
    </div>
"#,
            test_rows
        )
    }

    /// 渲染覆盖率摘要
    fn render_coverage_summary(&self, coverage: Option<&CoverageSummary>) -> String {
        match coverage {
            Some(coverage) => format!(
                r#"
    <div class="card">
        <h2>📊 Coverage Summary</h2>
        <div class="stat-value">{}%</div>
        <div class="progress-bar">
            <div class="progress-fill" style="width: {}%"></div>
        </div>
        <p>{} / {} statements covered</p>
    </div>
"#,
                coverage.percentage, coverage.percentage, coverage.covered_statements, coverage.total_statements
            ),
            None => "".to_string(),
        }
    }

    /// 渲染 lint 结果
    fn render_lint_results(&self, lint_results: &[LintFileResult]) -> String {
        if lint_results.is_empty() {
            return "".to_string();
        }

        let lint_rows = lint_results
            .iter()
            .map(|lint| {
                let diagnostics = lint.diagnostics.iter().map(|diag| format!("                        <div class='{}'>[{}] {} (line {}, col {})</div>", diag.severity, diag.severity, diag.message, diag.line, diag.column)).collect::<String>();
                format!(
                    "                <tr>
                    <td>{}</td>
                    <td>
{}
                    </td>
                </tr>",
                    lint.file, diagnostics
                )
            })
            .collect::<String>();

        format!(
            r#"
    <div class="card">
        <h2>🔍 Lint Results</h2>
        <table>
            <thead>
                <tr>
                    <th>File</th>
                    <th>Diagnostics</th>
                </tr>
            </thead>
            <tbody>
{}
            </tbody>
        </table>
    </div>
"#,
            lint_rows
        )
    }

    /// 渲染格式化结果
    fn render_format_results(&self, format_results: &[FormatFileResult]) -> String {
        if format_results.is_empty() {
            return "".to_string();
        }

        let format_rows = format_results
            .iter()
            .map(|format| {
                let status_class = if format.formatted { "success" } else { "error" };
                let status_text = if format.formatted { "FORMATTED" } else { "UNFORMATTED" };
                format!(
                    "                <tr>
                    <td>{}</td>
                    <td><span class='{}'>{}</span></td>
                </tr>",
                    format.file, status_class, status_text
                )
            })
            .collect::<String>();

        format!(
            r#"
    <div class="card">
        <h2>🎨 Formatting</h2>
        <table>
            <thead>
                <tr>
                    <th>File</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
{}
            </tbody>
        </table>
    </div>
"#,
            format_rows
        )
    }

    /// 渲染安全审计结果
    fn render_audit_results(&self, audit_results: &[AuditIssue]) -> String {
        if audit_results.is_empty() {
            return "".to_string();
        }

        let audit_rows = audit_results
            .iter()
            .map(|audit| {
                format!(
                    "                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{} (line {})</td>
                    <td><span class='{}'>[{}]</span></td>
                </tr>",
                    audit.category,
                    audit.file.display(),
                    audit.message,
                    audit.line,
                    audit.severity,
                    audit.severity
                )
            })
            .collect::<String>();

        format!(
            r#"
    <div class="card">
        <h2>🛡️ Security Audit</h2>
        <table>
            <thead>
                <tr>
                    <th>Category</th>
                    <th>File</th>
                    <th>Issue</th>
                    <th>Severity</th>
                </tr>
            </thead>
            <tbody>
{}
            </tbody>
        </table>
    </div>
"#,
            audit_rows
        )
    }
}
