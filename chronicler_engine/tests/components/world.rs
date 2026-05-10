use std::sync::Arc;

use chronicler_engine::model::map::MapDef;
use chronicler_engine::model::state::GameState;
use chronicler_engine::model::world::WorldCard;
use tower::util::ServiceExt;

use crate::create_test_state;

#[test]
fn test_load_world_includes_room_image_path() {
    // Read the map.json file directly like load_world does
    let map_json = std::fs::read_to_string("data/worlds/test/map.json").unwrap();
    let map: MapDef = serde_json::from_str(&map_json).unwrap();

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
    let map: MapDef = serde_json::from_str(&map_json).unwrap();

    let world_json = std::fs::read_to_string("data/worlds/test/world.json").unwrap();
    let manifest: chronicler_engine::model::world::WorldManifest =
        serde_json::from_str(&world_json).unwrap();
    let world: WorldCard = manifest.clone().into();

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

    // Create app and test the endpoint
    let app = chronicler_engine::create_app_for_testing(state);

    let req = axum::http::Request::builder()
        .uri("/fragment/visual-sidebar")
        .body(axum::body::Body::empty())
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
    let map: MapDef = serde_json::from_str(&map_json).unwrap();

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

    // Verify npcs_in_area starts empty
    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty on initialization"
    );
}

#[test]
fn test_npcs_in_area_can_be_populated() {
    let mut state = create_test_state();

    // Get an NPC from the state
    let npc: chronicler_engine::model::character::NpcCard =
        state.npcs.get("npc_1").cloned().expect("Should have npc_1");

    // Populate npcs_in_area
    state.scene.npcs_in_area.push(npc);

    assert_eq!(
        state.scene.npcs_in_area.len(),
        1,
        "npcs_in_area should have 1 NPC after population"
    );
    assert_eq!(state.scene.npcs_in_area[0].id, "npc_1", "Should be npc_1");
}

#[test]
fn test_npcs_in_area_can_be_cleared() {
    let mut state = create_test_state();

    // Get an NPC and populate npcs_in_area
    let npc = state.npcs.get("npc_1").cloned().expect("Should have npc_1");
    state.scene.npcs_in_area.push(npc);

    assert!(
        !state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be populated"
    );

    // Clear for re-quantification
    state.scene.npcs_in_area.clear();

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after clear"
    );
}

#[test]
fn test_npcs_in_area_can_be_replaced() {
    let mut state = create_test_state();

    // Add one NPC
    let npc1 = state.npcs.get("npc_1").cloned().expect("Should have npc_1");
    state.scene.npcs_in_area.push(npc1);

    assert_eq!(state.scene.npcs_in_area.len(), 1, "Should have 1 NPC");

    // Replace with new list (simulating re-quantification)
    let new_npcs = vec![]; // Empty list simulates no NPCs found
    state.scene.npcs_in_area = new_npcs;

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after replacement"
    );
}
