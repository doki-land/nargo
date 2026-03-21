#![warn(missing_docs)]

//! Nargo 模板引擎

mod context;
mod engine;
mod manager;

/// 模板引擎错误类型
pub mod error;
/// 语言前端与模板适配器
pub mod frontend;
/// 编译后模板 IR 定义
pub mod ir;
/// 栈式模板虚拟机
pub mod vm;

pub use context::ToNargoValue;
pub use error::{TemplateError, TemplateResult};
pub use engine::TemplateEngine;
pub use manager::TemplateManager;
pub use frontend::Frontend;
pub use frontend::TemplateAdapter;
pub use ir::TemplateIR;
pub use ir::Instruction;
pub use ir::IRBuilder;
pub use vm::VM;

#[cfg(feature = "dejavu")]
pub use frontend::dejavu::DejaVuFrontend;
#[cfg(feature = "dejavu")]
pub type DejaVuAdapter = TemplateAdapter<DejaVuFrontend>;
#[cfg(feature = "dejavu")]
pub use frontend::UnifiedTemplateEngine;
#[cfg(feature = "ejs")]
pub use frontend::ejs::EjsFrontend;
#[cfg(feature = "handlebars")]
pub use frontend::handlebars::HandlebarsFrontend;
#[cfg(feature = "jinja")]
pub use frontend::jinja2::Jinja2Frontend;
#[cfg(feature = "liquid")]
pub use frontend::liquid::LiquidFrontend;
