use crate::domain::model::character::{CharacterSheet, PlayerCard};
use crate::storage::backend::{Storage, TestOverride};
use crate::test_support::sqlite_storage;

#[test]
fn test_list_personas_in_memory_empty() {
    let storage = Storage::new_in_memory();
    let personas = storage.list_personas().unwrap();
    assert!(personas.is_empty());
}

#[test]
fn test_list_personas_in_memory_seeded() {
    let storage = Storage::new_in_memory();
    let card = test_persona_card("test_persona", "Test Persona");
    storage.seed_persona("test_key", &card).unwrap();

    let personas = storage.list_personas().unwrap();
    assert_eq!(personas.len(), 1);
    assert_eq!(personas[0].sheet.name, "Test Persona");
}

#[test]
fn test_get_persona_found_in_memory() {
    let storage = Storage::new_in_memory();
    let card = test_persona_card("persona1", "Persona One");
    storage.seed_persona("key1", &card).unwrap();

    let result = storage.get_persona("key1").unwrap();
    assert!(result.is_some());
    let retrieved = result.unwrap();
    assert_eq!(retrieved.sheet.name, "Persona One");
}

#[test]
fn test_get_persona_not_found_in_memory() {
    let storage = Storage::new_in_memory();
    let result = storage.get_persona("nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_seed_persona_sqlite() {
    let storage = sqlite_storage().unwrap();
    let card = test_persona_card("sqlite_persona", "SQLite Persona");
    storage.seed_persona("sqlite_key", &card).unwrap();

    let retrieved = storage.get_persona("sqlite_key").unwrap();
    assert!(retrieved.is_some());
    let card = retrieved.unwrap();
    assert_eq!(card.sheet.name, "SQLite Persona");

    let all = storage.list_personas().unwrap();
    assert_eq!(all.len(), 1);
}

#[test]
fn test_seed_persona_idempotent_in_memory() {
    let storage = Storage::new_in_memory();
    let card = test_persona_card("dup_persona", "Duplicate Persona");

    storage.seed_persona("dup_key", &card).unwrap();
    storage.seed_persona("dup_key", &card).unwrap();

    let personas = storage.list_personas().unwrap();
    assert_eq!(personas.len(), 1);
    assert_eq!(personas[0].sheet.name, "Duplicate Persona");
}

#[test]
fn test_list_personas_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("list_personas", TestOverride::internal("list failed"));

    let result = storage.list_personas();
    assert!(result.is_err());
}

#[test]
fn test_get_persona_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("get_persona", TestOverride::config("get failed"));

    let result = storage.get_persona("persona");
    assert!(result.is_err());
}

#[test]
fn test_seed_persona_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("seed_persona", TestOverride::internal("seed failed"));

    let card = test_persona_card("fail_persona", "Fail Persona");
    let result = storage.seed_persona("fail_key", &card);
    assert!(result.is_err());
}

fn test_persona_card(_id: &str, name: &str) -> PlayerCard {
    PlayerCard {
        key: format!("{}_key", name.to_lowercase()),
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
    }
}
