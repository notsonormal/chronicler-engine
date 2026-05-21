use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::EngineError;
use crate::model::game::Game;
use crate::model::message::Message;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::db::DbPool;
use crate::storage::message_storage::MessageStorage;
use crate::storage::models::game::DbGame;
use crate::storage::models::game_state_snapshot::DbGameStateSnapshot;
use crate::storage::models::message::DbMessage;

pub trait SnapshotStorage: Send + Sync {
    fn set_game_id(&self, game_id: u64);
    fn current_game_id(&self) -> u64;
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError>;
    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError>;
    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError>;

    fn list_games(&self) -> Result<Vec<Game>, EngineError>;
    fn create_game(&self, world_name: &str, name: &str) -> Result<u64, EngineError>;
    fn delete_game(&self, id: u64) -> Result<(), EngineError>;
    fn get_game(&self, id: u64) -> Result<Option<Game>, EngineError>;
}

pub struct SqliteGameStorage {
    pool: DbPool,
    game_id: AtomicU64,
}

impl SqliteGameStorage {
    pub fn new(pool: DbPool, game_id: u64) -> Self {
        Self {
            pool,
            game_id: AtomicU64::new(game_id),
        }
    }

    fn do_set_game_id(&self, game_id: u64) {
        let current = self.game_id();
        if current != game_id {
            let conn = self.pool.conn();
            let now = chrono::Utc::now().to_rfc3339();
            let _ = conn.execute(
                "UPDATE games SET updated_at = ?1 WHERE id = ?2",
                rusqlite::params![&now, game_id as i64],
            );
            self.game_id.store(game_id, Ordering::SeqCst);
        }
    }

    fn do_current_game_id(&self) -> u64 {
        self.game_id.load(Ordering::SeqCst)
    }

    fn game_id(&self) -> u64 {
        self.do_current_game_id()
    }

    pub fn reset(&self) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "DELETE FROM game_state_snapshots WHERE game_id = ?1",
            rusqlite::params![self.game_id() as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to reset snapshots: {e}")))?;
        conn.execute(
            "DELETE FROM messages WHERE game_id = ?1",
            rusqlite::params![self.game_id() as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to reset messages: {e}")))?;
        Ok(())
    }
}

fn db_game_to_game(db: &DbGame) -> Result<Game, EngineError> {
    Ok(Game {
        id: db.id as u64,
        world_name: db.world_name.clone(),
        name: db.name.clone(),
        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
            .map_err(|e| EngineError::Config(format!("Invalid created_at: {e}")))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&db.updated_at)
            .map_err(|e| EngineError::Config(format!("Invalid updated_at: {e}")))?
            .with_timezone(&chrono::Utc),
    })
}

impl SnapshotStorage for SqliteGameStorage {
    fn set_game_id(&self, game_id: u64) {
        self.do_set_game_id(game_id);
    }

    fn current_game_id(&self) -> u64 {
        self.do_current_game_id()
    }

    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError> {
        let conn = self.pool.conn();
        let db_snap = crate::storage::mappers::state_snapshot::snapshot_to_db(
            snapshot,
            self.game_id() as i64,
        )?;

        conn.execute(
            "INSERT INTO game_state_snapshots
             (game_id, movement, narrative, scene, npc_encounter_log, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                db_snap.game_id,
                db_snap.movement_json,
                db_snap.narrative_json,
                db_snap.scene_json,
                db_snap.npc_encounter_log_json,
                db_snap.created_at,
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to save snapshot: {e}")))?;

        Ok(conn.last_insert_rowid() as u64)
    }

    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, movement, narrative, scene, npc_encounter_log, created_at
                 FROM game_state_snapshots
                 WHERE game_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let db_result = stmt.query_row(rusqlite::params![self.game_id() as i64], |row| {
            Ok(DbGameStateSnapshot {
                id: row.get(0)?,
                game_id: self.game_id() as i64,
                movement_json: row.get(1)?,
                narrative_json: row.get(2)?,
                scene_json: row.get(3)?,
                npc_encounter_log_json: row.get(4)?,
                created_at: row.get(5)?,
            })
        });

        match db_result {
            Ok(db_snap) => Ok(Some(GameStateSnapshot::try_from(&db_snap)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!(
                "Failed to load latest snapshot: {e}"
            ))),
        }
    }

    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, movement, narrative, scene, npc_encounter_log, created_at
                 FROM game_state_snapshots
                 WHERE id = ?1 AND game_id = ?2",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let db_result = stmt.query_row(rusqlite::params![id, self.game_id() as i64], |row| {
            Ok(DbGameStateSnapshot {
                id: row.get(0)?,
                game_id: self.game_id() as i64,
                movement_json: row.get(1)?,
                narrative_json: row.get(2)?,
                scene_json: row.get(3)?,
                npc_encounter_log_json: row.get(4)?,
                created_at: row.get(5)?,
            })
        });

