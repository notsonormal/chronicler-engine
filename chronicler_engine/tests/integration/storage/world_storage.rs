use chronicler_engine::model::map::{MapDef, Overworld};
use chronicler_engine::model::scenario::StartingScenario;
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::storage::Storage;

use crate::fixtures::create_test_storage;

fn make_test_world(key: &str, name: &str) -> WorldCard {
    WorldCard {
        key: key.to_string(),
        name: name.to_string(),
        description: "Test world for integration testing".to_string(),
        global_rules: vec!["Rule 1".to_string(), "Rule 2".to_string()],
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: Some("/images/test.png".to_string()),
    }
}

fn make_test_map(id: &str, name: &str) -> MapDef {
    MapDef {
        overworld: Overworld {
            id: id.to_string(),
            name: name.to_string(),
            regions: vec![],
        },
    }
}

#[test]
fn test_delete_world_success() {
    let storage = Storage::new_in_memory();

    let world_card = WorldCard {
        key: "test_world".to_string(),
        name: "Test World".to_string(),
        description: "Test world for deletion".to_string(),
        global_rules: vec![],
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };
    let map = MapDef {
        overworld: Overworld {
            id: "test".to_string(),
            name: "Test".to_string(),
            regions: vec![],
        },
    };

    let _world_id = storage.create_world(&world_card, &map).unwrap();

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    assert_eq!(worlds[0].key, "test_world");

    storage.delete_world("test_world").unwrap();

    let worlds_after = storage.list_worlds().unwrap();
    assert!(worlds_after.is_empty(), "World should be deleted");
}

#[test]
fn test_delete_world_blocked_by_games() {
    let storage = Storage::new_in_memory();
    let world_card = make_test_world("test_world_blocked", "Test World");
    let map = make_test_map("test", "Test");

    let _world_id = storage.create_world(&world_card, &map).unwrap();

    let _game_id = storage
        .create_game(
            "Test World",
            "test_world_blocked",
            "test_player",
            "Test Player",
            "Test Game",
        )
        .unwrap();

    let result = storage.delete_world("test_world_blocked");
    assert!(result.is_err(), "Should not delete world with games");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Referenced by") || err_msg.contains("game"),
        "Error should mention games referencing world: {err_msg}"
    );

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1, "World should still exist");
}

#[test]
fn test_create_world_with_full_data() {
    let storage = Storage::new_in_memory();

    let mut world_card = make_test_world("full_data", "Full Data World");
    world_card.scenarios = vec![StartingScenario {
        id: "scenario1".to_string(),
        name: "Test Scenario".to_string(),
        description: "A test scenario".to_string(),
        starting_room_id: "start".to_string(),
        text: "Welcome!".to_string(),
        npcs: vec![],
    }];
    world_card.default_scenario_id = Some("scenario1".to_string());

    let map = make_test_map("full_map", "Full Map");

    let world_id = storage.create_world(&world_card, &map).unwrap();
    assert!(world_id > 0, "Should return positive world ID");

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    let created = &worlds[0];
    assert_eq!(created.key, "full_data");
    assert_eq!(created.name, "Full Data World");
    assert_eq!(created.global_rules.len(), 2);
    assert_eq!(created.scenarios.len(), 1);
    assert_eq!(created.default_scenario_id, Some("scenario1".to_string()));
}

#[test]
fn test_list_worlds_multiple() {
    let storage = Storage::new_in_memory();

    for i in 1..=3 {
        let world_card = make_test_world(&format!("world_{i}"), &format!("World {i}"));
        let map = make_test_map(&format!("map_{i}"), &format!("Map {i}"));
        storage.create_world(&world_card, &map).unwrap();
    }

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 3, "Should list all 3 worlds");

    let keys: Vec<&String> = worlds.iter().map(|w| &w.key).collect();
    assert!(keys.contains(&&"world_1".to_string()));
    assert!(keys.contains(&&"world_2".to_string()));
    assert!(keys.contains(&&"world_3".to_string()));
}

