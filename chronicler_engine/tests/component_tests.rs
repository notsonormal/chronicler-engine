//! [DOC: docs/reference/testing.md]

use std::sync::{Arc, Mutex};

use askama::Template;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing;
use chronicler_engine::model::character::{CharacterSheet, NpcCard, PlayerCard};
use chronicler_engine::model::map::MapDef;
use chronicler_engine::model::state::GameState;
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::server::templates::HeaderTemplate;

fn create_test_state() -> Arc<Mutex<GameState>> {
    use chronicler_engine::model::map::Room;

    let world = Arc::new(WorldCard {
        name: "Test World".into(),
        description: "A test world".into(),
        global_rules: vec![],
        default_room_image: None,
    });

    let test_room = Room {
        id: "room_1".into(),
        name: "Test Room".into(),
        description: "A test room for component tests.".into(),
        image_path: Some("data/images/test_room.png".into()),
        exits: std::collections::HashMap::new(),
        items: vec![],
        npcs: vec![],
        navigation_description: None,
    };

    let map = Arc::new(MapDef {
        overworld: chronicler_engine::model::map::Overworld {
            id: "test_overworld".into(),
            name: "Test Overworld".into(),
            regions: vec![chronicler_engine::model::map::Region {
                id: "region_1".into(),
                name: "Test Region".into(),
                rooms: vec![test_room],
            }],
        },
    });

    let player = Arc::new(PlayerCard {
        sheet: CharacterSheet {
            name: "Test Player".into(),
            description: "A test player".into(),
            personality: "Brave".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let npcs = vec![NpcCard {
        id: "npc_1".into(),
        sheet: CharacterSheet {
            name: "Test NPC".into(),
            description: "A test NPC".into(),
            personality: "Friendly".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello there!".into(),
            summary: None,
            profile_image: Some("data/images/npc.png".into()),
            headshot_image: Some("data/images/npc_headshot.png".into()),
        },
        inventory: vec![],
        triggers: vec![],
    }];

    let state = GameState::new(world, map, player, npcs, "room_1".to_string());
    Arc::new(Mutex::new(state))
}

// Template Tests (from template_tests.rs)
#[test]
fn test_header_template_renders_room_name() {
    let template = HeaderTemplate {
        room_name: "Test Room".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains("Chronicler Engine"),
        "Expected rendered output to contain 'Chronicler Engine': {rendered}"
    );
    assert!(
        rendered.contains(r#"class="header""#),
        "Expected header class: {rendered}"
    );
    assert!(
        rendered.contains(r#"class="game-title""#),
        "Expected game-title class: {rendered}"
    );
    assert!(
        rendered.contains("connection-status"),
        "Expected connection-status in: {rendered}"
    );
}

#[test]
fn test_header_template_ignores_room_name() {
    let template = HeaderTemplate {
        room_name: "<script>alert('xss')</script>".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains("Chronicler Engine"),
        "Should contain Chronicler Engine: {rendered}"
    );
    assert!(
        !rendered.contains("<script>"),
        "Template should not contain raw script tag: {rendered}"
    );
}

#[test]
fn test_header_template_connection_status() {
    let template = HeaderTemplate {
        room_name: "Any Room".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains(r#"id="connection-status""#),
        "Expected connection-status id: {rendered}"
    );
    assert!(
        rendered.contains("Connected"),
        "Expected Connected text: {rendered}"
    );
}

use std::sync::MutexGuard;
use std::sync::atomic::{AtomicU64, Ordering};

static SETTINGS_TEST_LOCK: Mutex<()> = Mutex::new(());
static SETTINGS_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempSettingsGuard {
    _lock: MutexGuard<'static, ()>,
    temp_path: std::path::PathBuf,
}

impl Default for TempSettingsGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl TempSettingsGuard {
    pub fn new() -> Self {
        let lock = SETTINGS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let counter = SETTINGS_TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_path = std::env::temp_dir().join(format!(
            "chronicler_test_settings_{}_{}.json",
            std::process::id(),
            counter
        ));
        unsafe { std::env::set_var("CHRONICLER_SETTINGS_PATH", &temp_path) };
        Self {
            _lock: lock,
            temp_path,
        }
    }
}

impl Drop for TempSettingsGuard {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("CHRONICLER_SETTINGS_PATH") };
        let _ = std::fs::remove_file(&self.temp_path);
    }
}

// HTTP Endpoint Tests (from fragment_tests.rs)
#[cfg(test)]
mod tests {
    use super::*;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_header_fragment_returns_html() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/fragment/header")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("class=\"header\""));
        assert!(body_str.contains("Chronicler Engine"));
    }

    #[tokio::test]
    async fn test_story_log_fragment_returns_html() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/fragment/story-log")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("id=\"story-log\""));
    }

    #[tokio::test]
    async fn test_visual_sidebar_fragment_returns_html() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/fragment/visual-sidebar")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("id=\"visual-sidebar\""));
    }

    #[tokio::test]
    async fn test_visual_sidebar_renders_room_image() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/fragment/visual-sidebar")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        // Should contain the image, not "No Location Image"
        assert!(
            body_str.contains("data/images/test_room.png"),
            "Expected room image in sidebar: {body_str}"
        );
        assert!(
            !body_str.contains("No Location Image"),
            "Should not show placeholder when image exists: {body_str}"
        );
    }

    #[tokio::test]
    async fn test_action_area_fragment_returns_html() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/fragment/action-area")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("id=\"action-area\""),
            "Expected action-area id: {body_str}"
        );
    }

    #[tokio::test]
    async fn test_action_handler_accepts_command() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/action")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command=look"))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("status"));
    }

    /// CRITICAL Validation Test - verifies empty command handling
    #[tokio::test]
    async fn test_action_handler_empty_command() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/action")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command="))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("Enter a command"),
            "Expected empty command error: {body_str}"
        );
    }

    #[tokio::test]
    async fn test_hints_handler() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/hints")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Look"));
    }

    #[tokio::test]
    async fn test_status_ready_handler() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/status/ready")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Ready"));
    }

    #[tokio::test]
    async fn test_character_headshots_fragment() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/fragment/character-headshots")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        // The test state has npc_1 with a profile_image, so headshots should render
        assert!(
            body_str.contains("headshot"),
            "Expected headshot in fragment: {body_str}"
        );
    }

    #[tokio::test]
    async fn test_generating_status_handler_idle() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/status/generating")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        // Should return "idle" when not generating
        assert!(body_str.contains("idle"));
    }

    #[tokio::test]
    async fn test_generating_status_handler_narrating() {
        let state = create_test_state();
        {
            let mut guard = state.lock().unwrap();
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating;
            guard.generation_state.phase =
                chronicler_engine::model::state::GenerationPhase::Narrating;
        }
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/status/generating")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("narrating"));
    }

    #[tokio::test]
    async fn test_generating_status_handler_quantifying() {
        let state = create_test_state();
        {
            let mut guard = state.lock().unwrap();
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating;
            guard.generation_state.phase =
                chronicler_engine::model::state::GenerationPhase::Quantifying;
        }
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/status/generating")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("quantifying"));
    }

    #[tokio::test]
    async fn test_reset_generating_handler() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        // reset-generating is POST, not GET
        let req = Request::builder()
            .uri("/status/reset-generating")
            .method(http::Method::POST)
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        // Should return "reset" on success
        assert!(body_str.contains("reset"));
    }

    #[tokio::test]
    async fn test_edit_history_handler_not_found() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        // Try to edit a non-existent log entry (ID 9999) - correct path is /history/:id
        let req = Request::builder()
            .uri("/history/9999")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("text=Edited text"))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        // Should return NOT_FOUND for non-existent entry
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_retry_handler_no_input() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        // retry is POST, not GET
        let req = Request::builder()
            .uri("/retry")
            .method(http::Method::POST)
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        // With no input history, should return BAD_REQUEST
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    mod settings_tests {
        use super::*;
        use crate::TempSettingsGuard;

        #[tokio::test]
        async fn test_settings_panel_returns_html() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/settings")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("Connections"),
                "Expected 'Connections' in response: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_settings_panel_has_provider_select() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/settings")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("conn_provider"),
                "Expected conn_provider select element: {body_str}"
            );
            assert!(
                body_str.contains("OpenRouter"),
                "Expected OpenRouter option: {body_str}"
            );
            assert!(
                body_str.contains("DeepSeek"),
                "Expected DeepSeek option: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_settings_panel_has_model_input() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/settings")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("conn_model"),
                "Expected conn_model input: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_save_settings_switch_narrator() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/settings")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "narration_connection_id=openrouter-euryale&quantifier_connection_id=openrouter-gpt-4o-mini",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("saved"),
                "Expected success response: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_save_settings_switch_quantifier() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/settings")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "narration_connection_id=openrouter-gpt-4o-mini&quantifier_connection_id=ollama-gemma-4-26B",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("saved"),
                "Expected success response: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_save_settings_switch_both() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/settings")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "narration_connection_id=openrouter-euryale&quantifier_connection_id=ollama-gemma-4-26B",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("saved"),
                "Expected success response: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_add_connection_openrouter() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/add")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "conn_name=My+OpenRouter&conn_provider=openrouter&conn_model=openai/gpt-4o&conn_api_key=sk-test&conn_base_url=",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("My OpenRouter"),
                "Expected new connection name in response: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_add_connection_deepseek() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/add")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "conn_name=My+DeepSeek&conn_provider=deepseek&conn_model=deepseek-chat&conn_api_key=&conn_base_url=",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("My DeepSeek"),
                "Expected new connection name in response: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_set_narrator() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/openrouter-euryale/set-narrator")
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
                body_str.contains("Narrator"),
                "Expected Narrator badge on euryale: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_set_quantifier() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/ollama-gemma-4-26B/set-quantifier")
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
                body_str.contains("Quantifier"),
                "Expected Quantifier badge on gemma: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_set_narrator_not_found() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/nonexistent/set-narrator")
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
                body_str.contains("Connection not found"),
                "Expected error for nonexistent connection: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_delete_connection() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/ollama-gemma-4-26B/delete")
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
                "Expected empty response (HTMX swap delete): '{body_str}'"
            );
        }

        #[tokio::test]
        async fn test_delete_connection_not_found() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/nonexistent/delete")
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
                body_str.contains("Connection not found"),
                "Expected error for nonexistent connection: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_edit_connection() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/openrouter-gpt-4o-mini/edit")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "conn_name=Updated+Name&conn_provider=openrouter&conn_model=gpt-4o&conn_api_key=new-key&conn_base_url=",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("Updated Name"),
                "Expected updated connection name: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_edit_connection_not_found() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/nonexistent/edit")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "conn_name=Updated+Name&conn_provider=openrouter&conn_model=gpt-4o&conn_api_key=&conn_base_url=",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("Connection not found"),
                "Expected error for nonexistent connection: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_connection_card_fragment() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/connections/openrouter-gpt-4o-mini")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 4096)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("openrouter-gpt-4o-mini"),
                "Expected connection card: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_connection_card_fragment_not_found() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/connections/nonexistent")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("Connection not found"),
                "Expected error for nonexistent connection: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_edit_connection_form() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/connections/openrouter-gpt-4o-mini/edit")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 8192)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("Edit openrouter-gpt-4o-mini"),
                "Expected edit form: {body_str}"
            );
            assert!(
                body_str.contains("conn_name"),
                "Expected conn_name field: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_edit_connection_form_not_found() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/connections/nonexistent/edit")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("Connection not found"),
                "Expected error for nonexistent connection: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_settings_panel_has_single_user_message_checkbox() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/settings")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("single_user_message"),
                "Expected single_user_message checkbox in settings panel: {body_str}"
            );
            assert!(
                body_str.contains("Single User Message"),
                "Expected 'Single User Message' label in settings panel: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_add_connection_with_single_user_message() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/add")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "conn_name=My+Mock&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&single_user_message=true",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("My Mock"),
                "Expected new connection name in response: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_edit_connection_preserves_single_user_message() {
            let _guard = TempSettingsGuard::new();
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/connections/openrouter-gpt-4o-mini/edit")
                .method(http::Method::POST)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(Body::from(
                    "conn_name=Updated+Name&conn_provider=openrouter&conn_model=gpt-4o&conn_api_key=new-key&conn_base_url=&single_user_message=true",
                ))
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("Updated Name"),
                "Expected updated connection name: {body_str}"
            );
        }

        #[tokio::test]
        async fn test_edit_connection_form_has_single_user_message_checkbox() {
            let state = create_test_state();
            let app = create_app_for_testing(state);

            let req = Request::builder()
                .uri("/fragment/connections/openrouter-gpt-4o-mini/edit")
                .body(Body::empty())
                .unwrap();
            let response = app.oneshot(req).await.unwrap();

            assert!(response.status().is_success());
            let body = axum::body::to_bytes(response.into_body(), 8192)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                body_str.contains("single_user_message"),
                "Expected single_user_message checkbox in edit form: {body_str}"
            );
        }
    }
}

