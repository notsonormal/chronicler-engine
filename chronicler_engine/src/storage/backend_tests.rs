use crate::model::llm_message::LlmMessageBuilder;
use crate::model::message::{Message, Swipe};
use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::model::state::{MessageType, MovementState, SceneState};
use crate::model::state_snapshot::{GameStateSnapshot, NarrativeSnapshot};
use crate::model::trigger::NpcEncounterLog;
use crate::storage::backend::{Operation, Storage, TestOverride};
use crate::storage::db::DbPool;

fn sqlite_storage() -> Storage {
    let pool = DbPool::new(":memory:").unwrap();
    Storage::new_sqlite(pool, 1)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Constructors & game_id
// ═══════════════════════════════════════════════════════════════════════════════

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
fn test_set_game_id() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(42);
    assert_eq!(storage.current_game_id(), 42);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Games (InMemory + SQLite)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_in_memory_create_and_get_game() {
    let storage = Storage::new_in_memory();
    let id = storage.create_game("test", "Game A").unwrap();
    assert!(id > 0);

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.world_name, "test");
    assert_eq!(game.name, "Game A");
}

#[test]
fn test_sqlite_create_and_get_game() {
    let storage = sqlite_storage();
    let id = storage.create_game("test", "Game A").unwrap();
    assert!(id > 0);

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.world_name, "test");
    assert_eq!(game.name, "Game A");
}

#[test]
fn test_get_game_not_found() {
    let storage = Storage::new_in_memory();
    assert!(storage.get_game(9999).unwrap().is_none());
}

#[test]
fn test_list_games_orders_by_updated_at() {
    let storage = Storage::new_in_memory();
    let id_a = storage.create_game("w", "A").unwrap();
    let id_b = storage.create_game("w", "B").unwrap();

    let games = storage.list_games().unwrap();
    assert_eq!(games[0].id, id_b);
    assert_eq!(games[1].id, id_a);
}

