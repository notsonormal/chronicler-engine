use std::collections::HashMap;

use crate::domain::model::map::{MapDef, Overworld, Region, Room};
use crate::domain::model::world::WorldCard;
use crate::adapters::driven::storage::backend::{Storage, TestOverride};
use crate::test_support::sqlite_storage;

#[test]
fn test_list_worlds_in_memory_empty() {
    let storage = Storage::new_in_memory();
    let worlds = storage.list_worlds().unwrap();
    assert!(worlds.is_empty());
}

#[test]
fn test_list_worlds_in_memory_seeded() {
    let storage = Storage::new_in_memory();
    let (world_card, map) = test_world_data("test_world", "Test World");
    storage.seed_world(&world_card, &map).unwrap();

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    assert_eq!(worlds[0].key, "test_world");
    assert_eq!(worlds[0].name, "Test World");
}

#[test]
fn test_get_world_found_in_memory() {
    let storage = Storage::new_in_memory();
    let (world_card, map) = test_world_data("world1", "World One");
    storage.seed_world(&world_card, &map).unwrap();

    let result = storage.get_world("world1").unwrap();
    assert!(result.is_some());
    let world_with_map = result.unwrap();
    assert_eq!(world_with_map.world_card.key, "world1");
    assert_eq!(world_with_map.world_card.name, "World One");
    assert_eq!(world_with_map.map.overworld.id, "overworld1");
}

#[test]
fn test_get_world_not_found_in_memory() {
    let storage = Storage::new_in_memory();
    let result = storage.get_world("nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_seed_world_sqlite() {
    let storage = sqlite_storage().unwrap();
    let (world_card, map) = test_world_data("sqlite_world", "SQLite World");
    let world_id = storage.seed_world(&world_card, &map).unwrap();

    let retrieved = storage.get_world("sqlite_world").unwrap();
    assert!(retrieved.is_some());
    let world_with_map = retrieved.unwrap();
    assert_eq!(world_with_map.world_card.key, "sqlite_world");
    assert_eq!(world_with_map.world_card.name, "SQLite World");
    assert_eq!(world_id, world_with_map.world_id);

    let all = storage.list_worlds().unwrap();
    assert_eq!(all.len(), 1);
}

#[test]
fn test_seed_world_idempotent_in_memory() {
    let storage = Storage::new_in_memory();
    let (world_card, map) = test_world_data("dup_world", "Duplicate World");

    let world_id1 = storage.seed_world(&world_card, &map).unwrap();
    let world_id2 = storage.seed_world(&world_card, &map).unwrap();

    assert_eq!(world_id1, world_id2);

    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    assert_eq!(worlds[0].key, "dup_world");
}

#[test]
fn test_list_worlds_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("list_worlds", TestOverride::internal("list failed"));

    let result = storage.list_worlds();
    assert!(result.is_err());
}

#[test]
fn test_get_world_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("get_world", TestOverride::config("get failed"));

    let result = storage.get_world("world");
    assert!(result.is_err());
}

#[test]
fn require_world_returns_entity_when_present() {
    let storage = Storage::new_in_memory();
    let (world_card, map) = test_world_data("hit_world", "Hit World");
    storage.seed_world(&world_card, &map).unwrap();
    let via_get = storage.get_world("hit_world").unwrap().unwrap();
    let via_require = storage.require_world("hit_world").unwrap();
    assert_eq!(via_require.world_card.key, via_get.world_card.key);
    assert_eq!(via_require.world_card.name, via_get.world_card.name);
    assert_eq!(via_require.world_id, via_get.world_id);
}

#[test]
fn require_world_returns_canonical_not_found_when_absent() {
    let storage = Storage::new_in_memory();
    let result = storage.require_world("missing");
    match result {
        Err(crate::error::EngineError::WorldNotFound(key)) => assert_eq!(key, "missing"),
        other => panic!("Expected EngineError::WorldNotFound(\"missing\"), got: {other:?}"),
    }
}

#[test]
fn test_seed_world_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("seed_world", TestOverride::internal("seed failed"));

    let (world_card, map) = test_world_data("fail_world", "Fail World");
    let result = storage.seed_world(&world_card, &map);
    assert!(result.is_err());
}

fn test_world_data(id: &str, name: &str) -> (WorldCard, MapDef) {
    let world_card = WorldCard {
        key: id.to_string(),
        name: name.to_string(),
        description: "Test description".to_string(),
        ..Default::default()
    };

    let map = MapDef {
        overworld: Overworld {
            id: "overworld1".to_string(),
            name: "Test Overworld".to_string(),
            regions: vec![Region {
                id: "region1".to_string(),
                name: "Test Region".to_string(),
                rooms: vec![Room {
                    id: "start".to_string(),
                    name: "Start Room".to_string(),
                    description: "Starting room".to_string(),
                    exits: HashMap::new(),
                    items: Vec::new(),
                    image_path: None,
                    navigation_description: None,
                }],
            }],
        },
    };

    (world_card, map)
}
