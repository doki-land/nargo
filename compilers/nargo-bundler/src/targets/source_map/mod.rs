use nargo_types::{Position, Result, Span};
use oak_source_map::{SourceMap as OakSourceMap, SourceMapBuilder as OakSourceMapBuilder};

/// 源码映射
///
/// 包装了 oak-source-map 的 SourceMap 实现
pub type SourceMap = oak_source_map::SourceMap;

/// 源码映射构建器
///
/// 包装了 oak-source-map 的 SourceMapBuilder 实现
pub struct SourceMapBuilder {
    inner: OakSourceMapBuilder,
}

impl SourceMapBuilder {
    /// 创建新的源码映射构建器
    pub fn new() -> Self {
        Self { inner: OakSourceMapBuilder::new() }
    }

    /// 添加映射条目
    pub fn add_mapping(&mut self, generated: Position, original: Position, source_file: Option<String>, _name: Option<String>) {
        if let Some(ref source) = source_file {
            self.inner.add_source(source);
        }
        let source_index = source_file.as_ref().map(|_| 0); // 简化处理，假设只有一个源文件
        self.inner.add_mapping(generated.line as u32, generated.column as u32, source_index, Some(original.line as u32), Some(original.column as u32), None);
    }

    /// 从写入器添加映射
    pub fn add_from_writer(&mut self, mappings: &[(Position, Span)], source_file: Option<String>) {
        if let Some(source) = source_file.as_ref() {
            self.inner.add_source(source);
        }
        for (generated, span) in mappings {
            let source_index = source_file.as_ref().map(|_| 0);
            self.inner.add_mapping(generated.line as u32, generated.column as u32, source_index, Some(span.start.line as u32), Some(span.start.column as u32), None);
        }
    }

    /// 完成构建并返回源码映射
    pub fn finish(self) -> SourceMap {
        self.inner.build()
    }
}

/// 转换 SourceMap 为 JSON 字符串
pub fn source_map_to_json(source_map: &SourceMap) -> Result<String> {
    source_map.to_json().map_err(|e| nargo_types::Error::external_error("source-map".to_string(), e.to_string(), Span::default()))
}
