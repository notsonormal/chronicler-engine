use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::domain::model::settings::AppSettings;
use crate::test_support::sqlite_storage;

#[test]
fn test_create_game_returns_positive_id() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("test", "test", "test_player", "Test Player", "Game A")
        .unwrap();
    assert!(id > 0);
}

#[test]
fn test_create_game_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage
        .create_game("test", "test", "test_player", "Test Player", "Game A")
        .unwrap();
    assert!(id > 0);

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.world_name, "test");
    assert_eq!(game.name, "Game A");
}

#[test]
fn test_create_game_in_memory() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("test", "test", "test_player", "Test Player", "Game A")
        .unwrap();
    assert!(id > 0);

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.world_name, "test");
    assert_eq!(game.name, "Game A");
}

#[test]
fn test_get_game_found() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "G")
        .unwrap();
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
    let id = storage
        .create_game("test", "test", "test_player", "Test Player", "SQLiteGame")
        .unwrap();
    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.name, "SQLiteGame");
}

#[test]
fn test_delete_game_existing() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "ToDelete")
        .unwrap();
    storage.delete_game(id).unwrap();

    let game = storage.get_game(id).unwrap();
    assert!(game.is_none());
}

#[test]
fn test_delete_game_nonexistent() {
    let storage = Storage::new_in_memory();
    let result = storage.delete_game(9999);
    assert!(result.is_ok());
}

#[test]
fn test_delete_game_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "ToDelete")
        .unwrap();
    storage.delete_game(id).unwrap();

    let game = storage.get_game(id).unwrap();
    assert!(game.is_none());
}

#[test]
fn test_game_timestamps_created_at() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "G")
        .unwrap();
    let game = storage.get_game(id).unwrap().unwrap();

    let now = chrono::Utc::now();
    let diff = now - game.created_at;
    assert!(diff.num_seconds() < 2);
}

#[test]
fn test_create_game_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("create_game", TestOverride::internal("create failed"));

    let result = storage.create_game("w", "w", "test_player", "Test Player", "G");
    assert!(result.is_err());
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
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "RequireHit")
        .unwrap();
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
fn test_delete_game_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("delete_game", TestOverride::config("delete failed"));

    let result = storage.delete_game(1);
    assert!(result.is_err());
}

#[test]
fn active_options_preset_id_prefers_current_game() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "Game A")
        .unwrap();
    storage.set_game_id(id);

    let mut game = storage.require_game(id).unwrap();
    game.active_options_prompt_preset_id = "game_specific_options".to_string();
    storage.update_game_config(&game).unwrap();

    let settings = AppSettings::default();
    assert_eq!(
        storage.active_options_preset_id(&settings),
        "game_specific_options",
        "the per-game preset id must win over the settings fallback"
    );
}

#[test]
fn active_options_preset_id_falls_back_to_settings_without_current_game() {
    // game_id starts at 0 and no game row exists — get_game(0) returns None,
    // so the accessor must serve the settings value.
    let storage = Storage::new_in_memory();
    let settings = AppSettings {
        active_options_prompt_preset_id: "settings_options_fallback".to_string(),
        ..AppSettings::default()
    };

    assert_eq!(
        storage.active_options_preset_id(&settings),
        "settings_options_fallback"
    );
}

#[test]
fn test_create_game_persists_a_readable_display_name() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game(
            "Redmist Estate",
            "redmist_estate",
            "p",
            "P",
            "Redmist Estate_2026-09-29_1",
        )
        .unwrap();
    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.name, "Redmist Estate_2026-09-29_1");
    assert_eq!(game.display_name, "Redmist Estate — 29 Sep 2026 (1)");
}

#[test]
fn test_create_game_persists_a_readable_display_name_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage
        .create_game(
            "Redmist Estate",
            "redmist_estate",
            "p",
            "P",
            "Redmist Estate_2026-09-29_1",
        )
        .unwrap();
    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.name, "Redmist Estate_2026-09-29_1");
    assert_eq!(game.display_name, "Redmist Estate — 29 Sep 2026 (1)");
}

#[test]
fn test_update_game_display_name_persists_in_memory() {
    let storage = Storage::new_in_memory();
    let id = storage.create_game("w", "w", "p", "P", "Game A").unwrap();

    storage.update_game_display_name(id, "Renamed").unwrap();

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.display_name, "Renamed");
    assert_eq!(game.name, "Game A", "the stable name is untouched");
}

#[test]
fn test_update_game_display_name_persists_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage.create_game("w", "w", "p", "P", "Game A").unwrap();

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
