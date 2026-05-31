use crate::storage::backend::{Operation, Storage, TestOverride};
use crate::storage::db::DbPool;

fn sqlite_storage() -> Storage {
    let pool = DbPool::new(":memory:").unwrap();
    Storage::new_sqlite(pool, 1)
}

#[test]
fn test_new_in_memory_default_game_id() {
    let storage = Storage::new_in_memory();
    assert_eq!(storage.current_game_id(), 1);
}

#[test]
fn test_new_sqlite_game_id() {
    let storage = sqlite_storage();
    assert_eq!(storage.current_game_id(), 1);
}

#[test]
fn test_set_game_id_updates_backend() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(42);
    assert_eq!(storage.current_game_id(), 42);
}

#[test]
fn test_current_game_id_returns_stored_value() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(100);
    assert_eq!(storage.current_game_id(), 100);
}

#[test]
fn test_with_failure_single_operation() {
    let storage = Storage::new_in_memory()
        .with_failure(Operation::SaveSnapshot, TestOverride::internal("fail"));

    let result = storage.save_snapshot(&dummy_snapshot());
    assert!(result.is_err());
}

#[test]
fn test_with_test_failures_shared_handle() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated failure"),
    );

    assert!(storage.save_snapshot(&dummy_snapshot()).is_err());

    handle.clear(Operation::SaveSnapshot);
    assert!(storage.save_snapshot(&dummy_snapshot()).is_ok());
}

#[test]
fn test_clear_restores_operation() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated failure"),
    );
    handle.clear(Operation::SaveSnapshot);

    let result = storage.save_snapshot(&dummy_snapshot());
    assert!(result.is_ok());
}

#[test]
fn test_non_overridden_operations_unaffected() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated failure"),
    );

    let msg = dummy_message("test");
    let id = storage.insert_message(&msg).unwrap();
    assert!(id > 0);
}

#[test]
fn test_config_error_variant() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(
        Operation::SaveSnapshot,
        TestOverride::config("configuration error"),
    );

    let result = storage.save_snapshot(&dummy_snapshot());
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
        Operation::SaveSnapshot,
        TestOverride::internal("internal error"),
    );

    let result = storage.save_snapshot(&dummy_snapshot());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("internal error"));
}

#[test]
fn test_clear_all_removes_all_overrides() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();

    handle.set(Operation::SaveSnapshot, TestOverride::internal("fail1"));
    handle.set(Operation::LoadLatestSnapshot, TestOverride::config("fail2"));

    handle.clear_all();

    assert!(storage.save_snapshot(&dummy_snapshot()).is_ok());
    assert!(storage.load_latest_snapshot().is_ok());
}

use crate::model::message::Message;
use crate::model::state::MessageType;
use crate::model::state_snapshot::{GameStateSnapshot, NarrativeSnapshot};
use crate::model::state::{MovementState, SceneState};
use crate::model::trigger::NpcEncounterLog;

fn dummy_snapshot() -> GameStateSnapshot {
    GameStateSnapshot {
        db_id: None,
        movement: MovementState {
            current_room_id: "start".to_string(),
            dynamic_rooms: std::collections::HashMap::new(),
        },
        narrative: NarrativeSnapshot::default(),
        scene: SceneState::default(),
        npc_encounter_log: NpcEncounterLog::default(),
        created_at: chrono::Utc::now(),
    }
}

fn dummy_message(text: &str) -> Message {
    Message::new(
        Some("Player".to_string()),
        text,
        MessageType::Input,
        None,
        None,
    )
}
