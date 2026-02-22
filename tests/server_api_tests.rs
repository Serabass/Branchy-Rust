use axum::body::Body;
use axum::http::{Request, StatusCode};
use branchy::server::{create_app, AppState};
use branchy::default_registry;
use serde_json::json;
use std::sync::Arc;
use tower::util::ServiceExt;

fn app() -> axum::Router {
    create_app(AppState {
        builtins: Arc::new(default_registry()),
    })
}

#[tokio::test]
async fn health_returns_ok() {
    let app = app();
    let req = Request::builder().uri("/health").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"ok");
}

#[tokio::test]
async fn examples_returns_json_array() {
    let app = app();
    let req = Request::builder().uri("/examples").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(out.is_array(), "expected array, got {:?}", out);
}

#[tokio::test]
async fn run_simple_branch() {
    let app = app();
    let body = json!({ "source": "[ a; b; c; ]" }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let result = out["result"].as_str().unwrap();
    let allowed = ["a", "b", "c"];
    assert!(allowed.contains(&result), "got {}", result);
}

#[tokio::test]
async fn run_literal_number() {
    let app = app();
    let body = json!({ "source": "[ 42; ]" }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(out["result"].as_str().unwrap(), "42");
}

#[tokio::test]
async fn run_inline_call() {
    let app = app();
    let body = json!({ "source": "[ hi <x|y> ]" }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let result = out["result"].as_str().unwrap();
    assert!(result == "hi x" || result == "hi y", "got {}", result);
}

#[tokio::test]
async fn run_template_with_block() {
    let app = app();
    let body = json!({
        "source": "[ greet :who { :who = [ world; ]; }; ]"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(out["result"].as_str().unwrap(), "greet world");
}

#[tokio::test]
async fn run_same_seed_same_result() {
    let app = app();
    let body = json!({ "source": "[ a; b; ]", "seed": 42 }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res1 = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    let out1: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(res1.into_body(), usize::MAX).await.unwrap(),
    )
    .unwrap();
    let body2 = json!({ "source": "[ a; b; ]", "seed": 42 }).to_string();
    let req2 = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body2))
        .unwrap();
    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    let out2: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(res2.into_body(), usize::MAX).await.unwrap(),
    )
    .unwrap();
    assert_eq!(out1["result"], out2["result"], "same seed must give same result");
}

#[tokio::test]
async fn run_user_function() {
    let app = app();
    let body = json!({
        "source": "!f(:x) = [ one :x; ]; [ !f(1); ]"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(out["result"].as_str().unwrap(), "one 1");
}

#[tokio::test]
async fn run_builtin_upper() {
    let app = app();
    let body = json!({ "source": "[ !upper(hello); ]" }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(out["result"].as_str().unwrap(), "HELLO");
}

#[tokio::test]
async fn run_builtin_lower_trim_concat() {
    let app = app();
    let body = json!({ "source": "[ !lower(ABC); ]" }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(out["result"].as_str().unwrap(), "abc");
}

#[tokio::test]
async fn run_invalid_source_returns_400() {
    let app = app();
    let body = json!({ "source": "[ unclosed ; " }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(out["error"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn run_source_with_include_returns_400() {
    let app = app();
    let body = json!({ "source": r#"include "lib.branchy"; [ x; ]"# }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(out["error"].as_str().unwrap().contains("include"));
}

#[tokio::test]
async fn run_not_found_route_returns_404() {
    let app = app();
    let req = Request::builder().uri("/nonexistent").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