#[tokio::test]
async fn test_css_valid() {
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/assets/styles.css")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let css_content = String::from_utf8_lossy(&body);

    assert!(
        css_content.contains(":root"),
        "CSS should contain :root for CSS variables"
    );
    assert!(
        css_content.contains("var(--"),
        "CSS should use CSS custom properties (var())"
    );
    assert!(
        css_content.contains("@media"),
        "CSS should contain @media queries for responsive breakpoints"
    );
}

#[tokio::test]
async fn test_scrollbar_styled() {
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/assets/styles.css")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let css_content = String::from_utf8_lossy(&body);

    assert!(
        css_content.contains("::-webkit-scrollbar"),
        "CSS should contain custom scrollbar styling"
    );
    assert!(
        css_content.contains("scrollbar-width"),
        "CSS should contain Firefox scrollbar-width"
    );
}

#[test]
fn test_load_world_includes_room_image_path() {
    // Read the map.json file directly like load_world does
    let map_json = std::fs::read_to_string("data/worlds/test/map.json").unwrap();
    let map: chronicler_engine::model::map::MapDef = serde_json::from_str(&map_json).unwrap();

    // Find the start room and verify image_path
    let start_room = map.overworld.regions[0]
        .rooms
        .iter()
        .find(|r| r.id == "start")
        .expect("Should have 'start' room");

    assert_eq!(
        start_room.image_path,
        Some("data/images/test_room.jpg".to_string()),
        "Room image_path should be loaded from JSON"
    );
}

