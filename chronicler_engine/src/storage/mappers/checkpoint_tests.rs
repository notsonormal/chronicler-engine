use chrono::Utc;

use crate::model::checkpoint::Checkpoint;
use crate::storage::models::checkpoint::DbCheckpoint;

#[test]
fn test_checkpoint_roundtrip() {
    let original = Checkpoint {
        id: "cp1".to_string(),
        snapshot_id: 42,
        name: "Test Checkpoint".to_string(),
        created_at: Utc::now(),
    };
    let db = DbCheckpoint::from(&original);
    let back = Checkpoint::try_from(&db).unwrap();

    assert_eq!(original.id, back.id);
    assert_eq!(original.snapshot_id, back.snapshot_id);
    assert_eq!(original.name, back.name);
    assert_eq!(original.created_at, back.created_at);
}

#[test]
fn test_checkpoint_db_fields() {
    let cp = Checkpoint {
        id: "save-1".to_string(),
        snapshot_id: 7,
        name: "Before Boss".to_string(),
        created_at: Utc::now(),
    };
    let db = DbCheckpoint::from(&cp);

    assert_eq!(db.id, "save-1");
    assert_eq!(db.snapshot_id, 7);
    assert_eq!(db.name, "Before Boss");
}