#[test]
fn test_get_world_by_key_found() {
    let storage = Storage::new_in_memory();

    let world_card = make_test_world("lookup_test", "Lookup World");
    let map = make_test_map("lookup_map", "Lookup Map");
    storage.create_world(&world_card, &map).unwrap();

    let result = storage.get_world("lookup_test").unwrap();
    assert!(result.is_some(), "Should find the world");
    let world_with_map = result.unwrap();
    assert_eq!(world_with_map.world_card.key, "lookup_test");
    assert_eq!(world_with_map.world_card.name, "Lookup World");
}

#[test]
fn test_get_world_by_key_not_found() {
    let storage = Storage::new_in_memory();

    let result = storage.get_world("nonexistent").unwrap();
    assert!(result.is_none(), "Should not find nonexistent world");
}

#[test]
fn test_update_world() {
    let storage = Storage::new_in_memory();

    let world_card = make_test_world("update_test", "Original Name");
    let map = make_test_map("update_map", "Original Map");
    let world_id = storage.create_world(&world_card, &map).unwrap();

    let mut updated_card = world_card.clone();
    updated_card.name = "Updated Name".to_string();
    updated_card.description = "Updated description".to_string();
    updated_card.global_rules = vec!["New Rule".to_string()];

    storage.update_world(world_id, &updated_card, &map).unwrap();

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    let updated = &worlds[0];
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.description, "Updated description");
    assert_eq!(updated.global_rules, vec!["New Rule".to_string()]);
}

#[test]
fn test_update_world_nonexistent_noop() {
    let storage = Storage::new_in_memory();

    let world_card = make_test_world("dummy", "Dummy");
    let map = make_test_map("dummy_map", "Dummy Map");

    let result = storage.update_world(999, &world_card, &map);
    assert!(result.is_ok(), "In-memory update should succeed (no-op)");
}

#[test]
fn test_world_with_scenarios() {
    let storage = Storage::new_in_memory();

    let mut world_card = make_test_world("scenario_world", "Scenario World");
    world_card.scenarios = vec![
        StartingScenario {
            id: "intro".to_string(),
            name: "Introduction".to_string(),
            description: "Start your journey".to_string(),
            starting_room_id: "start".to_string(),
            text: "Welcome to the adventure!".to_string(),
            npcs: vec![],
        },
        StartingScenario {
            id: "advanced".to_string(),
            name: "Advanced Mode".to_string(),
            description: "For experienced players".to_string(),
            starting_room_id: "start".to_string(),
            text: "You're already an expert!".to_string(),
            npcs: vec![],
        },
    ];
    world_card.default_scenario_id = Some("intro".to_string());

    let map = make_test_map("scenario_map", "Scenario Map");
    storage.create_world(&world_card, &map).unwrap();

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    let world = &worlds[0];
    assert_eq!(world.scenarios.len(), 2);
    assert_eq!(world.default_scenario_id, Some("intro".to_string()));
    assert_eq!(world.scenarios[0].id, "intro");
    assert_eq!(world.scenarios[1].id, "advanced");
}

#[test]
fn test_world_with_empty_optionals() {
    let storage = Storage::new_in_memory();

    let mut world_card = make_test_world("minimal", "Minimal World");
    world_card.default_room_image = None;
    world_card.default_scenario_id = None;
    world_card.global_rules = vec![];
    world_card.scenarios = vec![];

    let map = make_test_map("minimal_map", "Minimal Map");
    let result = storage.create_world(&world_card, &map);
    assert!(
        result.is_ok(),
        "Should create world with minimal data: {:?}",
        result.err()
    );

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    let world = &worlds[0];
    assert!(world.default_room_image.is_none());
    assert!(world.default_scenario_id.is_none());
    assert!(world.global_rules.is_empty());
}

