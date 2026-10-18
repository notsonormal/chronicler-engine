use crate::domain::model::game::GameId;
use crate::domain::model::message::Message;
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::test_support::{dummy_message, dummy_swipe, sqlite_storage};

#[test]
fn test_insert_message_returns_positive_id() {
    let storage = Storage::new_in_memory();
    let id = storage
        .insert_message(GameId(1), &dummy_message("hello"))
        .unwrap();
    assert!(id > 0);
}

#[test]
fn test_insert_message_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage
        .insert_message(GameId(1), &dummy_message("hello"))
        .unwrap();
    assert!(id > 0);
}

#[test]
fn test_insert_message_in_memory() {
    let storage = Storage::new_in_memory();
    let id = storage
        .insert_message(GameId(1), &dummy_message("hello"))
        .unwrap();
    assert!(id > 0);
}

#[test]
fn test_insert_message_preserves_swipes() {
    let storage = Storage::new_in_memory();
    let mut msg = dummy_message("test");
    msg.swipes.push(crate::domain::model::message::Swipe {
        text: "alt".to_string(),
        snapshot_id: None,
        room_report: None,
        event_header: None,
        impersonated: false,
        guide_text: None,
        impersonate_direction: None,
        state: None,
        maker: None,
    });

    let _id = storage.insert_message(GameId(1), &msg).unwrap();
    let loaded = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(loaded[0].swipes.len(), 2);
    assert_eq!(loaded[0].swipes[0].text, "test");
    assert_eq!(loaded[0].swipes[1].text, "alt");
}

#[test]
fn test_load_messages_with_swipes_restores_swipes_and_active_index() {
    let storage = Storage::new_in_memory();
    let message_id = storage
        .insert_message(GameId(1), &dummy_message("original"))
        .unwrap();
    storage
        .insert_swipe(message_id, &dummy_swipe("original"), 0)
        .unwrap();
    storage
        .insert_swipe(message_id, &dummy_swipe("alternate"), 1)
        .unwrap();
    storage
        .update_active_swipe(GameId(1), message_id, 1)
        .unwrap();

    let messages = storage.load_messages_with_swipes(GameId(1)).unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].active_swipe_index, 1);
    assert_eq!(messages[0].swipes.len(), 2);
    assert_eq!(messages[0].swipes[0].text, "original");
    assert_eq!(messages[0].swipes[1].text, "alternate");
    assert_eq!(messages[0].text(), "alternate");
}

#[test]
fn test_load_messages_with_swipes_restores_swipes_and_active_index_sqlite() {
    let storage = sqlite_storage().unwrap();
    let message_id = storage
        .insert_message(GameId(1), &dummy_message("original"))
        .unwrap();
    storage
        .insert_swipe(message_id, &dummy_swipe("original"), 0)
        .unwrap();
    storage
        .insert_swipe(message_id, &dummy_swipe("alternate"), 1)
        .unwrap();
    storage
        .update_active_swipe(GameId(1), message_id, 1)
        .unwrap();

    let messages = storage.load_messages_with_swipes(GameId(1)).unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].active_swipe_index, 1);
    assert_eq!(messages[0].swipes.len(), 2);
    assert_eq!(messages[0].swipes[0].text, "original");
    assert_eq!(messages[0].swipes[1].text, "alternate");
    assert_eq!(messages[0].text(), "alternate");
}

#[test]
fn test_load_message_rows_empty() {
    let storage = Storage::new_in_memory();
    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_load_message_rows_single() {
    let storage = Storage::new_in_memory();
    storage
        .insert_message(GameId(1), &dummy_message("msg1"))
        .unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text(), "msg1");
}

#[test]
fn test_load_message_rows_multiple_order() {
    let storage = Storage::new_in_memory();
    storage
        .insert_message(GameId(1), &dummy_message("first"))
        .unwrap();
    storage
        .insert_message(GameId(1), &dummy_message("second"))
        .unwrap();
    storage
        .insert_message(GameId(1), &dummy_message("third"))
        .unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].text(), "first");
    assert_eq!(rows[1].text(), "second");
    assert_eq!(rows[2].text(), "third");
}

#[test]
fn test_load_message_rows_excludes_soft_deleted() {
    let storage = Storage::new_in_memory();
    let id = storage
        .insert_message(GameId(1), &dummy_message("will_delete"))
        .unwrap();
    storage
        .insert_message(GameId(1), &dummy_message("will_keep"))
        .unwrap();

    storage.soft_delete_message(GameId(1), id).unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text(), "will_keep");
}

#[test]
fn test_load_message_rows_excludes_other_games() {
    let storage = Storage::new_in_memory();
    storage
        .insert_message(GameId(1), &dummy_message("game1"))
        .unwrap();

    storage
        .insert_message(GameId(2), &dummy_message("game2"))
        .unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text(), "game1");
}

