use crate::domain::model::game::GameId;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::domain::model::settings::AppSettings;
use crate::test_support::sqlite_storage;

/// The Game row these reads need, written whole into either backend.
fn seed_game(storage: &Storage, world_name: &str, world_key: &str, name: &str) -> u64 {
    crate::test_support::seed_game_row(
        storage,
        &crate::test_support::test_new_game(
            world_name,
            world_key,
            "test_player",
            "Test Player",
            name,
        ),
    )
    .0
}

#[test]
fn test_get_game_found() {
    let storage = Storage::new_in_memory();
    let id = seed_game(&storage, "w", "w", "G");
    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.id, id);
}

#[test]
fn test_get_game_not_found() {
    let storage = Storage::new_in_memory();
    let game = storage.get_game(9999).unwrap();
    assert!(game.is_none());
}

#[test]
fn test_get_game_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = seed_game(&storage, "test", "test", "SQLiteGame");
    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.name, "SQLiteGame");
    assert_eq!(game.world_name, "test");
}

#[test]
fn test_game_timestamps_created_at() {
    let storage = sqlite_storage().unwrap();
    let id = seed_game(&storage, "w", "w", "G");
    let game = storage.get_game(id).unwrap().unwrap();

    let now = chrono::Utc::now();
    let diff = now - game.created_at;
    assert!(diff.num_seconds() < 2);
}

#[test]
fn test_get_game_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("get_game", TestOverride::config("get failed"));

    let result = storage.get_game(1);
    assert!(result.is_err());
}

#[test]
fn require_game_returns_entity_when_present() {
    let storage = Storage::new_in_memory();
    let id = seed_game(&storage, "w", "w", "RequireHit");
    let via_get = storage.get_game(id).unwrap().unwrap();
    let via_require = storage.require_game(id).unwrap();
    assert_eq!(via_require.id, via_get.id);
    assert_eq!(via_require.name, via_get.name);
    assert_eq!(via_require.id, id);
}

#[test]
fn require_game_returns_canonical_not_found_when_absent() {
    let storage = Storage::new_in_memory();
    let result = storage.require_game(999);
    match result {
        Err(crate::error::EngineError::GameNotFound(id)) => assert_eq!(id, 999),
        other => panic!("Expected EngineError::GameNotFound(999), got: {other:?}"),
    }
}

#[test]
fn active_options_preset_id_prefers_game_row() {
    let storage = Storage::new_in_memory();
    let id = seed_game(&storage, "w", "w", "Game A");

    let mut game = storage.require_game(id).unwrap();
    game.active_options_prompt_preset_id = "game_specific_options".to_string();
    storage.update_game_config(&game).unwrap();

    let settings = AppSettings::default();
    assert_eq!(
        storage.active_options_preset_id(GameId(id), &settings),
        "game_specific_options",
        "the per-game preset id must win over the settings fallback"
    );
}

#[test]
fn active_options_preset_id_falls_back_to_settings_without_game_row() {
    let storage = Storage::new_in_memory();
    let settings = AppSettings {
        active_options_prompt_preset_id: "settings_options_fallback".to_string(),
        ..AppSettings::default()
    };

    assert_eq!(
        storage.active_options_preset_id(GameId(7), &settings),
        "settings_options_fallback"
    );
}

#[test]
fn test_game_row_reports_the_derived_display_name() {
    let storage = sqlite_storage().unwrap();
    let id = seed_game(
        &storage,
        "Redmist Estate",
        "redmist_estate",
        "Redmist Estate_2026-09-29_1",
    );
    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.name, "Redmist Estate_2026-09-29_1");
    assert_eq!(game.display_name, "Redmist Estate — 29 Sep 2026 (1)");
}

#[test]
fn test_update_game_display_name_persists_in_memory() {
    let storage = Storage::new_in_memory();
    let id = seed_game(&storage, "w", "w", "Game A");

    storage.update_game_display_name(id, "Renamed").unwrap();

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.display_name, "Renamed");
    assert_eq!(game.name, "Game A", "the stable name is untouched");
}

#[test]
fn test_update_game_display_name_persists_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = seed_game(&storage, "w", "w", "Game A");

    storage.update_game_display_name(id, "Renamed").unwrap();

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.display_name, "Renamed");
    assert_eq!(game.name, "Game A", "the stable name is untouched");
}

#[test]
fn test_update_game_display_name_missing_returns_not_found() {
    let storage = Storage::new_in_memory();
    let result = storage.update_game_display_name(9999, "Nope");
    assert!(matches!(
        result,
        Err(crate::error::EngineError::GameNotFound(9999))
    ));
}

#[test]
fn test_update_game_display_name_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        "update_game_display_name",
        TestOverride::config("rename failed"),
    );

    let result = storage.update_game_display_name(1, "Nope");
    assert!(result.is_err());
}
