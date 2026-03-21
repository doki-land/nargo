#![warn(missing_docs)]

use nargo_types::NargoValue;
use std::collections::HashMap;

/// 渲染上下文
///
/// 使用作用域栈管理变量查找。子作用域（如 for 循环体）
/// 可以访问父作用域中的变量，但子作用域中的变量不会泄漏到父作用域。
pub struct RenderContext {
    /// 作用域栈，从底到顶依次为全局作用域到当前作用域
    scopes: Vec<HashMap<String, NargoValue>>,
}

impl RenderContext {
    /// 从 NargoValue 创建渲染上下文
    ///
    /// 若值为 Object 则使用其内部映射作为全局作用域，否则创建空全局作用域。
    pub fn from_value(value: &NargoValue) -> Self {
        let data = match value {
            NargoValue::Object(map) => map.clone(),
            _ => HashMap::new(),
        };
        Self { scopes: vec![data] }
    }

    /// 创建空的渲染上下文
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()] }
    }

    /// 压入新的变量作用域
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// 弹出当前变量作用域
    ///
    /// 不会弹出全局作用域（最底层）。
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// 获取变量值
    ///
    /// 从当前作用域向上查找，返回第一个匹配的值。
    pub fn get(&self, key: &str) -> Option<&NargoValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(key) {
                return Some(v);
            }
        }
        None
    }

    /// 在当前（最顶层）作用域中设置变量
    pub fn set(&mut self, key: String, value: NargoValue) {
        self.scopes.last_mut().unwrap().insert(key, value);
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}
