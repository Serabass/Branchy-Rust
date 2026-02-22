use crate::ast::{span_from_offsets, SourceError};

pub(super) fn err_at(
  input: &str,
  start: usize,
  end: usize,
  message: impl Into<String>,
) -> SourceError {
  SourceError {
    message: message.into(),
    span: Some(span_from_offsets(input, start, end)),
  }
}
