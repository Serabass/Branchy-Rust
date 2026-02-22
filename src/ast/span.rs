//! Source span and error types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Source span for trace (Monaco: 1-based line, 1-based column; end exclusive for column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
  pub start_line: u32,
  pub start_column: u32,
  pub end_line: u32,
  pub end_column: u32,
}

/// Error with optional source location (line, column).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceError {
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub span: Option<Span>,
}

impl SourceError {
  pub fn with_span(mut self, span: Option<Span>) -> Self {
    self.span = span;
    self
  }
}

impl fmt::Display for SourceError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(s) = &self.span {
      write!(
        f,
        "line {}, column {}: {}",
        s.start_line, s.start_column, self.message
      )?;
      if s.start_line != s.end_line || s.start_column != s.end_column {
        write!(f, " (through line {}, column {})", s.end_line, s.end_column)?;
      }
      Ok(())
    } else {
      write!(f, "{}", self.message)
    }
  }
}

impl std::error::Error for SourceError {}

/// Build a Span from source string and byte offsets (0-based). Useful for error reporting.
pub fn span_from_offsets(source: &str, start: usize, end: usize) -> Span {
  let mut line_index = vec![0];
  for (i, c) in source.char_indices() {
    if c == '\n' {
      line_index.push(i + 1);
    }
  }
  let start_line = (0..line_index.len())
    .rposition(|i| line_index[i] <= start)
    .unwrap_or(0);
  let end_line = (0..line_index.len())
    .rposition(|i| line_index[i] <= end.saturating_sub(1))
    .unwrap_or(0);
  Span {
    start_line: (start_line + 1) as u32,
    start_column: (start - line_index[start_line] + 1) as u32,
    end_line: (end_line + 1) as u32,
    end_column: (end - line_index[end_line] + 1) as u32,
  }
}