#[test]
fn test_delete_game() {
    let storage = Storage::new_in_memory();
    let id = storage.create_game("w", "ToDelete").unwrap();
    storage.delete_game(id).unwrap();
    assert!(storage.get_game(id).unwrap().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Snapshots (InMemory + SQLite)
// ═══════════════════════════════════════════════════════════════════════════════

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

#[test]
fn test_save_and_load_latest_snapshot() {
    let storage = Storage::new_in_memory();
    let snap = dummy_snapshot();
    let id = storage.save_snapshot(&snap).unwrap();

    let loaded = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(loaded.db_id, Some(id));
}

#[test]
fn test_load_snapshot_by_id() {
    let storage = Storage::new_in_memory();
    let id = storage.save_snapshot(&dummy_snapshot()).unwrap();

    let loaded = storage.load_snapshot_by_id(id).unwrap().unwrap();
    assert_eq!(loaded.db_id, Some(id));
}

#[test]
fn test_load_snapshot_by_id_not_found() {
    let storage = Storage::new_in_memory();
    assert!(storage.load_snapshot_by_id(9999).unwrap().is_none());
}

#[test]
fn test_load_latest_returns_most_recent() {
    let storage = Storage::new_in_memory();
    storage.save_snapshot(&dummy_snapshot()).unwrap();
    let id_b = storage.save_snapshot(&dummy_snapshot()).unwrap();

    let latest = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(latest.db_id, Some(id_b));
}

#[test]
fn test_load_latest_no_snapshots() {
    let storage = Storage::new_in_memory();
    assert!(storage.load_latest_snapshot().unwrap().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Messages (InMemory + SQLite)
// ═══════════════════════════════════════════════════════════════════════════════

fn dummy_message(text: &str) -> Message {
    Message::new(
        Some("Player".to_string()),
        text,
        MessageType::Input,
        None,
        None,
    )
}

#[test]
fn test_insert_and_load_messages() {
    let storage = Storage::new_in_memory();
    let msg = dummy_message("hello");
    let id = storage.insert_message(&msg).unwrap();
    assert!(id > 0);

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows.len(), 1);
}

#[test]
fn test_delete_message() {
    let storage = Storage::new_in_memory();
    let id = storage.insert_message(&dummy_message("del")).unwrap();
    storage.delete_message(id).unwrap();
    assert!(storage.load_message_rows().unwrap().is_empty());
}

#[test]
fn test_load_messages_empty() {
    let storage = Storage::new_in_memory();
    assert!(storage.load_message_rows().unwrap().is_empty());
}

#[test]
fn test_soft_delete_restore_purge() {
    let storage = Storage::new_in_memory();
    let id = storage.insert_message(&dummy_message("x")).unwrap();

    storage.soft_delete_message(id).unwrap();
    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty(), "soft-deleted message should be hidden");

    storage.restore_soft_deleted(&[id]).unwrap();
    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows.len(), 1, "restored message should reappear");

    storage.soft_delete_message(id).unwrap();
    storage.purge_soft_deleted(&[id]).unwrap();
    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty(), "purged message should be gone permanently");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Swipes (InMemory + SQLite)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_insert_swipe_and_get_active_index() {
    let storage = Storage::new_in_memory();
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    assert_eq!(storage.get_active_swipe_index(msg_id).unwrap(), 0);

    let swipe = Swipe {
        text: "alt".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(msg_id, &swipe, 1).unwrap();
    assert_eq!(storage.get_active_swipe_index(msg_id).unwrap(), 0);
}

#[test]
fn test_update_active_swipe() {
    let storage = Storage::new_in_memory();
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = Swipe {
        text: "alt".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(msg_id, &swipe, 1).unwrap();
    storage.update_active_swipe(msg_id, 1).unwrap();
    assert_eq!(storage.get_active_swipe_index(msg_id).unwrap(), 1);
}

#[test]
fn test_update_swipe_text() {
    let storage = Storage::new_in_memory();
    let msg_id = storage.insert_message(&dummy_message("orig")).unwrap();

    let swipe = Swipe {
        text: "initial".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(msg_id, &swipe, 0).unwrap();

    storage.update_swipe_text(msg_id, 0, "changed").unwrap();
    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id][0].text, "changed");
}

#[test]
fn test_shift_swipe_indices_sqlite() {
    let storage = sqlite_storage();
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = Swipe {
        text: "s1".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(msg_id, &swipe, 1).unwrap();
    storage.shift_swipe_indices(msg_id, 5).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    // shift_swipe_indices adds offset to all swipes: 1 + 5 = 6
    assert_eq!(swipes[&msg_id].len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Presets (InMemory + SQLite)
// ═══════════════════════════════════════════════════════════════════════════════

fn dummy_preset(id: &str, preset_type: PresetType) -> PromptPreset {
    PromptPreset {
        id: id.to_string(),
        name: id.to_string(),
        preset_type,
        role: None,
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: false,
    }
}

#[test]
fn test_save_and_get_preset() {
    let storage = Storage::new_in_memory();
    let preset = dummy_preset("p1", PresetType::System);
    storage.save_preset(&preset).unwrap();

    let loaded = storage.get_preset("p1").unwrap().unwrap();
    assert_eq!(loaded.id, "p1");
}

#[test]
fn test_list_presets_filters_by_type() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("s1", PresetType::System))
        .unwrap();
    storage
        .save_preset(&dummy_preset("q1", PresetType::Quantifier))
        .unwrap();

    let system = storage.list_presets(PresetType::System).unwrap();
    assert_eq!(system.len(), 1);
    assert_eq!(system[0].id, "s1");
}

#[test]
fn test_delete_preset() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("p1", PresetType::System))
        .unwrap();
    storage.delete_preset("p1").unwrap();
    assert!(storage.get_preset("p1").unwrap().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// LLM Messages (InMemory + SQLite)
// ═══════════════════════════════════════════════════════════════════════════════

fn dummy_llm_message(model: &str) -> crate::model::llm_message::LlmMessage {
    LlmMessageBuilder::new()
        .agent_name("narrator")
        .backend_name("test")
        .model_name(model)
        .system_prompt("sys")
        .user_prompt("user")
        .raw_request_json("req")
        .raw_response_json("res")
        .parsed_response("parsed")
        .error_message(None::<String>)
        .build()
}

#[test]
fn test_save_and_list_llm_messages() {
    let storage = Storage::new_in_memory();
    storage.save_llm_message(&dummy_llm_message("m1")).unwrap();
    storage.save_llm_message(&dummy_llm_message("m2")).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_llm_message_cap_prunes_oldest() {
    let storage = sqlite_storage();
    for i in 0..55 {
        storage
            .save_llm_message(&dummy_llm_message(&format!("model-{i}")))
            .unwrap();
    }

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 50);
    assert_eq!(list[0].model_name, "model-5");
    assert_eq!(list[49].model_name, "model-54");
}

#[test]
fn test_llm_message_list_latest_limit() {
    let storage = sqlite_storage();
    for i in 0..10 {
        storage
            .save_llm_message(&dummy_llm_message(&format!("model-{i}")))
            .unwrap();
    }

    let list = storage.list_latest_llm_messages(3).unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[2].model_name, "model-9");
}

#[test]
fn test_llm_message_empty_list() {
    let storage = Storage::new_in_memory();
    assert!(storage.list_latest_llm_messages(50).unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test backend — failure injection
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_test_backend_injection_blocks_operation() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated save failure"),
    );

    let result = storage.save_snapshot(&dummy_snapshot());
    assert!(result.is_err());
}

#[test]
fn test_test_backend_clear_restores_operation() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated save failure"),
    );
    handle.clear(Operation::SaveSnapshot);

    let result = storage.save_snapshot(&dummy_snapshot());
    assert!(result.is_ok());
}

