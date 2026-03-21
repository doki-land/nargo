#![warn(missing_docs)]

use crate::error::{TemplateError, TemplateResult};
use crate::ir::{BinaryOp, UnaryOp};
use nargo_types::NargoValue;

/// 判断值是否为"真值"
///
/// 模板引擎中的真值判断规则：
/// - `Null` → 假
/// - `Bool(false)` → 假
/// - `Number(0.0)` → 假
/// - `String("")` → 假
/// - `Array([])` → 假
/// - `Object({})` → 假
/// - 其他值 → 真
pub fn is_truthy(value: &NargoValue) -> bool {
    match value {
        NargoValue::Null => false,
        NargoValue::Bool(b) => *b,
        NargoValue::Number(n) => *n != 0.0,
        NargoValue::String(s) => !s.is_empty(),
        NargoValue::Array(arr) => !arr.is_empty(),
        NargoValue::Object(map) => !map.is_empty(),
        NargoValue::Signal(s) => !s.is_empty(),
        NargoValue::Binary(b) => !b.is_empty(),
        NargoValue::Raw(s) => !s.is_empty(),
        NargoValue::Ref(s) => !s.is_empty(),
    }
}

/// 将 NargoValue 转换为显示字符串
///
/// 根据值的类型进行转换：
/// - `String` → 内部字符串
/// - `Number` → 数字字符串（整数为整数格式）
/// - `Bool` → `"true"` 或 `"false"`
/// - `Null` → 空字符串
/// - `Array` → 逗号分隔的值
/// - `Object` → 调试格式
/// - `Signal`/`Raw`/`Ref` → 内部字符串
/// - `Binary` → 空字符串
pub fn value_to_string(value: &NargoValue) -> String {
    match value {
        NargoValue::String(s) => s.clone(),
        NargoValue::Number(n) => {
            if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        NargoValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        NargoValue::Null => String::new(),
        NargoValue::Array(arr) => arr.iter().map(value_to_string).collect::<Vec<_>>().join(", "),
        NargoValue::Object(map) => {
            let pairs: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, value_to_string(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        NargoValue::Signal(s) => s.clone(),
        NargoValue::Raw(s) => s.clone(),
        NargoValue::Ref(s) => s.clone(),
        NargoValue::Binary(_) => String::new(),
    }
}

/// 判断两个值是否相等
///
/// 支持基本类型的深度相等比较：Null、Bool、Number、String、Array、Object。
/// 不同类型之间比较返回 `false`。
pub fn values_equal(left: &NargoValue, right: &NargoValue) -> bool {
    match (left, right) {
        (NargoValue::Null, NargoValue::Null) => true,
        (NargoValue::Bool(l), NargoValue::Bool(r)) => l == r,
        (NargoValue::Number(l), NargoValue::Number(r)) => l == r,
        (NargoValue::String(l), NargoValue::String(r)) => l == r,
        (NargoValue::Array(l), NargoValue::Array(r)) => {
            if l.len() != r.len() {
                return false;
            }
            for (a, b) in l.iter().zip(r.iter()) {
                if !values_equal(a, b) {
                    return false;
                }
            }
            true
        }
        (NargoValue::Object(l), NargoValue::Object(r)) => {
            if l.len() != r.len() {
                return false;
            }
            for (k, v) in l {
                match r.get(k) {
                    Some(rv) if values_equal(v, rv) => {}
                    _ => return false,
                }
            }
            true
        }
        _ => false,
    }
}

/// 执行二元运算
///
/// 根据 BinaryOp 和左右操作数执行具体的运算逻辑。
/// And 和 Or 实现短路求值。
pub fn evaluate_binary_op(op: &BinaryOp, left: &NargoValue, right: &NargoValue) -> TemplateResult<NargoValue> {
    match op {
        BinaryOp::Eq => Ok(NargoValue::Bool(values_equal(left, right))),
        BinaryOp::Ne => Ok(NargoValue::Bool(!values_equal(left, right))),
        BinaryOp::Lt => {
            let (l, r) = expect_numbers(left, right, "<")?;
            Ok(NargoValue::Bool(l < r))
        }
        BinaryOp::Le => {
            let (l, r) = expect_numbers(left, right, "<=")?;
            Ok(NargoValue::Bool(l <= r))
        }
        BinaryOp::Gt => {
            let (l, r) = expect_numbers(left, right, ">")?;
            Ok(NargoValue::Bool(l > r))
        }
        BinaryOp::Ge => {
            let (l, r) = expect_numbers(left, right, ">=")?;
            Ok(NargoValue::Bool(l >= r))
        }
        BinaryOp::Add => match (left, right) {
            (NargoValue::Number(l), NargoValue::Number(r)) => Ok(NargoValue::Number(l + r)),
            (NargoValue::String(l), NargoValue::String(r)) => {
                Ok(NargoValue::String(format!("{}{}", l, r)))
            }
            (NargoValue::String(l), right_val) => {
                Ok(NargoValue::String(format!("{}{}", l, value_to_string(right_val))))
            }
            (left_val, NargoValue::String(r)) => {
                Ok(NargoValue::String(format!("{}{}", value_to_string(left_val), r)))
            }
            _ => Err(TemplateError::Render("Operator + requires numbers or strings".to_string())),
        },
        BinaryOp::Sub => {
            let (l, r) = expect_numbers(left, right, "-")?;
            Ok(NargoValue::Number(l - r))
        }
        BinaryOp::Mul => {
            let (l, r) = expect_numbers(left, right, "*")?;
            Ok(NargoValue::Number(l * r))
        }
        BinaryOp::Div => {
            let (l, r) = expect_numbers(left, right, "/")?;
            if r == 0.0 {
                return Err(TemplateError::Render("Division by zero".to_string()));
            }
            Ok(NargoValue::Number(l / r))
        }
        BinaryOp::Mod => {
            let (l, r) = expect_numbers(left, right, "%")?;
            if r == 0.0 {
                return Err(TemplateError::Render("Modulo by zero".to_string()));
            }
            Ok(NargoValue::Number(l % r))
        }
        BinaryOp::And => {
            if !is_truthy(left) {
                Ok(NargoValue::Bool(false))
            } else {
                Ok(NargoValue::Bool(is_truthy(right)))
            }
        }
        BinaryOp::Or => {
            if is_truthy(left) {
                Ok(NargoValue::Bool(true))
            } else {
                Ok(NargoValue::Bool(is_truthy(right)))
            }
        }
    }
}

/// 执行一元运算
///
/// Not 对值取逻辑非，Neg 对数字取负。
pub fn evaluate_unary_op(op: &UnaryOp, val: &NargoValue) -> TemplateResult<NargoValue> {
    match op {
        UnaryOp::Not => Ok(NargoValue::Bool(!is_truthy(val))),
        UnaryOp::Neg => match val {
            NargoValue::Number(n) => Ok(NargoValue::Number(-n)),
            _ => Err(TemplateError::Render("Unary minus requires a number operand".to_string())),
        },
    }
}

/// 期望两个数值操作数
///
/// 辅助函数，将左右操作数提取为 f64，若任一不是数字则返回错误。
pub fn expect_numbers(left: &NargoValue, right: &NargoValue, op_name: &str) -> TemplateResult<(f64, f64)> {
    match (left.as_number(), right.as_number()) {
        (Some(l), Some(r)) => Ok((l, r)),
        _ => Err(TemplateError::Render(format!(
            "Operator {} requires number operands",
            op_name
        ))),
    }
}
