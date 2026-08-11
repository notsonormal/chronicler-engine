//! Tests for `WorldCatalogue`.

use std::sync::Arc;

use crate::application::world_catalogue::WorldCatalogue;
use crate::test_support::fixtures::{TestMap, TestWorld};

fn make_catalogue() -> WorldCatalogue {
    WorldCatalogue::new(Arc::new(
        crate::adapters::driven::storage::Storage::new_in_memory(),
    ))
}

#[test]
fn test_list_worlds_empty() {
    let catalogue = make_catalogue();
    assert!(catalogue.list_worlds().unwrap().is_empty());
}

#[test]
fn test_create_and_get_world_roundtrip() {
    let catalogue = make_catalogue();
    let world = TestWorld::minimal();
    let map = TestMap::single_room("room_1");
    let id = catalogue.create_world(world, map).unwrap();
    assert!(id > 0);

    let worlds = catalogue.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1);
    assert_eq!(worlds[0].key, "test");

    let (got_id, got_card, got_map) = catalogue.get_world("test").unwrap().unwrap();
    assert_eq!(got_id, id);
    assert_eq!(got_card.key, "test");
    assert_eq!(got_map.overworld.regions.len(), 1);
}

#[test]
fn test_get_world_missing() {
    let catalogue = make_catalogue();
    assert!(catalogue.get_world("missing").unwrap().is_none());
}

#[test]
fn test_update_world() {
    let catalogue = make_catalogue();
    let world = TestWorld::minimal();
    let map = TestMap::single_room("room_1");
    let id = catalogue.create_world(world, map).unwrap();

    let mut updated = TestWorld::minimal();
    updated.name = "Updated".to_string();
    let map = TestMap::single_room("room_1");
    catalogue.update_world(id, updated, map).unwrap();

    let (_, card, _) = catalogue.get_world("test").unwrap().unwrap();
    assert_eq!(card.name, "Updated");
}

#[test]
fn test_delete_world() {
    let catalogue = make_catalogue();
    let world = TestWorld::minimal();
    let map = TestMap::single_room("room_1");
    catalogue.create_world(world, map).unwrap();

    catalogue.delete_world("test").unwrap();
    assert!(catalogue.get_world("test").unwrap().is_none());
    assert!(catalogue.list_worlds().unwrap().is_empty());
}