#[tokio::test]
async fn test_visual_sidebar_with_real_world_data() {
    // Load real world data directly from JSON files
    let map_json = std::fs::read_to_string("data/worlds/test/map.json").unwrap();
    let map: chronicler_engine::model::map::MapDef = serde_json::from_str(&map_json).unwrap();

    let world_json = std::fs::read_to_string("data/worlds/test/world.json").unwrap();
    let manifest: chronicler_engine::model::world::WorldManifest =
        serde_json::from_str(&world_json).unwrap();
    let world: chronicler_engine::model::world::WorldCard = manifest.clone().into();

    let player_json = std::fs::read_to_string("data/personas/test_player.json").unwrap();
    let player: chronicler_engine::model::character::PlayerCard =
        serde_json::from_str(&player_json).unwrap();

    // Load NPCs from characters directory
    let chars_dir = std::path::Path::new("data/characters/test");
    let mut npcs = Vec::new();
    if chars_dir.is_dir() {
        for entry in std::fs::read_dir(chars_dir).unwrap().flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let char_json = std::fs::read_to_string(&path).unwrap();
                if let Ok(npc) =
                    serde_json::from_str::<chronicler_engine::model::character::NpcCard>(&char_json)
                {
                    npcs.push(npc);
                }
            }
        }
    }

    // Create game state with real data
    let state = GameState::new(
        Arc::new(world),
        Arc::new(map),
        Arc::new(player),
        npcs,
        manifest.starting_room_id.clone(),
    );

    // Verify the current room has image_path set BEFORE wrapping in Mutex
    {
        let state_guard = &state;
        let room = state_guard.map.overworld.regions[0]
            .rooms
            .iter()
            .find(|r| r.id == manifest.starting_room_id)
            .expect("Should find starting room");
        assert!(
            room.image_path.is_some(),
            "Room should have image_path loaded"
        );
        eprintln!("DEBUG: room.image_path = {:?}", room.image_path);
    }

    // Create app and test the endpoint - state needs to be wrapped in Arc<Mutex<...>>
    let state = Arc::new(Mutex::new(state));
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/fragment/visual-sidebar")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    let body = axum::body::to_bytes(response.into_body(), 2048)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);

    // Should show the room image, not the placeholder
    assert!(
        body_str.contains("test_room.jpg"),
        "Should contain room image: {body_str}"
    );
    assert!(
        !body_str.contains("No Location Image"),
        "Should not show placeholder: {body_str}"
    );
}

