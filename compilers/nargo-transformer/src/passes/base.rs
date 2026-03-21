use nargo_types::Span;

/// 基础变换 Pass 实现，提供通用的辅助方法
pub struct BaseTransform;

impl BaseTransform {
    /// 确保 Span 传播：如果新节点没有 Span，则尝试从旧节点继承
    pub fn inherit_span(old_span: Span, new_span: &mut Span) {
        if new_span.is_unknown() && !old_span.is_unknown() {
            *new_span = old_span;
        }
    }
}
