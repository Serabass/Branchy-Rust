//! HTTP API server.

mod error;
mod handlers;
mod types;

use axum::{routing::get, routing::post, Router};
use tower_http::cors::CorsLayer;

pub use handlers::{examples, format, health, run};
pub use types::AppState;

pub fn create_app(state: AppState) -> Router {
  Router::new()
    .route("/health", get(health))
    .route("/examples", get(examples))
    .route("/run", post(run))
    .route("/format", post(format))
    .layer(CorsLayer::permissive())
    .with_state(state)
}
