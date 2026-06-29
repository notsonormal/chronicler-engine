//! [DOC: docs/system/storage.md]
//! State snapshot mapper

use chrono::{DateTime, Utc};

use crate::error::EngineError;
use crate::adapters::driven::storage::snapshot_blob::GameStateSnapshot;
use crate::adapters::driven::storage::models::game_state_snapshot::DbGameStateSnapshot;

impl TryFrom<&DbGameStateSnapshot> for GameStateSnapshot {
    type Error = EngineError;

    fn try_from(db: &DbGameStateSnapshot) -> Result<Self, Self::Error> {
        let movement = serde_json::from_str(&db.movement_json)
            .map_err(|e| EngineError::Config(format!("Failed to parse snapshot movement: {e}")))?;
        let narrative = serde_json::from_str(&db.narrative_json)
            .map_err(|e| EngineError::Config(format!("Failed to parse snapshot narrative: {e}")))?;
        let scene = serde_json::from_str(&db.scene_json)
            .map_err(|e| EngineError::Config(format!("Failed to parse snapshot scene: {e}")))?;
        let npc_encounter_log = serde_json::from_str(&db.npc_encounter_log_json).map_err(|e| {
            EngineError::Config(format!("Failed to parse snapshot npc_encounter_log: {e}"))
        })?;
        let created_at = DateTime::parse_from_rfc3339(&db.created_at)
            .map_err(|e| EngineError::Config(format!("Failed to parse snapshot created_at: {e}")))?
            .with_timezone(&Utc);

        Ok(GameStateSnapshot {
            db_id: Some(db.id as u64),
            movement,
            narrative,
            scene,
            npc_encounter_log,
            created_at,
        })
    }
}

pub fn snapshot_to_db(
    snapshot: &GameStateSnapshot,
    game_id: i64,
) -> Result<DbGameStateSnapshot, EngineError> {
    let movement_json = serde_json::to_string(&snapshot.movement)
        .map_err(|e| EngineError::Config(format!("Failed to serialize movement: {e}")))?;
    let narrative_json = serde_json::to_string(&snapshot.narrative)
        .map_err(|e| EngineError::Config(format!("Failed to serialize narrative: {e}")))?;
    let scene_json = serde_json::to_string(&snapshot.scene)
        .map_err(|e| EngineError::Config(format!("Failed to serialize scene: {e}")))?;
    let npc_encounter_log_json = serde_json::to_string(&snapshot.npc_encounter_log)
        .map_err(|e| EngineError::Config(format!("Failed to serialize npc_encounter_log: {e}")))?;

    Ok(DbGameStateSnapshot {
        id: snapshot.db_id.map(|id| id as i64).unwrap_or(0),
        game_id,
        movement_json,
        narrative_json,
        scene_json,
        npc_encounter_log_json,
        created_at: snapshot.created_at.to_rfc3339(),
    })
}
