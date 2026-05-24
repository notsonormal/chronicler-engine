use crate::model::message::Message;
use crate::model::state::LogType;
use crate::storage::db::DbPool;
use crate::storage::message_storage::{MessageStorage, SqliteMessageRepository};

fn create_repo() -> SqliteMessageRepository {
    let pool = DbPool::new(":memory:").unwrap();
    SqliteMessageRepository::new(pool, 1)
}

#[test]
fn test_new_sets_game_id() {
    let repo = create_repo();
    assert_eq!(repo.current_game_id(), 1);
}

#[test]
fn test_set_game_id() {
    let repo = create_repo();
    repo.set_game_id(42);
    assert_eq!(repo.current_game_id(), 42);
}

#[test]
fn test_insert_message_returns_id() {
    let repo = create_repo();
    let msg = Message::new(
        Some("Player".to_string()),
        "hello",
        LogType::Input,
        None,
        None,
    );
    let id = repo.insert_message(&msg).unwrap();
    assert!(id > 0, "insert_message should return a positive id");
}

#[test]
fn test_insert_and_load_roundtrip() {
    let repo = create_repo();
    let msg = Message::new(
        Some("Player".to_string()),
        "hello",
        LogType::Input,
        None,
        None,
    );
    let id = repo.insert_message(&msg).unwrap();

    let loaded = repo.load_messages().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, id);
    assert_eq!(loaded[0].text, "hello");
}

#[test]
fn test_update_message() {
    let repo = create_repo();
    let msg = Message::new(
        Some("Player".to_string()),
        "original",
        LogType::Input,
        None,
        None,
    );
    let id = repo.insert_message(&msg).unwrap();

    repo.update_message(id, "updated").unwrap();

    let loaded = repo.load_messages().unwrap();
    assert_eq!(loaded[0].text, "updated");
}

#[test]
fn test_delete_message() {
    let repo = create_repo();
    let msg = Message::new(
        Some("Player".to_string()),
        "hello",
        LogType::Input,
        None,
        None,
    );
    let id = repo.insert_message(&msg).unwrap();
    assert_eq!(repo.load_messages().unwrap().len(), 1);

    repo.delete_message(id).unwrap();
    assert!(repo.load_messages().unwrap().is_empty());
}

#[test]
fn test_soft_delete_and_restore() {
    let repo = create_repo();
    let msg = Message::new(
        Some("Player".to_string()),
        "hello",
        LogType::Input,
        None,
        None,
    );
    let id = repo.insert_message(&msg).unwrap();

    repo.soft_delete_message(id).unwrap();
    assert!(repo.load_messages().unwrap().is_empty());

    repo.restore_soft_deleted(&[id]).unwrap();
    assert_eq!(repo.load_messages().unwrap().len(), 1);
}

#[test]
fn test_purge_soft_deleted() {
    let repo = create_repo();
    let msg = Message::new(
        Some("Player".to_string()),
        "hello",
        LogType::Input,
        None,
        None,
    );
    let id = repo.insert_message(&msg).unwrap();

    repo.soft_delete_message(id).unwrap();
    repo.purge_soft_deleted(&[id]).unwrap();
    assert!(repo.load_messages().unwrap().is_empty());
}

#[test]
fn test_insert_swipe_and_update_active() {
    let repo = create_repo();
    let msg = Message::new(
        Some("Player".to_string()),
        "hello",
        LogType::Narration,
        None,
        None,
    );
    let id = repo.insert_message(&msg).unwrap();

    let swipe = crate::model::message::Swipe {
        text: "swipe 1".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    repo.insert_swipe(id, &swipe, 1).unwrap();
    repo.update_active_swipe(id, 1).unwrap();

    let loaded = repo.load_messages().unwrap();
    assert_eq!(loaded[0].active_swipe_index, 1);
    assert_eq!(loaded[0].swipes.len(), 2);
}

#[test]
fn test_load_messages_empty() {
    let repo = create_repo();
    let loaded = repo.load_messages().unwrap();
    assert!(loaded.is_empty());
}
