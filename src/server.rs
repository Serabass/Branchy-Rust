use crate::ast::SourceError;
use crate::{interpret, parse_program};
use axum::{
  extract::State,
  http::StatusCode,
  routing::{get, post},
  Json, Router,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
  pub builtins: Arc<std::collections::HashMap<String, crate::builtins::BuiltinFn>>,
}

#[derive(Deserialize)]
pub struct RunRequest {
  pub source: String,
  #[serde(default)]
  pub input: Option<String>,
  /// If set, run is deterministic (same result every time). Must be a number.
  #[serde(default, deserialize_with = "deserialize_seed")]
  pub seed: Option<u64>,
}

fn deserialize_seed<'de, D>(d: D) -> Result<Option<u64>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  use serde::de::{Error, Visitor};
  use std::fmt;
  struct SeedVisitor;
  impl<'de> Visitor<'de> for SeedVisitor {
    type Value = Option<u64>;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "a number or null")
    }
    fn visit_none<E>(self) -> Result<Self::Value, E> {
      Ok(None)
    }
    fn visit_some<A>(self, value: A) -> Result<Self::Value, A::Error>
    where
      A: serde::Deserializer<'de>,
    {
      let n = u64::deserialize(value).map_err(|_| A::Error::custom("seed must be a number"))?;
      Ok(Some(n))
    }
  }
  d.deserialize_option(SeedVisitor)
}

#[derive(Serialize)]
pub struct RunResponse {
  pub result: String,
  /// Source spans that were used in this run (for highlighting in the editor).
  pub trace: Vec<crate::ast::Span>,
}

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

fn error_response(e: SourceError) -> (StatusCode, Json<ErrorResponse>) {
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

#[derive(Serialize)]
pub struct ExampleItem {
  pub id: String,
  pub name: String,
  pub source: String,
}

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
  let (result, trace) = interpret(&program, &state.builtins, &mut rng, input).map_err(|e| {
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

pub fn create_app(state: AppState) -> Router {
  Router::new()
    .route("/health", get(health))
    .route("/examples", get(examples))
    .route("/run", post(run))
    .layer(CorsLayer::permissive())
    .with_state(state)
}
