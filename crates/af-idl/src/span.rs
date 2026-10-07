//! Source spans for AFIDL diagnostics.
//!
//! Spans exist for parser/validator errors and future compiler diagnostics
//! only. They are deliberately *absent* from the canonical TypeSchema model
//! and can never influence a TypeSchemaDigest.

use std::fmt;

/// A half-open byte range plus 1-based line/column of the start position.
/// `column` counts bytes within the line (AFIDL sources are ASCII outside
/// comments, so byte and char columns coincide in practice).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize, line: u32, column: u32) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// A span covering from `first`'s start to `last`'s end.
    pub fn merge(first: SourceSpan, last: SourceSpan) -> Self {
        Self {
            start: first.start,
            end: last.end,
            line: first.line,
            column: first.column,
        }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, column {} (bytes {}..{})",
            self.line, self.column, self.start, self.end
        )
    }
}
