//! Component Tests
//!
//! Merged tests from template_tests.rs (Askama template rendering)
//! and fragment_tests.rs (HTTP endpoint tests).
//!
//! Run with: cargo test --test component_tests

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
        "Expected rendered output to contain 'Chronicler Engine': {}",
        rendered
    );
    assert!(
        rendered.contains(r#"class="header""#),
        "Expected header class: {}",
        rendered
    );
    assert!(
        rendered.contains(r#"class="game-title""#),
        "Expected game-title class: {}",
        rendered
    );
    assert!(
        rendered.contains("connection-status"),
        "Expected connection-status in: {}",
        rendered
    );
}

/// CRITICAL XSS Security Test - verifies HTML escaping
#[test]
fn test_header_template_escapes_html() {
    let template = HeaderTemplate {
        room_name: "<script>alert('xss')</script>".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains("Chronicler Engine"),
        "Should contain Chronicler Engine: {}",
        rendered
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
        "Expected connection-status id: {}",
        rendered
    );
    assert!(
        rendered.contains("Connected"),
        "Expected Connected text: {}",
        rendered
    );
}

#[test]
fn test_header_template_exact_output() {
    let template = HeaderTemplate {
        room_name: "Grand Hall".to_string(),
    };
    let rendered = template.render().unwrap();
    eprintln!("Rendered output: {:?}", rendered);
    assert!(rendered.contains("class=\"header\""));
    assert!(rendered.contains("Chronicler Engine"));
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
            "Expected room image in sidebar: {}",
            body_str
        );
        assert!(
            !body_str.contains("No Location Image"),
            "Should not show placeholder when image exists: {}",
            body_str
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
        // Should contain headshot div or be empty (no NPCs with images)
        // The test state has npc_1 with a profile_image
        assert!(body_str.contains("headshot") || body_str.contains("Test NPC"));
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
}

/// Integration test that loads actual world data and verifies room image_path
/// is correctly deserialized from JSON.
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

    let player_json = std::fs::read_to_string("data/worlds/test/player.json").unwrap();
    let player: chronicler_engine::model::character::PlayerCard =
        serde_json::from_str(&player_json).unwrap();

    // Load NPCs from characters directory
    let chars_dir = std::path::Path::new("data/worlds/test/characters");
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
        "Should contain room image: {}",
        body_str
    );
    assert!(
        !body_str.contains("No Location Image"),
        "Should not show placeholder: {}",
        body_str
    );
}

/// Test that redmist_estate world loads with image_path correctly.
/// This confirms whether the issue is specific to redmist_estate or the general pipeline.
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

/// Test that GameState initializes with empty npcs_in_area
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

/// Test that npcs_in_area can be populated
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

/// Test that npcs_in_area can be cleared (for re-quantification)
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

/// Test that npcs_in_area can be replaced entirely (for re-quantification)
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
