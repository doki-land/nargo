#![warn(missing_docs)]

use crate::error::{TemplateError, TemplateResult};
use nargo_types::NargoValue;

use super::value::{is_truthy, value_to_string};

/// 应用过滤器
///
/// 对已求值的输入值应用内置过滤器函数。
/// 支持的过滤器：upper、lower、trim、length、default、join、first、last、reverse、sort、escape。
///
/// 与旧版不同，参数已经是求值后的 NargoValue 而非待求值的表达式。
pub fn apply_filter(
    name: &str,
    value: &NargoValue,
    args: &[NargoValue],
) -> TemplateResult<NargoValue> {
    match name {
        "upper" => match value {
            NargoValue::String(s) => Ok(NargoValue::String(s.to_uppercase())),
            _ => Err(TemplateError::Render("Filter 'upper' requires a string".to_string())),
        },
        "lower" => match value {
            NargoValue::String(s) => Ok(NargoValue::String(s.to_lowercase())),
            _ => Err(TemplateError::Render("Filter 'lower' requires a string".to_string())),
        },
        "trim" => match value {
            NargoValue::String(s) => Ok(NargoValue::String(s.trim().to_string())),
            _ => Err(TemplateError::Render("Filter 'trim' requires a string".to_string())),
        },
        "length" => match value {
            NargoValue::String(s) => Ok(NargoValue::Number(s.len() as f64)),
            NargoValue::Array(arr) => Ok(NargoValue::Number(arr.len() as f64)),
            NargoValue::Object(map) => Ok(NargoValue::Number(map.len() as f64)),
            _ => Err(TemplateError::Render(
                "Filter 'length' requires a string, array, or object".to_string(),
            )),
        },
        "default" => {
            if is_truthy(value) {
                Ok(value.clone())
            } else if let Some(arg) = args.first() {
                Ok(arg.clone())
            } else {
                Ok(NargoValue::String(String::new()))
            }
        }
        "join" => match value {
            NargoValue::Array(arr) => {
                let separator = if let Some(arg) = args.first() {
                    value_to_string(arg)
                } else {
                    ", ".to_string()
                };
                let joined: String = arr.iter().map(value_to_string).collect::<Vec<_>>().join(&separator);
                Ok(NargoValue::String(joined))
            }
            _ => Err(TemplateError::Render("Filter 'join' requires an array".to_string())),
        },
        "first" => match value {
            NargoValue::Array(arr) => Ok(arr.first().cloned().unwrap_or(NargoValue::Null)),
            _ => Err(TemplateError::Render("Filter 'first' requires an array".to_string())),
        },
        "last" => match value {
            NargoValue::Array(arr) => Ok(arr.last().cloned().unwrap_or(NargoValue::Null)),
            _ => Err(TemplateError::Render("Filter 'last' requires an array".to_string())),
        },
        "reverse" => match value {
            NargoValue::Array(arr) => Ok(NargoValue::Array(arr.iter().rev().cloned().collect())),
            NargoValue::String(s) => Ok(NargoValue::String(s.chars().rev().collect())),
            _ => Err(TemplateError::Render(
                "Filter 'reverse' requires an array or string".to_string(),
            )),
        },
        "sort" => match value {
            NargoValue::Array(arr) => {
                let mut sorted = arr.clone();
                sorted.sort_by(|a, b| {
                    let a_str = value_to_string(a);
                    let b_str = value_to_string(b);
                    a_str.cmp(&b_str)
                });
                Ok(NargoValue::Array(sorted))
            }
            _ => Err(TemplateError::Render("Filter 'sort' requires an array".to_string())),
        },
        "escape" => match value {
            NargoValue::String(s) => Ok(NargoValue::String(
                s.replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
                    .replace('\'', "&#39;"),
            )),
            _ => Ok(value.clone()),
        },
        _ => Err(TemplateError::Render(format!("Unknown filter: {}", name))),
    }
}