#[test]
fn test_test_backend_non_overridden_operations_work() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated save failure"),
    );

    // SaveSnapshot is overridden, but InsertMessage is not
    let msg = dummy_message("hello");
    let id = storage.insert_message(&msg).unwrap();
    assert!(id > 0);
}

#[test]
fn test_with_failure_chaining() {
    let storage = Storage::new_in_memory()
        .with_failure(Operation::SaveSnapshot, TestOverride::internal("fail"));

    assert!(storage.save_snapshot(&dummy_snapshot()).is_err());
}

#[test]
fn test_test_backend_config_error() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(Operation::SaveSnapshot, TestOverride::config("bad config"));

    let result = storage.save_snapshot(&dummy_snapshot());
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Game-scoped isolation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_game_id_isolates_snapshots() {
    let storage = Storage::new_in_memory();
    let game_a = storage.create_game("w", "A").unwrap();
    let game_b = storage.create_game("w", "B").unwrap();

    storage.set_game_id(game_a);
    let id_a = storage.save_snapshot(&dummy_snapshot()).unwrap();

    storage.set_game_id(game_b);
    let id_b = storage.save_snapshot(&dummy_snapshot()).unwrap();

    let latest_b = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(latest_b.db_id, Some(id_b));

    storage.set_game_id(game_a);
    let latest_a = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(latest_a.db_id, Some(id_a));

    storage.set_game_id(999);
    assert!(storage.load_latest_snapshot().unwrap().is_none());
}

#[test]
fn test_game_id_isolates_messages() {
    let storage = Storage::new_in_memory();
    let game_a = storage.create_game("w", "A").unwrap();
    let game_b = storage.create_game("w", "B").unwrap();

    storage.set_game_id(game_a);
    storage.insert_message(&dummy_message("msg_a")).unwrap();

    storage.set_game_id(game_b);
    assert!(storage.load_message_rows().unwrap().is_empty());

    storage.set_game_id(game_a);
    assert_eq!(storage.load_message_rows().unwrap().len(), 1);
}