#[test]
fn test_redmist_estate_room_image_path() {
    let map_json = std::fs::read_to_string("data/worlds/redmist_estate/map.json").unwrap();
    let map: chronicler_engine::model::map::MapDef = serde_json::from_str(&map_json).unwrap();

    let front_gates = map.overworld.regions[0]
        .rooms
        .iter()
        .find(|r| r.id == "front_gates")
        .expect("Should have 'front_gates' room");

    assert_eq!(
        front_gates.image_path,
        Some("data/images/Redmist Estate.png".to_string()),
        "Redmist Estate room image_path should be loaded"
    );
}

#[test]
fn test_npcs_in_area_initialization() {
    let state = create_test_state();
    let state_guard = state.lock().unwrap();

    // Verify npcs_in_area starts empty
    assert!(
        state_guard.npcs_in_area.is_empty(),
        "npcs_in_area should be empty on initialization"
    );
}

#[test]
fn test_npcs_in_area_can_be_populated() {
    let state = create_test_state();
    let mut state_guard = state.lock().unwrap();

    // Get an NPC from the state
    let npc: chronicler_engine::model::character::NpcCard = state_guard
        .npcs
        .get("npc_1")
        .cloned()
        .expect("Should have npc_1");

    // Populate npcs_in_area
    state_guard.npcs_in_area.push(npc);

    assert_eq!(
        state_guard.npcs_in_area.len(),
        1,
        "npcs_in_area should have 1 NPC after population"
    );
    assert_eq!(state_guard.npcs_in_area[0].id, "npc_1", "Should be npc_1");
}

