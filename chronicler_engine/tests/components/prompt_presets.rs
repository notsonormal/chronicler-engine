use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing;

use crate::{TempSettingsGuard, create_test_state};

#[tokio::test]
async fn test_prompt_presets_panel_renders() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/fragment/prompt-presets")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("System Prompts"));
    assert!(body_str.contains("Quantifier Prompts"));
    assert!(body_str.contains("Add System Prompt Preset"));
    assert!(body_str.contains("Add Quantifier Prompt Preset"));
}

#[tokio::test]
async fn test_add_system_preset() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/prompt-presets")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=My+System+Prompt&prompt_text=You+are+a+test+narrator.&preset_type=system",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("My System Prompt"),
        "Expected preset name in response: {body_str}"
    );
}

#[tokio::test]
async fn test_add_quantifier_preset() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/prompt-presets")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=My+Quantifier+Prompt&prompt_text=Quantify+this+scene.&preset_type=quantifier",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("My Quantifier Prompt"),
        "Expected preset name in response: {body_str}"
    );
}

#[tokio::test]
async fn test_activate_preset_updates_settings() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/prompt-presets")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=Activate+Test&prompt_text=Test+prompt.&preset_type=system",
        ))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);

    let delete_marker = "/delete\"";
    let delete_pos = body_str
        .find(delete_marker)
        .expect("preset delete button in response");
    let url_prefix = "/prompt-presets/";
    let search_area = &body_str[..delete_pos];
    let url_pos = search_area
        .rfind(url_prefix)
        .expect("preset URL before delete");
    let id_start = url_pos + url_prefix.len();
    let preset_id = &body_str[id_start..delete_pos];

    let req = Request::builder()
        .uri(format!("/prompt-presets/{preset_id}/activate"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Active"),
        "Expected Active badge: {body_str}"
    );
}

#[tokio::test]
async fn test_delete_preset() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/prompt-presets")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=Delete+Test&prompt_text=Delete+me.&preset_type=system",
        ))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);

    let delete_marker = "/delete\"";
    let delete_pos = body_str
        .find(delete_marker)
        .expect("preset delete button in response");
    let url_prefix = "/prompt-presets/";
    let search_area = &body_str[..delete_pos];
    let url_pos = search_area
        .rfind(url_prefix)
        .expect("preset URL before delete");
    let id_start = url_pos + url_prefix.len();
    let preset_id = &body_str[id_start..delete_pos];

    let req = Request::builder()
        .uri(format!("/prompt-presets/{preset_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.is_empty(),
        "Expected empty body for delete: {body_str}"
    );
}

#[tokio::test]
async fn test_save_preset_invalid_type() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/prompt-presets")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=Bad+Type&prompt_text=Test.&preset_type=invalid",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Invalid preset type"),
        "Expected error for invalid preset type: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_nonexistent_preset() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/fragment/prompt-presets/does-not-exist/edit")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Preset not found"),
        "Expected error for missing preset: {body_str}"
    );
}

#[tokio::test]
async fn test_delete_nonexistent_preset() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/prompt-presets/does-not-exist/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Preset not found"),
        "Expected error for missing preset: {body_str}"
    );
}

#[tokio::test]
async fn test_update_preset_invalid_type() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    // First create a preset
    let req = Request::builder()
        .uri("/prompt-presets")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=Update+Test&prompt_text=Update+me.&preset_type=system",
        ))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);

    let delete_marker = "/delete\"";
    let delete_pos = body_str
        .find(delete_marker)
        .expect("preset delete button in response");
    let url_prefix = "/prompt-presets/";
    let search_area = &body_str[..delete_pos];
    let url_pos = search_area
        .rfind(url_prefix)
        .expect("preset URL before delete");
    let id_start = url_pos + url_prefix.len();
    let preset_id = &body_str[id_start..delete_pos];

    // Now update with invalid type
    let req = Request::builder()
        .uri(format!("/prompt-presets/{preset_id}"))
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=Updated&prompt_text=Updated.&preset_type=invalid",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Invalid preset type"),
        "Expected error for invalid preset type: {body_str}"
    );
}

#[tokio::test]
async fn test_update_preset_success() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/prompt-presets")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=Update+Test&prompt_text=Original+text.&preset_type=system",
        ))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);

    let delete_marker = "/delete\"";
    let delete_pos = body_str
        .find(delete_marker)
        .expect("preset delete button in response");
    let url_prefix = "/prompt-presets/";
    let search_area = &body_str[..delete_pos];
    let url_pos = search_area
        .rfind(url_prefix)
        .expect("preset URL before delete");
    let id_start = url_pos + url_prefix.len();
    let preset_id = &body_str[id_start..delete_pos];

    let req = Request::builder()
        .uri(format!("/prompt-presets/{preset_id}"))
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=Updated+Name&prompt_text=Updated+text.&preset_type=system",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Updated Name"),
        "Expected updated name in response: {body_str}"
    );
}

#[tokio::test]
async fn test_update_nonexistent_preset() {
    let _guard = TempSettingsGuard::new();
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/prompt-presets/does-not-exist")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "name=Updated&prompt_text=Updated.&preset_type=system",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Preset not found"),
        "Expected error for missing preset: {body_str}"
    );
}
