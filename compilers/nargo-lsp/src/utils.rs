//! LSP utilities.

use nargo_types::Span;
use oak_core::Range;

/// Check if offset is in span.
pub fn is_in_span(span: Span, offset: usize) -> bool {
    let offset = offset as u32;
    !span.is_unknown() && offset >= span.start.offset && offset <= span.end.offset
}

/// Convert span to range.
pub fn span_to_range(span: Span) -> Range<usize> {
    Range {
        start: span.start.offset as usize,
        end: span.end.offset as usize,
    }
}