#[test]
fn test_create_world_duplicate_key_idempotent() {
    let storage = Storage::new_in_memory();

    let world_card = make_test_world("duplicate", "First");
    let map = make_test_map("dup_map", "Duplicate Map");
    let id1 = storage.create_world(&world_card, &map).unwrap();

    let world_card2 = make_test_world("duplicate", "Second");
    let id2 = storage.create_world(&world_card2, &map).unwrap();

    assert_eq!(id1, id2, "Duplicate key should be idempotent");

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1, "Should only have one world");
}

#[test]
fn test_delete_world_no_games() {
    let storage = Storage::new_in_memory();

    let world_card = make_test_world("to_delete", "To Delete");
    let map = make_test_map("delete_map", "Delete Map");
    storage.create_world(&world_card, &map).unwrap();

    let worlds_before = storage.list_worlds().unwrap();
    assert_eq!(worlds_before.len(), 1);

    let result = storage.delete_world("to_delete");
    assert!(
        result.is_ok(),
        "Should delete world with no games: {:?}",
        result.err()
    );

    let worlds_after = storage.list_worlds().unwrap();
    assert!(worlds_after.is_empty(), "World should be deleted");
}

#[test]
fn test_delete_world_nonexistent_idempotent() {
    let storage = Storage::new_in_memory();

    let result = storage.delete_world("nonexistent");
    assert!(result.is_ok(), "Should succeed even if world doesn't exist");
}

// ─── SQLite Backend Tests ───

#[test]
fn test_sqlite_list_worlds() {
    let storage = create_test_storage(1);

    let world_card = make_test_world("sql_list", "SQL List World");
    let map = make_test_map("sql_map", "SQL Map");
    storage.seed_world(&world_card, &map).unwrap();

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    assert_eq!(worlds[0].key, "sql_list");
}

#[test]
fn test_sqlite_get_world_found() {
    let storage = create_test_storage(1);

    let world_card = make_test_world("sql_get", "SQL Get World");
    let map = make_test_map("sql_map", "SQL Map");
    storage.seed_world(&world_card, &map).unwrap();

    let result = storage.get_world("sql_get").unwrap();
    assert!(result.is_some());
    let wwm = result.unwrap();
    assert_eq!(wwm.world_card.key, "sql_get");
    assert_eq!(wwm.world_card.name, "SQL Get World");
}

#[test]
fn test_sqlite_get_world_not_found() {
    let storage = create_test_storage(1);
    let result = storage.get_world("nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_sqlite_seed_world_idempotent() {
    let storage = create_test_storage(1);

    let world_card = make_test_world("sql_idem", "Idempotent");
    let map = make_test_map("sql_map", "SQL Map");
    let _id1 = storage.seed_world(&world_card, &map).unwrap();
    let _id2 = storage.seed_world(&world_card, &map).unwrap();

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(
        worlds.len(),
        1,
        "Seeding same key twice should keep one world"
    );
    assert_eq!(worlds[0].key, "sql_idem");
}

#[test]
fn test_sqlite_get_world_by_id() {
    let storage = create_test_storage(1);

    let world_card = make_test_world("sql_by_id", "By ID");
    let map = make_test_map("sql_map", "SQL Map");
    let world_id = storage.seed_world(&world_card, &map).unwrap();

    let result = storage.get_world_by_id(world_id).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().world_card.key, "sql_by_id");
}

#[test]
fn test_sqlite_delete_world_blocked_by_games() {
    let storage = create_test_storage(1);

    let world_card = make_test_world("sql_blocked", "Blocked");
    let map = make_test_map("sql_map", "SQL Map");
    storage.seed_world(&world_card, &map).unwrap();

    storage
        .create_game(
            "Blocked",
            "sql_blocked",
            "test_player",
            "Test Player",
            "Test Game",
        )
        .unwrap();

    let result = storage.delete_world("sql_blocked");
    assert!(result.is_err(), "Should not delete world with games");

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1, "World should still exist");
}