#[test]
fn test_npcs_in_area_can_be_cleared() {
    let state = create_test_state();
    let mut state_guard = state.lock().unwrap();

    // Get an NPC and populate npcs_in_area
    let npc = state_guard
        .npcs
        .get("npc_1")
        .cloned()
        .expect("Should have npc_1");
    state_guard.npcs_in_area.push(npc);

    assert!(
        !state_guard.npcs_in_area.is_empty(),
        "npcs_in_area should be populated"
    );

    // Clear for re-quantification
    state_guard.npcs_in_area.clear();

    assert!(
        state_guard.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after clear"
    );
}

#[test]
fn test_npcs_in_area_can_be_replaced() {
    let state = create_test_state();
    let mut state_guard = state.lock().unwrap();

    // Add one NPC
    let npc1 = state_guard
        .npcs
        .get("npc_1")
        .cloned()
        .expect("Should have npc_1");
    state_guard.npcs_in_area.push(npc1);

    assert_eq!(state_guard.npcs_in_area.len(), 1, "Should have 1 NPC");

    // Replace with new list (simulating re-quantification)
    let new_npcs = vec![]; // Empty list simulates no NPCs found
    state_guard.npcs_in_area = new_npcs;

    assert!(
        state_guard.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after replacement"
    );
}

#[tokio::test]
async fn test_debug_state_endpoint_returns_json() {
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/debug/state")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    assert!(
        json.get("current_room_id").is_some(),
        "Debug state should include current_room_id"
    );
    assert!(
        json.get("npcs_in_area").is_some(),
        "Debug state should include npcs_in_area"
    );
    assert!(
        json.get("generation_status").is_some(),
        "Debug state should include generation_status"
    );
    assert!(
        json.get("generation_phase").is_some(),
        "Debug state should include generation_phase"
    );
    assert!(
        json.get("character_state").is_some(),
        "Debug state should include character_state"
    );
    assert!(
        json.get("narration_history_tail").is_some(),
        "Debug state should include narration_history_tail"
    );
}

