//! [DOC: docs/diataxis/reference/game_flow.md]
//! GameViewQuery — read-side queries that don't mutate game state.

use std::sync::Arc;

use crate::adapters::driven::storage::llm_messages::LLM_MESSAGE_RETENTION_LIMIT;
use crate::adapters::driven::storage::Storage;
use crate::application::debug::DebugStateView;
use crate::application::errors::ApplicationError;
use crate::application::generation::gate::{Busy, GameWriter};
use crate::domain::model::agent::Role;
use crate::domain::model::change::ChangeVersions;
use crate::domain::model::dock::DockRecord;
use crate::domain::model::game::GameId;
use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::message::Message;
use crate::domain::model::state::generation_status::{GenerationFailure, GenerationStatus};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::story_window::{StoryWindow, StoryWindowEntry};
use crate::error::EngineError;

const STORY_WINDOW_SIZE: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleHealth {
    pub role: Role,
    pub backend_model: Option<String>,
    pub last_error: Option<String>,
}

/// The status rule: busy first, then the Game's failure line, then idle.
pub enum Status {
    /// A Job runs or an Autoplay run holds the Game.
    Busy(Busy),
    /// The Game holds a failure line.
    Failed(GenerationFailure),
    /// Nothing runs and no line stands.
    Idle,
}

#[derive(Clone)]
pub struct GameViewQuery {
    storage: Arc<Storage>,
    writer: GameWriter,
}

impl GameViewQuery {
    pub fn new(storage: Arc<Storage>, writer: GameWriter) -> Self {
        Self { storage, writer }
    }

    pub fn get_current_room_view(
        &self,
        game_id: GameId,
    ) -> Result<(String, Option<String>), ApplicationError> {
        let storage = &self.storage;
        let game = storage.require_game(game_id.0)?;
        let world_with_map = storage.require_world(&game.world_key)?;
        let world = &world_with_map.world_card;
        let map = &world_with_map.map;
        let state = self.current_state(game_id)?;
        let room_id = &state.movement.current_room_id;
        let room = state
            .room(map)
            .ok_or_else(|| ApplicationError::from(EngineError::RoomNotFound(room_id.clone())))?;

        let image_path = room
            .image_path
            .clone()
            .or_else(|| world.default_room_image.clone());

        Ok((room.name.clone(), image_path))
    }

    /// The state of the active Swipe of the last Message. Every Message of a
    /// design-valid Game holds one, so its absence is a broken record, not a room.
    fn current_state(&self, game_id: GameId) -> Result<SwipeState, ApplicationError> {
        self.storage
            .stories()
            .tail(game_id)?
            .and_then(|tail| tail.current_state().cloned())
            .ok_or_else(|| {
                ApplicationError::from(EngineError::Internal(crate::error::InternalError::new(
                    "stories_tail: the current state is missing",
                )))
            })
    }

