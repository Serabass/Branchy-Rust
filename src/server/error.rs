//! Error response for API.

use crate::ast::SourceError;
use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
  pub error: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_line: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_column: Option<u32>,
}

pub fn error_response(e: SourceError) -> (StatusCode, Json<ErrorResponse>) {
  let (line, column, end_line, end_column) = e
    .span
    .as_ref()
    .map(|s| {
      (
        Some(s.start_line),
        Some(s.start_column),
        Some(s.end_line),
        Some(s.end_column),
      )
    })
    .unwrap_or((None, None, None, None));
  (
    StatusCode::BAD_REQUEST,
    Json(ErrorResponse {
      error: e.message,
      line,
      column,
      end_line,
      end_column,
    }),
  )
}
