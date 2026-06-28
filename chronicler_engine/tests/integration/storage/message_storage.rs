use chronicler_engine::model::message::{Message, Swipe};
use chronicler_engine::model::state::message_types::MessageType;
use chronicler_engine::storage::Storage;

use crate::fixtures::create_test_storage;

fn create_storage() -> Storage {
    create_test_storage(1)
}

#[test]
fn test_soft_delete_message() {
    let storage = create_storage();
    let msg = Message::new(
        Some("Player".to_string()),
        "to soft delete",
        MessageType::Input,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();

    let before = storage.load_message_rows().unwrap();
    assert_eq!(before.len(), 1);

    storage.soft_delete_message(id).unwrap();

    let after = storage.load_message_rows().unwrap();
    assert!(after.is_empty(), "Soft-deleted message should be hidden");
}

#[test]
fn test_restore_soft_deleted() {
    let storage = create_storage();
    let msg = Message::new(
        Some("Player".to_string()),
        "to restore",
        MessageType::Input,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();
    storage.soft_delete_message(id).unwrap();

    let deleted = storage.load_message_rows().unwrap();
    assert!(deleted.is_empty());

    storage.restore_soft_deleted(&[id]).unwrap();

    let restored = storage.load_message_rows().unwrap();
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].id, id);
}

#[test]
fn test_purge_soft_deleted() {
    let storage = create_storage();
    let msg = Message::new(
        Some("Player".to_string()),
        "to purge",
        MessageType::Input,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();
    storage.soft_delete_message(id).unwrap();

    storage.purge_soft_deleted(&[id]).unwrap();

    let purged = storage.load_message_rows().unwrap();
    assert!(purged.is_empty(), "Purged message should be gone");
}

#[test]
fn test_insert_swipe_and_load() {
    let storage = create_storage();
    let msg = Message::new(
        Some("Player".to_string()),
        "original",
        MessageType::Narration,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();

    let swipe = Swipe {
        text: "swiped text".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(id, &swipe, 0).unwrap();

    let count = storage.count_swipes_for_message(id).unwrap();
    assert_eq!(count, 1);

    let loaded = storage.load_swipes_for_messages(&[id]).unwrap();
    assert_eq!(loaded.get(&id).unwrap().len(), 1);
    assert_eq!(loaded[&id][0].text, "swiped text");
}

#[test]
fn test_update_swipe_text() {
    let storage = create_storage();
    let msg = Message::new(
        Some("Player".to_string()),
        "original",
        MessageType::Narration,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();

    let swipe = Swipe {
        text: "before".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(id, &swipe, 0).unwrap();
    storage.update_swipe_text(id, 0, "after").unwrap();

    let loaded = storage.load_swipes_for_messages(&[id]).unwrap();
    assert_eq!(loaded[&id][0].text, "after");
}

#[test]
fn test_shift_swipe_indices() {
    let storage = create_storage();
    let msg = Message::new(
        Some("Player".to_string()),
        "original",
        MessageType::Narration,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();

    let swipe = Swipe {
        text: "first".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(id, &swipe, 0).unwrap();
    storage.shift_swipe_indices(id, 1).unwrap();

    let loaded = storage.load_swipes_for_messages(&[id]).unwrap();
    // After shifting by 1, the swipe at index 0 moves to index 1
    assert_eq!(loaded[&id][0].text, "first");
    // The load order is by swipe_index, so index should be reflected
}

#[test]
fn test_load_swipes_for_messages_multi() {
    let storage = create_storage();

    let msg_a = Message::new(
        Some("Player".to_string()),
        "msg a",
        MessageType::Input,
        None,
        None,
    );
    let id_a = storage.insert_message(&msg_a).unwrap();

    let msg_b = Message::new(
        Some("NPC".to_string()),
        "msg b",
        MessageType::Narration,
        None,
        None,
    );
    let id_b = storage.insert_message(&msg_b).unwrap();

    storage
        .insert_swipe(
            id_a,
            &Swipe {
                text: "a swipe".to_string(),
                snapshot_id: None,
                location_header: None,
                event_header: None,
            },
            0,
        )
        .unwrap();
    storage
        .insert_swipe(
            id_b,
            &Swipe {
                text: "b swipe".to_string(),
                snapshot_id: None,
                location_header: None,
                event_header: None,
            },
            0,
        )
        .unwrap();

    let loaded = storage.load_swipes_for_messages(&[id_a, id_b]).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[&id_a][0].text, "a swipe");
    assert_eq!(loaded[&id_b][0].text, "b swipe");
}

#[test]
fn test_load_swipes_for_messages_empty() {
    let storage = create_storage();
    let loaded = storage.load_swipes_for_messages(&[]).unwrap();
    assert!(loaded.is_empty());
}

#[test]
fn test_count_swipes_for_message_none() {
    let storage = create_storage();
    let msg = Message::new(
        Some("Player".to_string()),
        "no swipes",
        MessageType::Input,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();

    let count = storage.count_swipes_for_message(id).unwrap();
    assert_eq!(count, 0);
}

// ─── InMemory Backend Tests ───

#[test]
fn test_inmemory_insert_and_load() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let msg = Message::new(
        Some("Player".to_string()),
        "inmem test",
        MessageType::Input,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();
    let loaded = storage.load_message_rows().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, id);
}

#[test]
fn test_inmemory_delete_message() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let msg = Message::new(
        Some("Player".to_string()),
        "to delete",
        MessageType::Input,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();
    storage.delete_message(id).unwrap();
    assert!(storage.load_message_rows().unwrap().is_empty());
}

#[test]
fn test_inmemory_get_active_swipe_not_found() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let result = storage.get_active_swipe_index(999);
    assert!(result.is_err());
}

#[test]
fn test_inmemory_soft_delete_and_restore() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let msg = Message::new(
        Some("Player".to_string()),
        "soft",
        MessageType::Input,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();
    storage.soft_delete_message(id).unwrap();
    assert!(storage.load_message_rows().unwrap().is_empty());

    storage.restore_soft_deleted(&[id]).unwrap();
    assert_eq!(storage.load_message_rows().unwrap().len(), 1);
}

#[test]
fn test_inmemory_purge_deleted() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let msg = Message::new(
        Some("Player".to_string()),
        "purge",
        MessageType::Input,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();
    storage.soft_delete_message(id).unwrap();
    storage.purge_soft_deleted(&[id]).unwrap();
    assert!(storage.load_message_rows().unwrap().is_empty());
}
