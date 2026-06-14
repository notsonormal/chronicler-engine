use crate::model::character::{CharacterSheet, NpcCard};
use crate::model::map::MapDef;
use crate::model::world::WorldCard;
use crate::storage::backend::{Operation, Storage, TestOverride};
use crate::storage::db::DbPool;

fn sqlite_storage() -> Storage {
    let pool = DbPool::new(":memory:").unwrap();
    Storage::new_sqlite(pool, 1)
}

#[test]
fn test_list_characters_in_memory_empty() {
    let storage = Storage::new_in_memory();
    let characters = storage.list_characters(1).unwrap();
    assert!(characters.is_empty());
}

#[test]
fn test_list_characters_in_memory_seeded() {
    let storage = Storage::new_in_memory();
    let card = test_character_card("test_char", "Test Character");
    storage.seed_character(1, &card).unwrap();

    let characters = storage.list_characters(1).unwrap();
    assert_eq!(characters.len(), 1);
    assert_eq!(characters[0].id, "test_char");
    assert_eq!(characters[0].sheet.name, "Test Character");
}

#[test]
fn test_get_character_found_in_memory() {
    let storage = Storage::new_in_memory();
    let card = test_character_card("char1", "Character One");
    storage.seed_character(1, &card).unwrap();

    let result = storage.get_character(1, "char1").unwrap();
    assert!(result.is_some());
    let retrieved = result.unwrap();
    assert_eq!(retrieved.id, "char1");
    assert_eq!(retrieved.sheet.name, "Character One");
}

#[test]
fn test_get_character_not_found_in_memory() {
    let storage = Storage::new_in_memory();
    let result = storage.get_character(1, "nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_seed_character_sqlite() {
    let storage = sqlite_storage();

    let (world_card, map) = test_world_data();
    storage.seed_world(&world_card, &map).unwrap();

    let card = test_character_card("sqlite_char", "SQLite Character");
    storage.seed_character(1, &card).unwrap();

    let retrieved = storage.get_character(1, "sqlite_char").unwrap();
    assert!(retrieved.is_some());
    let card = retrieved.unwrap();
    assert_eq!(card.id, "sqlite_char");
    assert_eq!(card.sheet.name, "SQLite Character");

    let all = storage.list_characters(1).unwrap();
    assert_eq!(all.len(), 1);
}

#[test]
fn test_seed_character_idempotent_in_memory() {
    let storage = Storage::new_in_memory();
    let card = test_character_card("dup_char", "Duplicate Character");

    storage.seed_character(1, &card).unwrap();
    storage.seed_character(1, &card).unwrap();

    let characters = storage.list_characters(1).unwrap();
    assert_eq!(characters.len(), 1);
    assert_eq!(characters[0].id, "dup_char");
}

#[test]
fn test_list_characters_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        Operation::ListCharacters,
        TestOverride::internal("list failed"),
    );

    let result = storage.list_characters(1);
    assert!(result.is_err());
}

#[test]
fn test_get_character_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(Operation::GetCharacter, TestOverride::config("get failed"));

    let result = storage.get_character(1, "char");
    assert!(result.is_err());
}

#[test]
fn test_seed_character_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        Operation::SeedCharacter,
        TestOverride::internal("seed failed"),
    );

    let card = test_character_card("fail_char", "Fail Character");
    let result = storage.seed_character(1, &card);
    assert!(result.is_err());
}

fn test_character_card(id: &str, name: &str) -> NpcCard {
    NpcCard {
        id: id.to_string(),
        sheet: CharacterSheet {
            name: name.to_string(),
            description: "Test description".to_string(),
            personality: "Test personality".to_string(),
            scenario: "Test scenario".to_string(),
            example_dialogue: String::new(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: Vec::new(),
        triggers: Vec::new(),
        relationships: Vec::new(),
    }
}

fn test_world_data() -> (WorldCard, MapDef) {
    use crate::model::map::{MapDef, Overworld, Region, Room};
    use crate::model::world::WorldCard;
    use std::collections::HashMap;

    let world_card = WorldCard {
        key: "test".to_string(),
        name: "Test World".to_string(),
        description: "Test description".to_string(),
        player_key: "player".to_string(),
        starting_room_id: "start".to_string(),
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
