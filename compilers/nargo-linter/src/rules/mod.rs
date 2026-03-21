//! Lint rules for Nargo.

pub mod no_alert;
pub mod no_console;
pub mod no_debugger;
pub mod no_deprecated_tags;
pub mod no_empty_template;
pub mod no_magic_numbers;
pub mod no_undeclared_variables;
pub mod no_unreachable_code;
pub mod no_unused_imports;
pub mod no_unused_vars;

pub use no_alert::NoAlert;
pub use no_console::NoConsole;
pub use no_debugger::NoDebugger;
pub use no_deprecated_tags::NoDeprecatedTags;
pub use no_empty_template::NoEmptyTemplate;
pub use no_magic_numbers::NoMagicNumbers;
pub use no_undeclared_variables::NoUndeclaredVariables;
pub use no_unreachable_code::NoUnreachableCode;
pub use no_unused_imports::NoUnusedImports;
pub use no_unused_vars::NoUnusedVars;