    pub fn get_character_headshots(
        &self,
        game_id: GameId,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError> {
        let storage = &self.storage;
        let game = storage.require_game(game_id.0)?;
        let world_with_map = storage.require_world(&game.world_key)?;
        let characters_list = storage.list_characters(world_with_map.world_id)?;

        let character_ids: Vec<String> = if scene_only {
            self.current_state(game_id)?
                .scene
                .characters_present
                .iter()
                .map(|character| character.id.clone())
                .collect()
        } else {
            characters_list
                .iter()
                .map(|character| character.id.clone())
                .collect()
        };

        let character_data: Vec<(String, String)> = character_ids
            .iter()
            .filter_map(|id| {
                let character = characters_list
                    .iter()
                    .find(|character| &character.id == id)?;
                let image_path = character.sheet.preferred_image()?.to_string();
                let name = character.sheet.name.clone();
                Some((image_path, name))
            })
            .collect();

        Ok(character_data)
    }

    pub fn get_debug_state(&self, game_id: GameId) -> Result<DebugStateView, ApplicationError> {
        let state = self.current_state(game_id)?;
        let tail = self.storage.stories().tail(game_id)?;
        let count = tail.as_ref().map(|tail| tail.count).unwrap_or(0);
        let history_tail = self.storage.stories().entries(game_id, None, 5)?;
        let status = self.status(game_id)?;
        let (generation_status, step, last_error) = match status {
            Status::Busy(busy) => (GenerationStatus::Busy, busy.step().cloned(), None),
            Status::Failed(failure) => (
                GenerationStatus::Failed(failure.clone()),
                None,
                Some(failure.raw),
            ),
            Status::Idle => (GenerationStatus::Idle, None, None),
        };

        Ok(DebugStateView {
            current_room_id: state.movement.current_room_id.clone(),
            characters_present: state
                .scene
                .characters_present
                .iter()
                .map(|character| character.id.clone())
                .collect(),
            generation_status,
            step,
            encounters: state.encounters.characters.clone(),
            narration_history_tail: history_tail,
            narration_history_length: count,
            dynamic_rooms: state.movement.dynamic_rooms.keys().cloned().collect(),
            dynamic_room_count: state.movement.dynamic_rooms.len(),
            last_error,
            backend_name: None,
            model_name: None,
            current_options: self.dock(game_id)?.offered,
        })
    }

    pub fn story_window(&self, game_id: GameId) -> Result<StoryWindow, ApplicationError> {
        let read = self.storage.stories().window(game_id, STORY_WINDOW_SIZE)?;
        if read.entries.is_empty() {
            return Ok(StoryWindow {
                entries: Vec::new(),
                retrigger_allowed: read.retrigger_allowed,
            });
        }
        let game = self.storage.require_game(game_id.0)?;
        let world = self.storage.require_world(&game.world_key)?;
        let room_names: Vec<Option<String>> = read
            .entries
            .iter()
            .map(|(_, state)| state.room(&world.map).map(|room| room.name.clone()))
            .collect();
        let entries = read
            .entries
            .into_iter()
            .zip(StoryWindow::room_headers(&room_names))
            .map(|((row, _), room_header)| StoryWindowEntry { row, room_header })
            .collect();
        Ok(StoryWindow {
            entries,
            retrigger_allowed: read.retrigger_allowed,
        })
    }

    /// Every Message of the story with all its Swipes, oldest first.
    ///
    /// Includes the stored inputs of Swipes that are not active, which no production
    /// read returns.
    pub fn story_messages(&self, game_id: GameId) -> Result<Vec<Message>, ApplicationError> {
        self.storage.stories().messages(game_id).map_err(Into::into)
    }

    pub fn dock(&self, game_id: GameId) -> Result<DockRecord, ApplicationError> {
        Ok(self.storage.docks().get(game_id)?.unwrap_or_default())
    }

    pub fn status(&self, game_id: GameId) -> Result<Status, EngineError> {
        if let Some(busy) = self.writer.busy(game_id) {
            return Ok(Status::Busy(busy));
        }
        Ok(
            match self
                .storage
                .get_game(game_id.0)?
                .and_then(|game| game.failure)
            {
                Some(failure) => Status::Failed(failure),
                None => Status::Idle,
            },
        )
    }

    /// Reads memory only.
    pub fn change_versions(&self, game: Option<GameId>) -> ChangeVersions {
        self.storage.changes().versions(game)
    }

    pub fn is_busy(&self, game_id: GameId) -> bool {
        self.writer.busy(game_id).is_some()
    }

    /// True between two Jobs of the run too; a plain Job never holds.
    pub fn autoplay_holds(&self, game_id: GameId) -> bool {
        matches!(self.writer.busy(game_id), Some(Busy::Autoplay { .. }))
    }

    pub fn game_display_name(&self, game_id: GameId) -> Result<String, ApplicationError> {
        let storage = &self.storage;
        match storage.get_game(game_id.0)? {
            Some(g) => Ok(g.display_name),
            None => Ok("Unknown".to_string()),
        }
    }

    pub fn list_latest_llm_messages(
        &self,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError> {
        self.storage
            .list_latest_llm_messages(limit)
            .map_err(Into::into)
    }

    pub fn role_health(&self) -> Result<Vec<RoleHealth>, ApplicationError> {
        // Retention keeps the whole table inside one window, so reading it
        // whole sees every role's newest attempt however long ago it ran.
        let messages = self
            .storage
            .list_latest_llm_messages(LLM_MESSAGE_RETENTION_LIMIT)?;
        let health = Role::ALL
            .iter()
            .map(|role| {
                let newest = messages
                    .iter()
                    .filter(|message| message.agent_name == role.agent_name())
                    .max_by_key(|message| (message.created_at, message.id));
                RoleHealth {
                    role: *role,
                    backend_model: newest.map(|m| format!("{} {}", m.backend_name, m.model_name)),
                    last_error: newest.and_then(|m| m.error_message.clone()),
                }
            })
            .collect();
        Ok(health)
    }
}
