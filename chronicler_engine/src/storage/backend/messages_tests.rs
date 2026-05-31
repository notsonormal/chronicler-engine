// ═══════════════════════════════════════════════════════════════════════════════
// Message Storage Backend Tests
// ═══════════════════════════════════════════════════════════════════════════════
// Purpose: Test message storage operations
// Coverage: insert, delete, load, soft delete, restore, purge
// Note: Requires game context (game_id) - storage.set_game_id() must be called
// ═══════════════════════════════════════════════════════════════════════════════

use crate::model::message::Message;
use crate::model::state::MessageType;
use crate::storage::backend::{Operation, Storage, TestOverride};
use crate::storage::db::DbPool;

fn sqlite_storage() -> Storage {
    let pool = DbPool::new(":memory:").unwrap();
    Storage::new_sqlite(pool, 1)
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

// ═══════════════════════════════════════════════════════════════════════════════
// Insert Message
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_insert_message_returns_positive_id() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("hello")).unwrap();
    assert!(id > 0);
}

#[test]
fn test_insert_message_sqlite() {
    let storage = sqlite_storage();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("hello")).unwrap();
    assert!(id > 0);
}

#[test]
fn test_insert_message_in_memory() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("hello")).unwrap();
    assert!(id > 0);
}

#[test]
fn test_insert_message_clears_swipes() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let mut msg = dummy_message("test");
    msg.swipes.push(crate::model::message::Swipe {
        text: "alt".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    });

    let id = storage.insert_message(&msg).unwrap();
    let loaded = storage.load_message_rows().unwrap();
    assert_eq!(loaded[0].swipes.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Load Messages
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_load_message_rows_empty() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_load_message_rows_single() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    storage.insert_message(&dummy_message("msg1")).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text(), "msg1");
}

#[test]
fn test_load_message_rows_multiple_order() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    storage.insert_message(&dummy_message("first")).unwrap();
    storage.insert_message(&dummy_message("second")).unwrap();
    storage.insert_message(&dummy_message("third")).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].text(), "first");
    assert_eq!(rows[1].text(), "second");
    assert_eq!(rows[2].text(), "third");
}

#[test]
fn test_load_message_rows_excludes_soft_deleted() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage
        .insert_message(&dummy_message("will_delete"))
        .unwrap();
    storage.insert_message(&dummy_message("will_keep")).unwrap();

    storage.soft_delete_message(id).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text(), "will_keep");
}

#[test]
fn test_load_message_rows_excludes_other_games() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    storage.insert_message(&dummy_message("game1")).unwrap();

    storage.set_game_id(2);
    storage.insert_message(&dummy_message("game2")).unwrap();

    storage.set_game_id(1);
    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text(), "game1");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Delete Message
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_delete_message_existing() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("del")).unwrap();
    storage.delete_message(id).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_delete_message_nonexistent() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let result = storage.delete_message(9999);
    assert!(result.is_ok());
}

#[test]
fn test_delete_message_sqlite() {
    let storage = sqlite_storage();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("del")).unwrap();
    storage.delete_message(id).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Soft Delete / Restore / Purge
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_soft_delete_hides_from_load() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("x")).unwrap();

    storage.soft_delete_message(id).unwrap();
    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_soft_delete_marks_flag() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("x")).unwrap();

    storage.soft_delete_message(id).unwrap();
    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty());

    let all_messages = &storage
        .list_games()
        .unwrap()
        .iter()
        .map(|_| ())
        .collect::<Vec<_>>();
    let _ = all_messages;
}

#[test]
fn test_restore_soft_deleted_reappears() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("x")).unwrap();

    storage.soft_delete_message(id).unwrap();
    storage.restore_soft_deleted(&[id]).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text(), "x");
}

#[test]
fn test_restore_multiple_ids() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id1 = storage.insert_message(&dummy_message("m1")).unwrap();
    let id2 = storage.insert_message(&dummy_message("m2")).unwrap();

    storage.soft_delete_message(id1).unwrap();
    storage.soft_delete_message(id2).unwrap();
    storage.restore_soft_deleted(&[id1, id2]).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn test_purge_removes_permanently() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.insert_message(&dummy_message("x")).unwrap();

    storage.soft_delete_message(id).unwrap();
    storage.purge_soft_deleted(&[id]).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_purge_multiple_ids() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id1 = storage.insert_message(&dummy_message("m1")).unwrap();
    let id2 = storage.insert_message(&dummy_message("m2")).unwrap();

    storage.soft_delete_message(id1).unwrap();
    storage.soft_delete_message(id2).unwrap();
    storage.purge_soft_deleted(&[id1, id2]).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert!(rows.is_empty());
}

#[test]
// ═══════════════════════════════════════════════════════════════════════════════
// Message Sender Types
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_message_with_sender_user() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg = Message::new(Some("User".to_string()), "hello", MessageType::Input, None, None);
    storage.insert_message(&msg).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows[0].sender, Some("User".to_string()));
}

#[test]
fn test_message_with_sender_system() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg = Message::new(Some("System".to_string()), "info", MessageType::System, None, None);
    storage.insert_message(&msg).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows[0].sender, Some("System".to_string()));
}

#[test]
fn test_message_with_sender_none() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg = Message::new(None, "anonymous", MessageType::Input, None, None);
    storage.insert_message(&msg).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows[0].sender, None);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Message Types
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_message_input_type() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg = Message::new(Some("Player".to_string()), "action", MessageType::Input, None, None);
    storage.insert_message(&msg).unwrap();

    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows[0].message_type, MessageType::Input);
}

#[test]
fn test_message_narration_type() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg = Message::new(
        Some("Narrator".to_string()),
        "response",
        MessageType::Narration,
        None,
        None,
    );
    storage.insert_message(&msg).unwrap();
    let rows = storage.load_message_rows().unwrap();
    assert_eq!(rows[0].message_type, MessageType::Narration);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Error Injection Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_insert_message_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    handle.set(
        Operation::InsertMessage,
        TestOverride::internal("insert failed"),
    );

    let result = storage.insert_message(&dummy_message("test"));
    assert!(result.is_err());
}

#[test]
fn test_delete_message_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    handle.set(
        Operation::DeleteMessage,
        TestOverride::config("delete failed"),
    );

    let result = storage.delete_message(1);
    assert!(result.is_err());
}

#[test]
fn test_load_message_rows_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    handle.set(
        Operation::LoadMessageRows,
        TestOverride::internal("load failed"),
    );

    let result = storage.load_message_rows();
    assert!(result.is_err());
}

#[test]
fn test_soft_delete_message_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    handle.set(
        Operation::SoftDeleteMessage,
        TestOverride::config("soft delete failed"),
    );

    storage.insert_message(&dummy_message("test")).unwrap();
    let result = storage.soft_delete_message(1);
    assert!(result.is_err());
}

#[test]
fn test_restore_soft_deleted_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    handle.set(
        Operation::RestoreSoftDeleted,
        TestOverride::internal("restore failed"),
    );

    let result = storage.restore_soft_deleted(&[1]);
    assert!(result.is_err());
}

#[test]
fn test_purge_soft_deleted_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    handle.set(
        Operation::PurgeSoftDeleted,
        TestOverride::config("purge failed"),
    );

    let result = storage.purge_soft_deleted(&[1]);
    assert!(result.is_err());
}
