use serde::{Deserialize, Serialize};

use super::dependency::default_true;

/// Formatting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// Enable formatting.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Indentation width.
    #[serde(default = "default_indent_width")]
    pub indent_width: u8,

    /// Use tabs for indentation.
    #[serde(default)]
    pub use_tabs: bool,

    /// Line width.
    #[serde(default = "default_line_width")]
    pub line_width: u16,

    /// Quote style.
    #[serde(default)]
    pub quote_style: QuoteStyle,

    /// Semicolons.
    #[serde(default = "default_true")]
    pub semicolons: bool,

    /// Trailing commas.
    #[serde(default)]
    pub trailing_commas: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self { enabled: true, indent_width: 2, use_tabs: false, line_width: 80, quote_style: QuoteStyle::Double, semicolons: true, trailing_commas: true }
    }
}

/// Returns the default indent width.
pub fn default_indent_width() -> u8 {
    2
}

/// Returns the default line width.
pub fn default_line_width() -> u16 {
    80
}

/// Quote style preference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QuoteStyle {
    /// Single quotes.
    Single,
    /// Double quotes.
    Double,
    /// Auto - prefer single but use double when needed.
    Auto,
}

impl Default for QuoteStyle {
    fn default() -> Self {
        QuoteStyle::Double
    }
}
