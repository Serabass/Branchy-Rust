//! API handlers: examples, health, run.

use crate::ast::SourceError;
use crate::{interpret, parse_program};
use axum::{extract::State, Json};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::Path;

use super::error::{error_response, ErrorResponse};
use super::types::{ExampleItem, RunRequest, RunResponse};
use super::AppState;
use axum::http::StatusCode;

pub async fn examples() -> Json<Vec<ExampleItem>> {
  let base = std::env::current_dir().unwrap_or_else(|_| Path::new(".").into());
  let examples_dir = base.join("examples");
  let mut out = Vec::new();
  if let Ok(rd) = std::fs::read_dir(&examples_dir) {
    let mut names: Vec<String> = rd
      .filter_map(|e| e.ok())
      .map(|e| e.path())
      .filter(|p| p.extension().map_or(false, |e| e == "branchy"))
      .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
      .collect();
    names.sort();
    for name in names {
      let path = examples_dir.join(format!("{}.branchy", name));
      if let Ok(source) = std::fs::read_to_string(&path) {
        let display = name.replace('_', " ");
        out.push(ExampleItem {
          id: name.clone(),
          name: display,
          source,
        });
      }
    }
  }
  Json(out)
}

pub async fn health() -> &'static str {
  "ok"
}

pub async fn run(
  State(state): State<AppState>,
  Json(body): Json<RunRequest>,
) -> Result<Json<RunResponse>, (StatusCode, Json<ErrorResponse>)> {
  let program = parse_program(&body.source).map_err(error_response)?;
  if !program.includes.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: "includes are not supported in API; send merged source".into(),
        line: None,
        column: None,
        end_line: None,
        end_column: None,
      }),
    ));
  }
  let seed = body.seed.unwrap_or_else(rand::random::<u64>);
  let mut rng = StdRng::seed_from_u64(seed);
  let input = body.input.as_deref();
  let (result, trace) = interpret(&program, &state.builtins, &mut rng, input).map_err(|e: SourceError| {
    let s = e.span.as_ref();
    (
      StatusCode::UNPROCESSABLE_ENTITY,
      Json(ErrorResponse {
        error: e.message,
        line: s.map(|x| x.start_line),
        column: s.map(|x| x.start_column),
        end_line: s.map(|x| x.end_line),
        end_column: s.map(|x| x.end_column),
      }),
    )
  })?;
  Ok(Json(RunResponse { result, trace }))
}