mod text_check_tests {
    use super::*;
    use crate::TempSettingsGuard;
    use chronicler_engine::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
    use std::io::Write;

    fn write_text_check_settings(mode: TextCheckMode) {
        let settings = AppSettings {
            text_check: TextCheckSettings {
                mode,
                enable_auto_check: true,
                ignored_words: vec![],
            },
            ..Default::default()
        };
        let path = std::env::var("CHRONICLER_SETTINGS_PATH").unwrap();
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(serde_json::to_string(&settings).unwrap().as_bytes())
            .unwrap();
    }

    #[tokio::test]
    async fn test_action_check_disabled_forwards_to_action() {
        let _guard = TempSettingsGuard::new();
        write_text_check_settings(TextCheckMode::Disabled);

        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/action/check")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command=look"))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("status"),
            "Expected status fragment: {body_str}"
        );
    }

    #[tokio::test]
    async fn test_action_check_empty_command() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/action/check")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command="))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// CRITICAL: /action/confirm must return full action area HTML for outerHTML swap.
    /// Returning only a status span breaks the DOM when hx-swap="outerHTML" targets #action-area.
    #[tokio::test]
    async fn test_action_confirm_returns_full_action_area() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/action/confirm")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command=look"))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("id=\"action-area\""),
            "Expected action-area container: {body_str}"
        );
        assert!(
            body_str.contains(r#"<form id="command-form""#),
            "Expected command form: {body_str}"
        );
        assert!(
            !body_str.starts_with("<span class=\"status"),
            "Must not return bare status span: {body_str}"
        );
    }

    #[tokio::test]
    async fn test_check_text_disabled() {
        let _guard = TempSettingsGuard::new();
        write_text_check_settings(TextCheckMode::Disabled);

        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/check-text")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command=go to the casle"))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("disabled"),
            "Expected disabled message: {body_str}"
        );
    }

    #[tokio::test]
    async fn test_check_text_empty() {
        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/check-text")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command="))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_check_text_finds_issues() {
        let _guard = TempSettingsGuard::new();
        write_text_check_settings(TextCheckMode::Spell);

        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/check-text")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command=go+to+the+casle"))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("text-check-preview"),
            "Expected preview fragment: {body_str}"
        );
        assert!(
            body_str.contains("castle"),
            "Expected corrected text 'castle': {body_str}"
        );
    }

    #[tokio::test]
    async fn test_check_text_no_issues() {
        let _guard = TempSettingsGuard::new();
        write_text_check_settings(TextCheckMode::SpellGrammar);

        let state = create_test_state();
        let app = create_app_for_testing(state);

        let req = Request::builder()
            .uri("/check-text")
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from("command=go+to+the+castle"))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("No issues found"),
            "Expected no-issues message: {body_str}"
        );
    }
}
