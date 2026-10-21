use crate::adapters::driven::storage::{Storage, StorageCall, TestOverride};

#[test]
fn test_with_failure_single_operation() {
    let storage =
        Storage::new_in_memory().with_failure(StorageCall::GetGame, TestOverride::internal("fail"));

    let result = storage.get_game(0);
    assert!(result.is_err());
}

#[test]
fn test_with_test_failures_shared_handle() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        StorageCall::GetGame,
        TestOverride::internal("simulated failure"),
    );

    assert!(storage.get_game(0).is_err());

    handle.clear(StorageCall::GetGame);
    assert!(storage.get_game(0).is_ok());
}

#[test]
fn test_clear_restores_operation() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        StorageCall::GetGame,
        TestOverride::internal("simulated failure"),
    );
    handle.clear(StorageCall::GetGame);

    let result = storage.get_game(0);
    assert!(result.is_ok());
}

#[test]
fn test_non_overridden_operations_unaffected() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        StorageCall::GetGame,
        TestOverride::internal("simulated failure"),
    );

    assert!(storage.list_games().unwrap().is_empty());
}

#[test]
fn test_config_error_variant() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        StorageCall::GetGame,
        TestOverride::config("configuration error"),
    );

    let result = storage.get_game(0);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("configuration error")
    );
}

#[test]
fn test_internal_error_variant() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        StorageCall::GetGame,
        TestOverride::internal("internal error"),
    );

    let result = storage.get_game(0);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("internal error"));
}

#[test]
fn test_clear_all_removes_all_overrides() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(StorageCall::GetGame, TestOverride::internal("fail1"));
    handle.set(StorageCall::ListGames, TestOverride::config("fail2"));

    handle.clear_all();

    assert!(storage.get_game(0).is_ok());
    assert!(storage.list_games().is_ok());
}

#[test]
fn with_failure_adds_does_not_nest() {
    let s = Storage::new_in_memory()
        .with_failure(StorageCall::GetGame, TestOverride::internal("first"))
        .with_failure(StorageCall::GetGame, TestOverride::internal("second"));
    let (layer, base) = s.backend_layer_info();
    assert_eq!(
        layer, "Test",
        "expected single Test layer after chained with_failure"
    );
    assert_eq!(base, "Sqlite", "with_failure must not stack Test layers");
    let err = s.get_game(0).unwrap_err().to_string();
    assert!(
        err.contains("second"),
        "latest with_failure must win for same method: {err}"
    );
    assert!(
        !err.contains("first"),
        "first override leaked through: {err}"
    );
}

#[test]
fn add_failure_reuses_map_does_not_nest() {
    let s = Storage::new_in_memory();
    s.add_failure(StorageCall::GetGame, TestOverride::internal("snap fail"));
    s.add_failure(StorageCall::ListGames, TestOverride::config("load fail"));
    let (layer, base) = s.backend_layer_info();
    assert_eq!(
        layer, "Test",
        "expected single Test layer after double add_failure"
    );
    assert_eq!(base, "Sqlite", "add_failure must not stack Test layers");
    let snap_err = s.get_game(0).unwrap_err().to_string();
    assert!(
        snap_err.contains("snap fail"),
        "first add_failure must fire: {snap_err}"
    );
    let load_err = s.list_games().unwrap_err().to_string();
    assert!(
        load_err.contains("load fail"),
        "second add_failure must fire: {load_err}"
    );
}
