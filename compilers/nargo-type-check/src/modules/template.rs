//! 模板类型检查模块
//!
//! 包含模板相关的类型检查代码

/// 模板类型检查器
pub trait TemplateChecker {
    /// 类型检查模板
    ///
    /// # 参数
    ///
    /// * `template` - 模板
    fn check_template(&mut self, template: &nargo_ir::TemplateIR);

    /// 类型检查模板节点
    ///
    /// # 参数
    ///
    /// * `node` - 模板节点
    fn check_template_node(&mut self, node: &nargo_ir::TemplateNodeIR);

    /// 类型检查元素
    ///
    /// # 参数
    ///
    /// * `element` - 元素
    fn check_element(&mut self, element: &nargo_ir::ElementIR);

    /// 类型检查属性
    ///
    /// # 参数
    ///
    /// * `attr` - 属性
    fn check_attribute(&mut self, attr: &nargo_ir::AttributeIR);

    /// 类型检查 if 节点
    ///
    /// # 参数
    ///
    /// * `if_node` - if 节点
    fn check_if_node(&mut self, if_node: &nargo_ir::IfNodeIR);

    /// 类型检查 for 节点
    ///
    /// # 参数
    ///
    /// * `for_node` - for 节点
    fn check_for_node(&mut self, for_node: &nargo_ir::ForNodeIR);

    /// 类型检查插值
    ///
    /// # 参数
    ///
    /// * `interpolation` - 插值
    fn check_interpolation(&mut self, interpolation: &nargo_ir::ExpressionIR);
}
