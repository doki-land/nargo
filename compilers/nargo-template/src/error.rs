#![warn(missing_docs)]

/// 模板引擎结果类型
pub type TemplateResult<T> = std::result::Result<T, TemplateError>;

/// 模板引擎错误类型
#[derive(Debug)]
pub enum TemplateError {
    /// IO 错误
    Io(std::io::Error),
    /// 渲染错误
    Render(String),
    /// 模板未找到
    TemplateNotFound(String),
    /// 语法错误
    Syntax(String),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::Io(e) => write!(f, "IO error: {}", e),
            TemplateError::Render(msg) => write!(f, "Render error: {}", msg),
            TemplateError::TemplateNotFound(name) => write!(f, "Template not found: {}", name),
            TemplateError::Syntax(msg) => write!(f, "Syntax error: {}", msg),
        }
    }
}

impl std::error::Error for TemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TemplateError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TemplateError {
    fn from(e: std::io::Error) -> Self {
        TemplateError::Io(e)
    }
}

impl From<walkdir::Error> for TemplateError {
    fn from(e: walkdir::Error) -> Self {
        if let Some(io_err) = e.io_error() {
            TemplateError::Io(std::io::Error::new(io_err.kind(), e.to_string()))
        } else {
            TemplateError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }
    }
}
