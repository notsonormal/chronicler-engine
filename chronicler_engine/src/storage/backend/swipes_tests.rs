use crate::model::message::{Message, Swipe};
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

fn dummy_swipe(text: &str) -> Swipe {
    Swipe {
        text: text.to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    }
}

#[test]
fn test_insert_swipe_initial_index_zero() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = dummy_swipe("alt");
    storage.insert_swipe(msg_id, &swipe, 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 1);
    assert_eq!(swipes[&msg_id][0].text, "alt");
}

#[test]
fn test_insert_swipe_sqlite() {
    let storage = sqlite_storage();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = dummy_swipe("alt");
    storage.insert_swipe(msg_id, &swipe, 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 1);
}

#[test]
fn test_insert_swipe_in_memory() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = dummy_swipe("alt");
    storage.insert_swipe(msg_id, &swipe, 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 1);
}

#[test]
fn test_insert_swipe_increments_index() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage.insert_swipe(msg_id, &dummy_swipe("s0"), 0).unwrap();
    storage.insert_swipe(msg_id, &dummy_swipe("s1"), 1).unwrap();
    storage.insert_swipe(msg_id, &dummy_swipe("s2"), 2).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 3);
}

#[test]
fn test_get_active_swipe_index_default() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let index = storage.get_active_swipe_index(msg_id).unwrap();
    assert_eq!(index, 0);
}

#[test]
fn test_get_active_swipe_index_after_update() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage
        .insert_swipe(msg_id, &dummy_swipe("alt"), 1)
        .unwrap();
    storage.update_active_swipe(msg_id, 1).unwrap();

    let index = storage.get_active_swipe_index(msg_id).unwrap();
    assert_eq!(index, 1);
}

#[test]
fn test_update_active_swipe_sqlite() {
    let storage = sqlite_storage();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage
        .insert_swipe(msg_id, &dummy_swipe("alt"), 1)
        .unwrap();
    storage.update_active_swipe(msg_id, 1).unwrap();

    let index = storage.get_active_swipe_index(msg_id).unwrap();
    assert_eq!(index, 1);
}

#[test]
fn test_update_active_swipe_in_memory() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage
        .insert_swipe(msg_id, &dummy_swipe("alt1"), 1)
        .unwrap();
    storage
        .insert_swipe(msg_id, &dummy_swipe("alt2"), 2)
        .unwrap();
    storage.update_active_swipe(msg_id, 2).unwrap();

    let index = storage.get_active_swipe_index(msg_id).unwrap();
    assert_eq!(index, 2);
}

#[test]
fn test_update_swipe_text_single() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("orig")).unwrap();

    storage
        .insert_swipe(msg_id, &dummy_swipe("initial"), 0)
        .unwrap();
    storage.update_swipe_text(msg_id, 0, "changed").unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id][0].text, "changed");
}

#[test]
fn test_update_swipe_text_sqlite() {
    let storage = sqlite_storage();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("orig")).unwrap();

    storage
        .insert_swipe(msg_id, &dummy_swipe("initial"), 0)
        .unwrap();
    storage.update_swipe_text(msg_id, 0, "changed").unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id][0].text, "changed");
}

#[test]
fn test_load_swipes_for_messages_empty() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let swipes = storage.load_swipes_for_messages(&[]).unwrap();
    assert!(swipes.is_empty());
}

#[test]
fn test_load_swipes_for_messages_single() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();
    storage.insert_swipe(msg_id, &dummy_swipe("s1"), 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 1);
}

#[test]
fn test_load_swipes_for_messages_multiple() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id1 = storage.insert_message(&dummy_message("m1")).unwrap();
    let msg_id2 = storage.insert_message(&dummy_message("m2")).unwrap();

    storage
        .insert_swipe(msg_id1, &dummy_swipe("s1"), 0)
        .unwrap();
    storage
        .insert_swipe(msg_id2, &dummy_swipe("s2"), 0)
        .unwrap();

    let swipes = storage
        .load_swipes_for_messages(&[msg_id1, msg_id2])
        .unwrap();
    assert_eq!(swipes[&msg_id1].len(), 1);
    assert_eq!(swipes[&msg_id2].len(), 1);
}

#[test]
fn test_load_swipes_groups_by_message_id() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage.insert_swipe(msg_id, &dummy_swipe("s0"), 0).unwrap();
    storage.insert_swipe(msg_id, &dummy_swipe("s1"), 1).unwrap();
    storage.insert_swipe(msg_id, &dummy_swipe("s2"), 2).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 3);
}

