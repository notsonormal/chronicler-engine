//! [DOC: docs/diataxis/reference/storage.md]
//! Game state snapshot model

pub struct DbGameStateSnapshot {
    pub id: i64,
    pub game_id: i64,
    pub movement_json: String,
    pub narrative_json: String,
    pub scene_json: String,
    pub npc_encounter_log_json: String,
    pub created_at: String,
}

impl DbGameStateSnapshot {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbGameStateSnapshot {
            id: row.get(0)?,
            game_id: row.get(1)?,
            movement_json: row.get(2)?,
            narrative_json: row.get(3)?,
            scene_json: row.get(4)?,
            npc_encounter_log_json: row.get(5)?,
            created_at: row.get(6)?,
        })
    }
}
