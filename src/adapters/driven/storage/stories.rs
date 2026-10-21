//! [DOC: docs/diataxis/reference/storage.md]
//! The story reads: the end of the story, one Swipe state, and the entries of a window

use crate::adapters::driven::storage::models::message::DbMessage;
use crate::adapters::driven::storage::models::swipe::DbSwipe;
use crate::adapters::driven::storage::utils::{parse_datetime, require_message_of_game};
use crate::adapters::driven::storage::{Backend, Storage, StorageCall};
use crate::domain::model::game::GameId;
use crate::domain::model::message::{Message, SwipeAt};
use crate::domain::model::state::message_types::StoryRow;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::tail::Tail;
use crate::error::{DatabaseContext, EngineError};

/// The biggest story read there is: the callers that take the whole story pass it,
/// and the assembler drops the oldest Messages to fit the connection's budget.
pub const STORY_LIMIT: usize = i32::MAX as usize;

pub struct StoryReads<'a> {
    storage: &'a Storage,
}

impl Storage {
    pub fn stories(&self) -> StoryReads<'_> {
        StoryReads { storage: self }
    }
}

impl StoryReads<'_> {
    /// No other Message and no other Swipe state is read than the two it names.
    pub fn tail(&self, game: GameId) -> Result<Option<Tail>, EngineError> {
        self.storage
            .with_backend_mut(StorageCall::StoriesTail, |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let count = Self::message_count(&conn, game)?;
                    if count == 0 {
                        return Ok(None);
                    }
                    let ids = Self::last_message_ids(&conn, game, 2)?;
                    let Some(last_id) = ids.last().copied() else {
                        return Ok(None);
                    };

                    let last = Self::load_message(&conn, last_id)?;
                    let state_before_last = match ids.iter().rev().nth(1) {
                        Some(before_id) => {
                            let at = SwipeAt {
                                message: *before_id as u64,
                                swipe: Self::active_swipe_index(&conn, *before_id)?,
                            };
                            Self::load_swipe_state(&conn, at)?.map(|state| (at, state))
                        }
                        None => None,
                    };

                    Ok(Some(Tail {
                        last,
                        state_before_last,
                        count,
                    }))
                }
            })
    }

    pub fn swipe_state(
        &self,
        game: GameId,
        at: SwipeAt,
    ) -> Result<Option<SwipeState>, EngineError> {
        self.storage
            .with_backend_mut(StorageCall::StoriesSwipeState, |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    require_message_of_game(&conn, at.message, game)?;
                    Self::load_swipe_state(&conn, at)
                }
            })
    }

    /// Every Message of the story with all its Swipes, oldest first, and no Swipe
    /// state beyond the one each Swipe holds. Only `GameViewQuery::story_messages`
    /// calls it, for tests.
    pub fn messages(&self, game: GameId) -> Result<Vec<Message>, EngineError> {
        self.storage
            .with_backend_mut(StorageCall::StoriesMessages, |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let ids = Self::last_message_ids(&conn, game, STORY_LIMIT)?;
                    ids.into_iter()
                        .map(|id| Self::load_message(&conn, id))
                        .collect()
                }
            })
    }

    /// The active Swipe of a Message of this Game, or none when the Game holds no
    /// such Message.
    pub fn active_swipe(&self, game: GameId, message: u64) -> Result<Option<SwipeAt>, EngineError> {
        self.storage
            .with_backend_mut(StorageCall::StoriesActiveSwipe, |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    match conn.query_row(
                        "SELECT active_swipe_index FROM messages WHERE id = ?1 AND game_id = ?2",
                        rusqlite::params![message as i64, game.0 as i64],
                        |row| row.get::<_, i64>(0),
                    ) {
                        Ok(swipe) => Ok(Some(SwipeAt {
                            message,
                            swipe: swipe as usize,
                        })),
                        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                        Err(e) => Err(e).db_context("Failed to read the active swipe"),
                    }
                }
            })
    }

    /// Whether any Message of the story holds text: `/options` needs something to
    /// work from. One `EXISTS` query, and no Swipe state.
    pub fn has_text(&self, game: GameId) -> Result<bool, EngineError> {
        self.storage
            .with_backend_mut(StorageCall::StoriesHasText, |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let found: i64 = conn
                        .query_row(
                            "SELECT EXISTS(SELECT 1 FROM messages m \
                             JOIN message_swipes s ON s.message_id = m.id AND s.swipe_index = m.active_swipe_index \
                             WHERE m.game_id = ?1 AND TRIM(COALESCE(s.text, '')) <> '')",
                            rusqlite::params![game.0 as i64],
                            |row| row.get(0),
                        )
                        .db_context("Failed to read the story text")?;
                    Ok(found != 0)
                }
            })
    }

    /// The `limit` newest Messages before `before` (exclusive), oldest first, and no
    /// Swipe state.
    pub fn entries(
        &self,
        game: GameId,
        before: Option<u64>,
        limit: usize,
    ) -> Result<Vec<StoryRow>, EngineError> {
        self.storage
            .with_backend_mut(StorageCall::StoriesEntries, |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let before = before.map(|id| id as i64).unwrap_or(i64::MAX);
                    let mut stmt = conn
                        .prepare(
                            "SELECT m.id, m.message_type, m.kind, m.timestamp, COALESCE(s.text, ''), s.room_report, s.event_header, \
                                    (SELECT COUNT(*) FROM message_swipes WHERE message_id = m.id), m.active_swipe_index
                             FROM messages m
                             LEFT JOIN message_swipes s ON s.message_id = m.id AND s.swipe_index = m.active_swipe_index
                             WHERE m.game_id = ?1 AND m.id < ?2
                             ORDER BY m.id DESC
                             LIMIT ?3",
                        )
                        .db_context("Failed to prepare the story entries")?;
                    let rows = stmt
                        .query_map(
                            rusqlite::params![game.0 as i64, before, limit as i64],
                            |row| {
                                Ok(StoryRow {
                                    id: row.get::<_, i64>(0)? as u64,
                                    message_type: row.get(1)?,
                                    kind: row.get(2)?,
                                    timestamp: parse_datetime(
                                        &row.get::<_, String>(3)?,
                                        "message timestamp",
                                    )
                                    .map_err(|e| {
                                        rusqlite::Error::FromSqlConversionFailure(
                                            3,
                                            rusqlite::types::Type::Text,
                                            Box::new(e),
                                        )
                                    })?,
                                    text: row.get(4)?,
                                    room_report: row.get(5)?,
                                    event_header: row.get(6)?,
                                    swipe_count: row.get::<_, i64>(7)? as usize,
                                    active_swipe_index: row.get::<_, i64>(8)? as usize,
                                })
                            },
                        )
                        .db_context("Failed to read the story entries")?;

                    let mut entries: Vec<StoryRow> = rows
                        .collect::<Result<Vec<_>, _>>()
                        .db_context("Failed to read a story entry")?;
                    entries.reverse();
                    Ok(entries)
                }
            })
    }

    fn message_count(conn: &rusqlite::Connection, game: GameId) -> Result<usize, EngineError> {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE game_id = ?1",
                rusqlite::params![game.0 as i64],
                |row| row.get(0),
            )
            .db_context("Failed to count the story")?;
        Ok(count as usize)
    }

    fn last_message_ids(
        conn: &rusqlite::Connection,
        game: GameId,
        limit: usize,
    ) -> Result<Vec<i64>, EngineError> {
        let mut stmt = conn
            .prepare(
                "SELECT id FROM (SELECT id FROM messages WHERE game_id = ?1 ORDER BY id DESC LIMIT ?2) ORDER BY id ASC",
            )
            .db_context("Failed to prepare the story ids")?;
        let rows = stmt
            .query_map(rusqlite::params![game.0 as i64, limit as i64], |row| {
                row.get::<_, i64>(0)
            })
            .db_context("Failed to read the story ids")?;
        rows.collect::<Result<Vec<_>, _>>()
            .db_context("Failed to read a story id")
    }

    fn load_message(conn: &rusqlite::Connection, id: i64) -> Result<Message, EngineError> {
        let db_message = conn
            .query_row(
                "SELECT id, game_id, message_type, kind, timestamp, active_swipe_index FROM messages WHERE id = ?1",
                rusqlite::params![id],
                DbMessage::from_row,
            )
            .db_context("Failed to read the message")?;
        let swipes = Self::load_swipe_rows(conn, id)?;
        Message::try_from((&db_message, &swipes[..]))
    }

    fn load_swipe_rows(
        conn: &rusqlite::Connection,
        message: i64,
    ) -> Result<Vec<DbSwipe>, EngineError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, message_id, swipe_index, text, room_report, event_header, impersonated, guide_text, impersonate_direction, state, maker
                 FROM message_swipes WHERE message_id = ?1 ORDER BY swipe_index ASC",
            )
            .db_context("Failed to prepare the swipe rows")?;
        let rows = stmt
            .query_map(rusqlite::params![message], DbSwipe::from_row)
            .db_context("Failed to read the swipe rows")?;
        rows.collect::<Result<Vec<_>, _>>()
            .db_context("Failed to read a swipe row")
    }

    fn load_swipe_state(
        conn: &rusqlite::Connection,
        at: SwipeAt,
    ) -> Result<Option<SwipeState>, EngineError> {
        let state_json = match conn.query_row(
            "SELECT state FROM message_swipes WHERE message_id = ?1 AND swipe_index = ?2",
            rusqlite::params![at.message as i64, at.swipe as i64],
            |row| row.get::<_, String>(0),
        ) {
            Ok(json) => json,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(other) => {
                return Err(other).db_context("Failed to read the swipe state");
            }
        };
        Ok(Some(serde_json::from_str(&state_json)?))
    }

    fn active_swipe_index(conn: &rusqlite::Connection, message: i64) -> Result<usize, EngineError> {
        let index: i64 = conn
            .query_row(
                "SELECT active_swipe_index FROM messages WHERE id = ?1",
                rusqlite::params![message],
                |row| row.get(0),
            )
            .db_context("Failed to read the active swipe index")?;
        Ok(index as usize)
    }
}