        match db_result {
            Ok(db_snap) => Ok(Some(GameStateSnapshot::try_from(&db_snap)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!(
                "Failed to load snapshot by id: {e}"
            ))),
        }
    }

    fn list_games(&self) -> Result<Vec<Game>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, world_name, name, created_at, updated_at
                 FROM games
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare list games: {e}")))?;

        let db_games: Vec<DbGame> = stmt
            .query_map([], |row| {
                Ok(DbGame {
                    id: row.get(0)?,
                    world_name: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to list games: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| EngineError::Config(format!("Failed to read game row: {e}")))?;

        db_games.iter().map(db_game_to_game).collect()
    }

    fn create_game(&self, world_name: &str, name: &str) -> Result<u64, EngineError> {
        let conn = self.pool.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO games (world_name, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            rusqlite::params![world_name, name, &now],
        )
        .map_err(|e| EngineError::Config(format!("Failed to create game: {e}")))?;
        Ok(conn.last_insert_rowid() as u64)
    }

    fn delete_game(&self, id: u64) -> Result<(), EngineError> {
        let mut conn = self.pool.conn();
        let tx = conn
            .transaction()
            .map_err(|e| EngineError::Config(format!("Failed to begin transaction: {e}")))?;
        tx.execute(
            "DELETE FROM game_state_snapshots WHERE game_id = ?1",
            rusqlite::params![id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete snapshots: {e}")))?;
        tx.execute(
            "DELETE FROM messages WHERE game_id = ?1",
            rusqlite::params![id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete messages: {e}")))?;
        tx.execute(
            "DELETE FROM games WHERE id = ?1",
            rusqlite::params![id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete game: {e}")))?;
        tx.commit()
            .map_err(|e| EngineError::Config(format!("Failed to commit transaction: {e}")))?;
        Ok(())
    }

    fn get_game(&self, id: u64) -> Result<Option<Game>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, world_name, name, created_at, updated_at
                 FROM games
                 WHERE id = ?1
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare get game: {e}")))?;

        let db_result = stmt.query_row(rusqlite::params![id as i64], |row| {
            Ok(DbGame {
                id: row.get(0)?,
                world_name: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        });

        match db_result {
            Ok(db) => Ok(Some(db_game_to_game(&db)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!("Failed to get game: {e}"))),
        }
    }
}

impl MessageStorage for SqliteGameStorage {
    fn set_game_id(&self, game_id: u64) {
        self.do_set_game_id(game_id);
    }

    fn current_game_id(&self) -> u64 {
        self.do_current_game_id()
    }

    fn insert_message(&self, msg: &mut Message) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        let db_msg = crate::storage::mappers::message::message_to_db(msg, self.game_id() as i64)?;
        conn.execute(
            "INSERT INTO messages (game_id, sender, text, log_type, timestamp, location_header, event_header, snapshot_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                db_msg.game_id,
                db_msg.sender.as_deref(),
                db_msg.text,
                db_msg.log_type_json,
                db_msg.timestamp,
                db_msg.location_header.as_deref(),
                db_msg.event_header.as_deref(),
                db_msg.snapshot_id,
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to insert message: {e}")))?;
        msg.id = conn.last_insert_rowid() as u64;
        Ok(())
    }

    fn update_message(&self, id: u64, text: &str) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "UPDATE messages SET text = ?1 WHERE id = ?2 AND game_id = ?3",
            rusqlite::params![text, id as i64, self.game_id() as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to update message: {e}")))?;
        Ok(())
    }

    fn delete_message(&self, id: u64) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "DELETE FROM messages WHERE id = ?1 AND game_id = ?2",
            rusqlite::params![id as i64, self.game_id() as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete message: {e}")))?;
        Ok(())
    }

    fn load_messages(&self) -> Result<Vec<Message>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, sender, text, log_type, timestamp, location_header, event_header, snapshot_id
                 FROM messages
                 WHERE game_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare message query: {e}")))?;

        let rows = stmt
            .query_map(rusqlite::params![self.game_id() as i64], |row| {
                Ok(DbMessage {
                    id: row.get(0)?,
                    game_id: self.game_id() as i64,
                    sender: row.get(1)?,
                    text: row.get(2)?,
                    log_type_json: row.get(3)?,
                    timestamp: row.get(4)?,
                    location_header: row.get(5)?,
                    event_header: row.get(6)?,
                    snapshot_id: row.get(7)?,
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to query messages: {e}")))?;

        rows.map(|row| {
            let db_msg =
                row.map_err(|e| EngineError::Config(format!("Failed to read message row: {e}")))?;
            Message::try_from(&db_msg)
        })
        .collect()
    }
}
