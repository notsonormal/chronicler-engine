use crate::model::message::Swipe;
use crate::storage::db::DbPool;
use crate::storage::message_swipe_storage::{MessageSwipeStorage, SqliteMessageSwipeRepository};

fn create_repo() -> (DbPool, SqliteMessageSwipeRepository) {
    let pool = DbPool::new(":memory:").unwrap();
    let repo = SqliteMessageSwipeRepository::new(pool.clone());
    (pool, repo)
}

/// Insert a bare message row (no swipes) so tests are independent of
/// MessageStorage::insert_message's bundled behavior.
fn insert_raw_message(pool: &DbPool, game_id: i64) -> i64 {
    let conn = pool.conn();
    conn.execute(
        "INSERT INTO messages (game_id, sender, log_type, timestamp, active_swipe_index, is_deleted)
         VALUES (?1, NULL, 'Narration', '2024-01-01T00:00:00Z', 0, 0)",
        rusqlite::params![game_id],
    )
    .unwrap();
    conn.last_insert_rowid()
}

#[test]
fn test_insert_swipe() {
    let (pool, repo) = create_repo();
    let message_id = insert_raw_message(&pool, 1) as u64;

    let swipe = Swipe {
        text: "swipe 1".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    repo.insert_swipe(message_id, &swipe, 0).unwrap();

    let loaded = repo.load_swipes_for_messages(&[message_id]).unwrap();
    assert_eq!(loaded.get(&message_id).map(|v| v.len()), Some(1));
    assert_eq!(
        loaded.get(&message_id).map(|v| v[0].text.clone()),
        Some("swipe 1".to_string())
    );
}

#[test]
fn test_update_swipe_text() {
    let (pool, repo) = create_repo();
    let message_id = insert_raw_message(&pool, 1) as u64;

    let swipe = Swipe {
        text: "original".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    };
    repo.insert_swipe(message_id, &swipe, 0).unwrap();
    repo.update_swipe_text(message_id, 0, "updated").unwrap();

    let loaded = repo.load_swipes_for_messages(&[message_id]).unwrap();
    assert_eq!(loaded[&message_id][0].text, "updated");
}

#[test]
fn test_shift_swipe_indices() {
    let (pool, repo) = create_repo();
    let message_id = insert_raw_message(&pool, 1) as u64;

    // Start with one swipe at index 0
    repo.insert_swipe(
        message_id,
        &Swipe {
            text: "first".to_string(),
            snapshot_id: None,
            location_header: None,
            event_header: None,
        },
        0,
    )
    .unwrap();

    // Shift by 1 to make room at index 0 (0 -> 1, no collision)
    repo.shift_swipe_indices(message_id, 1).unwrap();

    // Insert new swipe at index 0
    repo.insert_swipe(
        message_id,
        &Swipe {
            text: "new first".to_string(),
            snapshot_id: None,
            location_header: None,
            event_header: None,
        },
        0,
    )
    .unwrap();

    let loaded = repo.load_swipes_for_messages(&[message_id]).unwrap();
    let swipes = &loaded[&message_id];
    assert_eq!(swipes.len(), 2);
    assert_eq!(swipes[0].text, "new first");
    assert_eq!(swipes[1].text, "first");
}

#[test]
fn test_load_swipes_for_multiple_messages() {
    let (pool, repo) = create_repo();
    let id1 = insert_raw_message(&pool, 1) as u64;
    let id2 = insert_raw_message(&pool, 1) as u64;

    repo.insert_swipe(
        id1,
        &Swipe {
            text: "msg1 swipe".to_string(),
            snapshot_id: None,
            location_header: None,
            event_header: None,
        },
        0,
    )
    .unwrap();
    repo.insert_swipe(
        id2,
        &Swipe {
            text: "msg2 swipe".to_string(),
            snapshot_id: None,
            location_header: None,
            event_header: None,
        },
        0,
    )
    .unwrap();

    let loaded = repo.load_swipes_for_messages(&[id1, id2]).unwrap();
    assert_eq!(loaded[&id1].len(), 1);
    assert_eq!(loaded[&id2].len(), 1);
    assert_eq!(loaded[&id1][0].text, "msg1 swipe");
    assert_eq!(loaded[&id2][0].text, "msg2 swipe");
}

#[test]
fn test_load_swipes_for_empty_ids() {
    let (_pool, repo) = create_repo();
    let loaded = repo.load_swipes_for_messages(&[]).unwrap();
    assert!(loaded.is_empty());
}
