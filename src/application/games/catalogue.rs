//! [DOC: docs/diataxis/reference/game_flow.md]
//! GameCatalogue — game-lifecycle storage orchestration.

use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::application::message_service::MessageService;
use crate::domain::model::game::Game;
use crate::domain::model::utils::game_name::generate_game_name;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::adapters::driven::storage::Storage;

#[derive(Clone)]
pub struct GameCatalogue {
    storage: Arc<Storage>,
    message_service: Arc<MessageService>,
}

impl GameCatalogue {
    pub fn new(storage: Arc<Storage>, message_service: Arc<MessageService>) -> Self {
        Self {
            storage,
            message_service,
        }
    }

    pub fn create_game(&self, world_key: &str, persona_key: &str) -> Result<u64, ApplicationError> {
        let storage = &self.storage;
        let world_with_map = storage
            .get_world(world_key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let world_name = world_with_map.world_card.name.clone();
        let player = storage
            .get_persona(persona_key)?
            .ok_or_else(|| ApplicationError::validation("Persona not found"))?;
        let games = storage.list_games()?;
        let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        let name = generate_game_name(&world_name, &existing_names);

        let new_id = storage.create_game(
            &world_name,
            world_key,
            persona_key,
            &player.sheet.name,
            &name,
        )?;
        let old_id = storage.current_game_id();
        self.storage.set_game_id(new_id);

        match self.persist_initial_state_with_swipes() {
            Ok(_) => {}
            Err(e) => {
                self.storage.set_game_id(old_id);
                return Err(e);
            }
        }

        Ok(new_id)
    }

    pub fn switch_game(&self, id: u64) -> Result<(), ApplicationError> {
        if self.storage.get_game(id)?.is_none() {
            return Err(ApplicationError::validation("Game not found"));
        }

        self.storage.set_game_id(id);
        Ok(())
    }

    pub fn delete_game(&self, id: u64) -> Result<(), ApplicationError> {
        if id == self.storage.current_game_id() {
            return Err(ApplicationError::validation(
                "Cannot delete the active game",
            ));
        }
        self.storage.delete_game(id)?;
        Ok(())
    }

    pub fn list_games(&self) -> Result<Vec<Game>, ApplicationError> {
        self.storage.list_games().map_err(Into::into)
    }

    pub fn current_game_id(&self) -> u64 {
        self.storage.current_game_id()
    }

    pub fn reset(&self) -> Result<(), ApplicationError> {
        let storage = &self.storage;
        let current_id = storage.current_game_id();
        let game = storage
            .get_game(current_id)?
            .ok_or_else(|| ApplicationError::validation("Current game not found"))?;
        let world_key = game.world_key.clone();
        let persona_key = game.persona_key.clone();

        let world_with_map = storage
            .get_world(&world_key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let world_name = world_with_map.world_card.name.clone();
        let player = storage
            .get_persona(&persona_key)?
            .ok_or_else(|| ApplicationError::validation("Persona not found"))?;

        storage.delete_game(current_id)?;

        let existing_names: Vec<String> = storage
            .list_games()?
            .into_iter()
            .filter(|g| g.world_key == world_key)
            .map(|g| g.name)
            .collect();

        let new_name = generate_game_name(&world_name, &existing_names);
        let new_id = storage.create_game(
            &world_name,
            &world_key,
            &persona_key,
            &player.sheet.name,
            &new_name,
        )?;
        self.storage.set_game_id(new_id);

        let _ = self.persist_initial_state_with_swipes();

        Ok(())
    }

    fn persist_initial_state_with_swipes(&self) -> Result<u64, ApplicationError> {
        let mut initial_state = self.message_service.build_fresh_initial_state()?;
        let storage = &self.storage;
        let snapshot = GameStateSnapshot::from_game_state(&initial_state);
        let snapshot_id = storage.save_snapshot(&snapshot)?;

        if let Some(msg) = initial_state.narrative.history.last_mut() {
            if msg.is_unpersisted() {
                msg.set_snapshot_id(Some(snapshot_id));
                match storage.insert_message(&*msg) {
                    Ok(id) => {
                        msg.id = id;
                        storage.persist_swipes(id, &msg.swipes);
                    }
                    Err(e) => {
                        tracing::error!("persist_initial_state: message insert failed: {e}");
                    }
                }
            }
        }

        Ok(snapshot_id)
    }
}
