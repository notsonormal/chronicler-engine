use chronicler_engine::model::map::MapDef;
use chronicler_engine::model::world::WorldCard;
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

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

    // Verify the current room has image_path set before building the app
    let starting_room_id = world.starting_room_id();
    let room = map.overworld.regions[0]
        .rooms
        .iter()
        .find(|r| r.id == starting_room_id)
        .expect("Should find starting room");
    assert!(
        room.image_path.is_some(),
        "Room should have image_path loaded"
    );
    eprintln!("DEBUG: room.image_path = {:?}", room.image_path);

    // Create app and test the endpoint
    let app = TestAppBuilder::new(world, player)
        .map(map)
        .npcs(npcs)
        .build();

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
