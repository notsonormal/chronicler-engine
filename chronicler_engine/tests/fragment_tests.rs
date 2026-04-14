//! Fragment Handler Integration Tests
//!
//! Same-process tests for HTMX fragment endpoints.
//! Uses create_app_for_testing to run tests in-process without spawning a server.
//!
//! Run with: cargo test --test fragment_tests

use std::sync::{Arc, Mutex};

use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing;
use chronicler_engine::model::character::{CharacterSheet, NpcCard, PlayerCard};
use chronicler_engine::model::map::MapDef;
use chronicler_engine::model::state::GameState;
use chronicler_engine::model::world::WorldCard;

fn create_test_state() -> Arc<Mutex<GameState>> {
    use chronicler_engine::model::map::Room;

    let world = Arc::new(WorldCard {
        name: "Test World".into(),
        description: "A test world".into(),
        global_rules: vec![],
    });

    let test_room = Room {
        id: "room_1".into(),
        name: "Test Room".into(),
        description: "A test room for fragment tests.".into(),
        image_path: None,
        exits: std::collections::HashMap::new(),
        items: vec![],
        npcs: vec![],
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
            image_path: None,
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
            image_path: None,
        },
        inventory: vec![],
    }];

    let state = GameState::new(world, map, player, npcs, "room_1".to_string());
    Arc::new(Mutex::new(state))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(body_str.contains("id=\"action-area\"") || body_str.contains("cmd-area"));
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
        // Should return status HTML (ready or thinking)
        assert!(body_str.contains("status"));
    }

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
        // Should return error status
        assert!(body_str.contains("error") || body_str.is_empty());
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
}
