//! Direct trait tests for `HttpError` impls (covers `EngineError: HttpError` path
//! not exercised by the adapter `IntoResponse` tests in `adapters/driving/http/error_tests.rs`).

use crate::application::application_service::ApplicationError;
use crate::application::error::{HttpError, HttpStatusCode};
use crate::error::EngineError;

#[test]
fn application_validation_maps_to_bad_request() {
    let err = ApplicationError::validation("missing field");
    assert!(matches!(err.status_code(), HttpStatusCode::BadRequest));
    let body = err.error_body();
    assert!(matches!(body.status, HttpStatusCode::BadRequest));
    assert!(body.body.contains("missing field"));
    assert!(body.body.contains("&amp;") || !body.body.contains('&'));
}

#[test]
fn application_concurrent_generation_maps_to_service_unavailable() {
    let err = ApplicationError::ConcurrentGeneration;
    assert!(matches!(
        err.status_code(),
        HttpStatusCode::ServiceUnavailable
    ));
    let body = err.error_body();
    assert!(matches!(body.status, HttpStatusCode::ServiceUnavailable));
    assert!(body.body.contains("Generation in progress"));
}

#[test]
fn application_shutting_down_maps_to_service_unavailable() {
    let err = ApplicationError::ShuttingDown;
    assert!(matches!(
        err.status_code(),
        HttpStatusCode::ServiceUnavailable
    ));
    let body = err.error_body();
    assert!(matches!(body.status, HttpStatusCode::ServiceUnavailable));
    assert!(body.body.contains("shutting down"));
}

#[test]
fn application_engine_error_maps_to_internal_server_error() {
    let err = ApplicationError::Engine(EngineError::Render("disk failed".into()));
    assert!(matches!(
        err.status_code(),
        HttpStatusCode::InternalServerError
    ));
    let body = err.error_body();
    assert!(matches!(body.status, HttpStatusCode::InternalServerError));
    assert!(body.body.contains("disk failed"));
}

#[test]
fn engine_error_directly_maps_to_internal_server_error() {
    let err = EngineError::Render("render failed".into());
    assert!(matches!(
        err.status_code(),
        HttpStatusCode::InternalServerError
    ));
    let body = err.error_body();
    assert!(matches!(body.status, HttpStatusCode::InternalServerError));
    assert!(body.body.contains("render failed"));
}

#[test]
fn html_escape_covers_all_special_chars() {
    let err = ApplicationError::validation("<script>alert(\"x\")</script>");
    let body = err.error_body();
    assert!(body.body.contains("&lt;script&gt;"));
    assert!(body.body.contains("&quot;"));
    assert!(!body.body.contains("<script>"));
}
