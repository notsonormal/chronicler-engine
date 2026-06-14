//! [DOC: docs/system/storage.md]
//! Game database model

pub struct DbGame {
    pub id: i64,
    pub world_name: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub world_key: String,
}
