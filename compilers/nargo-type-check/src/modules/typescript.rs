//! TypeScript 语句处理模块
//!
//! 包含 TypeScript 特有的语句处理相关的代码

use regex::Regex;
use std::collections::HashMap;

use crate::modules::types::{Type, TypeEnv};

/// TypeScript 语句处理器
pub trait TypeScriptStmtHandler {
    /// 处理 TypeScript 特有的语句
    ///
    /// # 参数
    ///
    /// * `stmt_str` - 语句字符串
    fn handle_typescript_stmt(&mut self, stmt_str: &str);

    /// 处理接口定义
    ///
    /// # 参数
    ///
    /// * `name` - 接口名称
    /// * `members` - 接口成员
    /// * `extends` - 继承的接口
    fn handle_interface_def(&mut self, name: String, members: HashMap<String, Type>, extends: Vec<String>);

    /// 处理类型别名
    ///
    /// # 参数
    ///
    /// * `name` - 类型别名名称
    /// * `ty` - 类型
    fn handle_type_alias(&mut self, name: String, ty: Type);

    /// 处理泛型类型
    ///
    /// # 参数
    ///
    /// * `name` - 类型名称
    /// * `type_args` - 类型参数
    ///
    /// # 返回
    ///
    /// 处理后的类型
    fn handle_generic_type(&self, name: String, type_args: Vec<Type>) -> Type;

    /// 处理联合类型
    ///
    /// # 参数
    ///
    /// * `types` - 联合的类型
    ///
    /// # 返回
    ///
    /// 联合类型
    fn handle_union_type(&self, types: Vec<Type>) -> Type;

    /// 处理交叉类型
    ///
    /// # 参数
    ///
    /// * `types` - 交叉的类型
    ///
    /// # 返回
    ///
    /// 交叉类型
    fn handle_intersection_type(&self, types: Vec<Type>) -> Type;

    /// 处理条件类型
    ///
    /// # 参数
    ///
    /// * `check_type` - 检查类型 (T)
    /// * `extends_type` - 扩展类型 (U)
    /// * `true_type` - 条件为真时的类型 (X)
    /// * `false_type` - 条件为假时的类型 (Y)
    ///
    /// # 返回
    ///
    /// 条件类型
    fn handle_conditional_type(&self, check_type: Type, extends_type: Type, true_type: Type, false_type: Type) -> Type;

    /// 处理映射类型
    ///
    /// # 参数
    ///
    /// * `key_var` - 键变量名 (K)
    /// * `source_type` - 源类型 (T)
    /// * `mapped_type` - 映射后的类型
    ///
    /// # 返回
    ///
    /// 映射类型
    fn handle_mapped_type(&self, key_var: String, source_type: Type, mapped_type: Type) -> Type;

    /// 处理索引访问类型
    ///
    /// # 参数
    ///
    /// * `object_type` - 对象类型 (T)
    /// * `index_type` - 索引类型 (K)
    ///
    /// # 返回
    ///
    /// 索引访问类型
    fn handle_index_access_type(&self, object_type: Type, index_type: Type) -> Type;

    /// 处理 KeyOf 类型
    ///
    /// # 参数
    ///
    /// * `target_type` - 目标类型 (T)
    ///
    /// # 返回
    ///
    /// KeyOf 类型
    fn handle_keyof_type(&self, target_type: Type) -> Type;

    /// 处理字面量类型
    ///
    /// # 参数
    ///
    /// * `literal` - 字面量值
    ///
    /// # 返回
    ///
    /// 字面量类型
    fn handle_literal_type(&self, literal: String) -> Type;

    /// 处理 this 类型
    ///
    /// # 参数
    ///
    /// * `this_type` - this 类型
    ///
    /// # 返回
    ///
    /// this 类型
    fn handle_this_type(&self, this_type: Type) -> Type;

    /// 处理省略 this 参数类型
    ///
    /// # 参数
    ///
    /// * `target_type` - 目标类型
    ///
    /// # 返回
    ///
    /// 省略 this 参数后的类型
    fn handle_omit_this_parameter_type(&self, target_type: Type) -> Type;

    /// 处理 this 参数类型
    ///
    /// # 参数
    ///
    /// * `target_type` - 目标类型
    ///
    /// # 返回
    ///
    /// this 参数类型
    fn handle_this_parameter_type(&self, target_type: Type) -> Type;

    /// 处理接口定义语句
    ///
    /// # 参数
    ///
    /// * `stmt_str` - 接口定义语句
    fn handle_interface_stmt(&mut self, stmt_str: &str);

    /// 解析接口成员
    ///
    /// # 参数
    ///
    /// * `members_str` - 接口成员字符串
    ///
    /// # 返回
    ///
    /// 接口成员映射
    fn parse_interface_members(&self, members_str: &str) -> HashMap<String, Type>;

    /// 处理类型别名语句
    ///
    /// # 参数
    ///
    /// * `stmt_str` - 类型别名语句
    fn handle_type_alias_stmt(&mut self, stmt_str: &str);

    /// 解析类型字符串
    ///
    /// # 参数
    ///
    /// * `type_str` - 类型字符串
    ///
    /// # 返回
    ///
    /// 解析后的类型
    fn parse_type(&self, type_str: &str) -> Type;
}
