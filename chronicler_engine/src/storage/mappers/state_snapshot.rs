use chrono::{DateTime, Utc};

use crate::error::EngineError;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::models::game_state_snapshot::DbGameStateSnapshot;

impl TryFrom<&DbGameStateSnapshot> for GameStateSnapshot {
    type Error = EngineError;

    fn try_from(db: &DbGameStateSnapshot) -> Result<Self, Self::Error> {
        let movement = serde_json::from_str(&db.movement_json)
            .map_err(|e| EngineError::Config(format!("Failed to parse snapshot movement: {e}")))?;
        let narrative = serde_json::from_str(&db.narrative_json)
            .map_err(|e| EngineError::Config(format!("Failed to parse snapshot narrative: {e}")))?;
        let scene = serde_json::from_str(&db.scene_json)
            .map_err(|e| EngineError::Config(format!("Failed to parse snapshot scene: {e}")))?;
        let character_state = serde_json::from_str(&db.character_state_json).map_err(|e| {
            EngineError::Config(format!("Failed to parse snapshot character_state: {e}"))
        })?;
        let created_at = DateTime::parse_from_rfc3339(&db.created_at)
            .map_err(|e| EngineError::Config(format!("Failed to parse snapshot created_at: {e}")))?
            .with_timezone(&Utc);

        Ok(GameStateSnapshot {
            db_id: Some(db.id as u64),
            movement,
            narrative,
            scene,
            character_state,
            committed: db.committed != 0,
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
    let character_state_json = serde_json::to_string(&snapshot.character_state)
        .map_err(|e| EngineError::Config(format!("Failed to serialize character_state: {e}")))?;

    Ok(DbGameStateSnapshot {
        id: snapshot.db_id.map(|id| id as i64).unwrap_or(0),
        game_id,
        movement_json,
        narrative_json,
        scene_json,
        character_state_json,
        committed: if snapshot.committed { 1 } else { 0 },
        created_at: snapshot.created_at.to_rfc3339(),
    })
}