#[test]
fn test_load_swipes_orders_by_index() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage
        .insert_swipe(msg_id, &dummy_swipe("second"), 1)
        .unwrap();
    storage
        .insert_swipe(msg_id, &dummy_swipe("first"), 0)
        .unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id][0].text, "first");
    assert_eq!(swipes[&msg_id][1].text, "second");
}

#[test]
fn test_shift_swipe_indices_positive_offset() {
    let storage = sqlite_storage();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage.insert_swipe(msg_id, &dummy_swipe("s1"), 1).unwrap();
    storage.insert_swipe(msg_id, &dummy_swipe("s2"), 2).unwrap();

    storage.shift_swipe_indices(msg_id, 5).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 2);
}

#[test]
fn test_shift_swipe_indices_zero_offset() {
    let storage = sqlite_storage();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage.insert_swipe(msg_id, &dummy_swipe("s1"), 1).unwrap();
    storage.shift_swipe_indices(msg_id, 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 1);
}

#[test]
fn test_shift_swipe_indices_sqlite() {
    let storage = sqlite_storage();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    storage.insert_swipe(msg_id, &dummy_swipe("s1"), 1).unwrap();
    storage.shift_swipe_indices(msg_id, 5).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id].len(), 1);
}

#[test]
fn test_swipe_with_text_only() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = Swipe {
        text: "text only".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(msg_id, &swipe, 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id][0].text, "text only");
    assert!(swipes[&msg_id][0].snapshot_id.is_none());
}

#[test]
fn test_swipe_with_snapshot_id() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = Swipe {
        text: "with snapshot".to_string(),
        snapshot_id: Some(42),
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(msg_id, &swipe, 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(swipes[&msg_id][0].snapshot_id, Some(42));
}

#[test]
fn test_swipe_with_location_header() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = Swipe {
        text: "with location".to_string(),
        snapshot_id: None,
        location_header: Some("Room-123".to_string()),
        event_header: None,
    };
    storage.insert_swipe(msg_id, &swipe, 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(
        swipes[&msg_id][0].location_header,
        Some("Room-123".to_string())
    );
}

#[test]
fn test_swipe_with_event_header() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();

    let swipe = Swipe {
        text: "with event".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: Some("CombatStarted".to_string()),
    };
    storage.insert_swipe(msg_id, &swipe, 0).unwrap();

    let swipes = storage.load_swipes_for_messages(&[msg_id]).unwrap();
    assert_eq!(
        swipes[&msg_id][0].event_header,
        Some("CombatStarted".to_string())
    );
}

#[test]
fn test_insert_swipe_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    storage.insert_message(&dummy_message("m")).unwrap();

    handle.set(
        Operation::InsertSwipe,
        TestOverride::config("insert failed"),
    );

    let result = storage.insert_swipe(1, &dummy_swipe("s"), 0);
    assert!(result.is_err());
}

#[test]
fn test_update_swipe_text_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();
    storage.insert_swipe(msg_id, &dummy_swipe("s"), 0).unwrap();

    handle.set(
        Operation::UpdateSwipeText,
        TestOverride::internal("update failed"),
    );

    let result = storage.update_swipe_text(msg_id, 0, "new");
    assert!(result.is_err());
}

#[test]
fn test_shift_swipe_indices_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    let msg_id = storage.insert_message(&dummy_message("m")).unwrap();
    storage.insert_swipe(msg_id, &dummy_swipe("s"), 0).unwrap();

    handle.set(
        Operation::ShiftSwipeIndices,
        TestOverride::config("shift failed"),
    );

    let result = storage.shift_swipe_indices(msg_id, 5);
    assert!(result.is_err());
}

#[test]
fn test_load_swipes_for_messages_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);

    handle.set(
        Operation::LoadSwipesForMessages,
        TestOverride::internal("load failed"),
    );

    let result = storage.load_swipes_for_messages(&[1]);
    assert!(result.is_err());
}

#[test]
fn test_update_active_swipe_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    storage.insert_message(&dummy_message("m")).unwrap();

    handle.set(
        Operation::UpdateActiveSwipe,
        TestOverride::internal("update failed"),
    );

    let result = storage.update_active_swipe(1, 5);
    assert!(result.is_err());
}