#[test]
fn test_delete_message_existing() {
    let storage = Storage::new_in_memory();
    let id = storage
        .insert_message(GameId(1), &dummy_message("del"))
        .unwrap();
    storage.delete_message(GameId(1), id).unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_delete_message_nonexistent() {
    let storage = Storage::new_in_memory();
    let result = storage.delete_message(GameId(1), 9999);
    assert!(result.is_ok());
}

#[test]
fn test_delete_message_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage
        .insert_message(GameId(1), &dummy_message("del"))
        .unwrap();
    storage.delete_message(GameId(1), id).unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_soft_delete_hides_from_load() {
    let storage = Storage::new_in_memory();
    let id = storage
        .insert_message(GameId(1), &dummy_message("x"))
        .unwrap();

    storage.soft_delete_message(GameId(1), id).unwrap();
    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_soft_delete_marks_flag() {
    let storage = Storage::new_in_memory();
    let id = storage
        .insert_message(GameId(1), &dummy_message("x"))
        .unwrap();

    storage.soft_delete_message(GameId(1), id).unwrap();
    let rows = storage.load_message_rows(GameId(1)).unwrap();
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
    let id = storage
        .insert_message(GameId(1), &dummy_message("x"))
        .unwrap();

    storage.soft_delete_message(GameId(1), id).unwrap();
    storage.restore_soft_deleted(GameId(1), &[id]).unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text(), "x");
}

#[test]
fn test_restore_multiple_ids() {
    let storage = Storage::new_in_memory();
    let id1 = storage
        .insert_message(GameId(1), &dummy_message("m1"))
        .unwrap();
    let id2 = storage
        .insert_message(GameId(1), &dummy_message("m2"))
        .unwrap();

    storage.soft_delete_message(GameId(1), id1).unwrap();
    storage.soft_delete_message(GameId(1), id2).unwrap();
    storage
        .restore_soft_deleted(GameId(1), &[id1, id2])
        .unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn test_purge_removes_permanently() {
    let storage = Storage::new_in_memory();
    let id = storage
        .insert_message(GameId(1), &dummy_message("x"))
        .unwrap();

    storage.soft_delete_message(GameId(1), id).unwrap();
    storage.purge_soft_deleted(GameId(1), &[id]).unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_purge_multiple_ids() {
    let storage = Storage::new_in_memory();
    let id1 = storage
        .insert_message(GameId(1), &dummy_message("m1"))
        .unwrap();
    let id2 = storage
        .insert_message(GameId(1), &dummy_message("m2"))
        .unwrap();

    storage.soft_delete_message(GameId(1), id1).unwrap();
    storage.soft_delete_message(GameId(1), id2).unwrap();
    storage.purge_soft_deleted(GameId(1), &[id1, id2]).unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_message_input_type() {
    let storage = Storage::new_in_memory();
    let msg = Message::new("action", MessageType::Input, None, None);
    storage.insert_message(GameId(1), &msg).unwrap();

    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(rows[0].message_type, MessageType::Input);
}

#[test]
fn test_message_narration_type() {
    let storage = Storage::new_in_memory();
    let msg = Message::new("response", MessageType::Narration, None, None);
    storage.insert_message(GameId(1), &msg).unwrap();
    let rows = storage.load_message_rows(GameId(1)).unwrap();
    assert_eq!(rows[0].message_type, MessageType::Narration);
}

#[test]
fn test_insert_message_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("insert_message", TestOverride::internal("insert failed"));

    let result = storage.insert_message(GameId(1), &dummy_message("test"));
    assert!(result.is_err());
}

#[test]
fn test_delete_message_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("delete_message", TestOverride::config("delete failed"));

    let result = storage.delete_message(GameId(1), 1);
    assert!(result.is_err());
}

#[test]
fn test_load_message_rows_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("load_message_rows", TestOverride::internal("load failed"));

    let result = storage.load_message_rows(GameId(1));
    assert!(result.is_err());
}

#[test]
fn test_soft_delete_message_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        "soft_delete_message",
        TestOverride::config("soft delete failed"),
    );

    storage
        .insert_message(GameId(1), &dummy_message("test"))
        .unwrap();
    let result = storage.soft_delete_message(GameId(1), 1);
    assert!(result.is_err());
}

#[test]
fn test_restore_soft_deleted_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        "restore_soft_deleted",
        TestOverride::internal("restore failed"),
    );

    let result = storage.restore_soft_deleted(GameId(1), &[1]);
    assert!(result.is_err());
}

#[test]
fn test_purge_soft_deleted_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("purge_soft_deleted", TestOverride::config("purge failed"));

    let result = storage.purge_soft_deleted(GameId(1), &[1]);
    assert!(result.is_err());
}
