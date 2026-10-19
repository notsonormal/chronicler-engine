use crate::adapters::driven::storage::{Storage, TestOverride};

#[test]
fn test_with_failure_single_operation() {
    let storage = Storage::new_in_memory().with_failure("get_game", TestOverride::internal("fail"));

    let result = storage.get_game(0);
    assert!(result.is_err());
}

#[test]
fn test_with_test_failures_shared_handle() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set("get_game", TestOverride::internal("simulated failure"));

    assert!(storage.get_game(0).is_err());

    handle.clear("get_game");
    assert!(storage.get_game(0).is_ok());
}

#[test]
fn test_clear_restores_operation() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set("get_game", TestOverride::internal("simulated failure"));
    handle.clear("get_game");

    let result = storage.get_game(0);
    assert!(result.is_ok());
}

#[test]
fn test_non_overridden_operations_unaffected() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set("get_game", TestOverride::internal("simulated failure"));

    assert!(storage.list_games().unwrap().is_empty());
}

#[test]
fn test_config_error_variant() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set("get_game", TestOverride::config("configuration error"));

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

    handle.set("get_game", TestOverride::internal("internal error"));

    let result = storage.get_game(0);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("internal error"));
}

#[test]
fn test_clear_all_removes_all_overrides() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set("get_game", TestOverride::internal("fail1"));
    handle.set("list_games", TestOverride::config("fail2"));

    handle.clear_all();

    assert!(storage.get_game(0).is_ok());
    assert!(storage.list_games().is_ok());
}

#[test]
#[should_panic(expected = "Unconsumed overrides remain")]
fn test_typoed_override_key_panics_on_assert() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("get_gam", TestOverride::internal("typo deliberate"));
    let _ = storage.get_game(0);
    handle.assert_no_unconsumed();
}

#[test]
fn with_failure_adds_does_not_nest() {
    let s = Storage::new_in_memory()
        .with_failure("get_game", TestOverride::internal("first"))
        .with_failure("get_game", TestOverride::internal("second"));
    let (layer, base) = s.backend_layer_info();
    assert_eq!(
        layer, "Test",
        "expected single Test layer after chained with_failure"
    );
    assert_eq!(base, "InMemory", "with_failure must not stack Test layers");
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
    s.add_failure("get_game", TestOverride::internal("snap fail"));
    s.add_failure("list_games", TestOverride::config("load fail"));
    let (layer, base) = s.backend_layer_info();
    assert_eq!(
        layer, "Test",
        "expected single Test layer after double add_failure"
    );
    assert_eq!(base, "InMemory", "add_failure must not stack Test layers");
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
