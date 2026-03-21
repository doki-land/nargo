#![warn(missing_docs)]

use nargo_types::NargoValue;

/// 模板上下文 trait
/// 定义模板数据的统一接口
pub trait TemplateContext: std::fmt::Debug + Clone + std::any::Any {
    /// 转换为 Any 类型，用于类型检查
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 将类型转换为 NargoValue 的 trait
pub trait ToNargoValue {
    /// 将自身转换为 NargoValue
    fn to_nargo_value(&self) -> NargoValue;
}

impl ToNargoValue for String {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::String(self.clone())
    }
}

impl ToNargoValue for &str {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::String(self.to_string())
    }
}

impl ToNargoValue for i64 {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::Number(*self as f64)
    }
}

impl ToNargoValue for i32 {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::Number(*self as f64)
    }
}

impl ToNargoValue for f64 {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::Number(*self)
    }
}

impl ToNargoValue for bool {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::Bool(*self)
    }
}

impl ToNargoValue for usize {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::Number(*self as f64)
    }
}

impl<T: ToNargoValue> ToNargoValue for Vec<T> {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::Array(self.iter().map(|v| v.to_nargo_value()).collect())
    }
}

impl<V: ToNargoValue> ToNargoValue for std::collections::HashMap<String, V> {
    fn to_nargo_value(&self) -> NargoValue {
        let mut map = std::collections::HashMap::new();
        for (k, v) in self {
            map.insert(k.clone(), v.to_nargo_value());
        }
        NargoValue::Object(map)
    }
}

impl<T: ToNargoValue> ToNargoValue for Option<T> {
    fn to_nargo_value(&self) -> NargoValue {
        match self {
            Some(v) => v.to_nargo_value(),
            None => NargoValue::Null,
        }
    }
}

impl ToNargoValue for std::path::PathBuf {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::String(self.to_string_lossy().to_string())
    }
}
