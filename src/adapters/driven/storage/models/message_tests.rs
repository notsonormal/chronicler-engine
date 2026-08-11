use rusqlite::Connection;

use super::message::{DbMessage, DbSwipe};

#[test]
fn test_db_message_from_row_maps_columns() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE m (id INTEGER, game_id INTEGER, sender TEXT, \
         message_type_json TEXT, timestamp TEXT, active_swipe_index INTEGER);\
         INSERT INTO m VALUES (7, 3, 'npc', '\"Narration\"', '2026-01-01', 2);",
    )
    .unwrap();

    let msg = conn
        .query_row(
            "SELECT id, game_id, sender, message_type_json, timestamp, active_swipe_index FROM m",
            [],
            DbMessage::from_row,
        )
        .unwrap();

    assert_eq!(msg.id, 7);
    assert_eq!(msg.game_id, 3);
    assert_eq!(msg.sender.as_deref(), Some("npc"));
    assert_eq!(msg.message_type_json, "\"Narration\"");
    assert_eq!(msg.timestamp, "2026-01-01");
    assert_eq!(msg.active_swipe_index, 2);
    // from_row always initializes is_deleted to 0; it is not read from the row.
    assert_eq!(msg.is_deleted, 0);
}

#[test]
fn test_db_message_from_row_null_sender() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE m (id INTEGER, game_id INTEGER, sender TEXT, \
         message_type_json TEXT, timestamp TEXT, active_swipe_index INTEGER);\
         INSERT INTO m VALUES (1, 1, NULL, '\"Input\"', 't', 0);",
    )
    .unwrap();

    let msg = conn
        .query_row(
            "SELECT id, game_id, sender, message_type_json, timestamp, active_swipe_index FROM m",
            [],
            DbMessage::from_row,
        )
        .unwrap();

    assert_eq!(msg.sender, None);
}

#[test]
fn test_db_swipe_from_row_maps_columns() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE s (id INTEGER, message_id INTEGER, swipe_index INTEGER, \
         text TEXT, snapshot_id INTEGER, location_header TEXT, event_header TEXT);\
         INSERT INTO s VALUES (5, 9, 1, 'hello', 42, 'Town', 'Battle');",
    )
    .unwrap();

    let swipe = conn
        .query_row(
            "SELECT id, message_id, swipe_index, text, snapshot_id, location_header, event_header \
             FROM s",
            [],
            DbSwipe::from_row,
        )
        .unwrap();

    assert_eq!(swipe.id, 5);
    assert_eq!(swipe.message_id, 9);
    assert_eq!(swipe.swipe_index, 1);
    assert_eq!(swipe.text, "hello");
    assert_eq!(swipe.snapshot_id, Some(42));
    assert_eq!(swipe.location_header.as_deref(), Some("Town"));
    assert_eq!(swipe.event_header.as_deref(), Some("Battle"));
}

#[test]
fn test_db_swipe_from_row_null_optionals() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE s (id INTEGER, message_id INTEGER, swipe_index INTEGER, \
         text TEXT, snapshot_id INTEGER, location_header TEXT, event_header TEXT);\
         INSERT INTO s VALUES (1, 1, 0, 't', NULL, NULL, NULL);",
    )
    .unwrap();

    let swipe = conn
        .query_row(
            "SELECT id, message_id, swipe_index, text, snapshot_id, location_header, event_header \
             FROM s",
            [],
            DbSwipe::from_row,
        )
        .unwrap();

    assert_eq!(swipe.snapshot_id, None);
    assert_eq!(swipe.location_header, None);
    assert_eq!(swipe.event_header, None);
}
