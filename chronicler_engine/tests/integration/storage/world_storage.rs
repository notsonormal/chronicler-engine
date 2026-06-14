use chronicler_engine::model::map::{MapDef, Overworld};
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::storage::Storage;

/// Test deleting a world successfully when no games reference it.
#[test]
fn test_delete_world_success() {
    let storage = Storage::new_in_memory();

    // Create a test world
    let world_card = WorldCard {
        key: "test_world".to_string(),
        name: "Test World".to_string(),
        description: "Test world for deletion".to_string(),
        global_rules: vec![],
        starting_room_id: "start".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
        player_key: "player".to_string(),
    };
    let map = MapDef {
        overworld: Overworld {
            id: "test".to_string(),
            name: "Test".to_string(),
            regions: vec![],
        },
    };

    let _world_id = storage.create_world(&world_card, &map).unwrap();

    // Verify the world exists
    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    assert_eq!(worlds[0].key, "test_world");

    // Delete the world
    storage.delete_world("test_world").unwrap();

    // Verify the world is gone
    let worlds_after = storage.list_worlds().unwrap();
    assert!(worlds_after.is_empty(), "World should be deleted");
}

/// Test that deleting a world with games referencing it fails.
#[test]
fn test_delete_world_blocked_by_games() {
    let storage = Storage::new_in_memory();

    // Create a test world
    let world_card = WorldCard {
        key: "test_world_blocked".to_string(),
        name: "Test World".to_string(),
        description: "Test world with games".to_string(),
        global_rules: vec![],
        starting_room_id: "start".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
        player_key: "player".to_string(),
    };
    let map = MapDef {
        overworld: Overworld {
            id: "test".to_string(),
            name: "Test".to_string(),
            regions: vec![],
        },
    };

    let _world_id = storage.create_world(&world_card, &map).unwrap();

    // Create a game referencing this world
    let _game_id = storage
        .create_game("Test World", "test_world_blocked", "Test Game")
        .unwrap();

    // Try to delete the world - should fail with game_count > 0
    let result = storage.delete_world("test_world_blocked");
    assert!(result.is_err(), "Should not delete world with games");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Referenced by") || err_msg.contains("game"),
        "Error should mention games referencing world: {err_msg}"
    );

    // Verify the world still exists
    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1, "World should still exist");
}
