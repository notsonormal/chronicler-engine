use std::sync::Arc;

use crate::application::ApplicationError;
use crate::application::context::GameServiceContext;
use crate::application::game_service::DefaultGameService;
use crate::bootstrap::build_fresh_initial_state;
use crate::model::game::{Game, generate_game_name};
use crate::model::state_snapshot::GameStateSnapshot;

#[allow(dead_code)]
pub struct GameLifecycleService {
    game_service: Arc<DefaultGameService>,
}

#[allow(dead_code)]
impl GameLifecycleService {
    pub fn new(game_service: Arc<DefaultGameService>) -> Self {
        Self { game_service }
    }

    pub fn create_game(&self, ctx: GameServiceContext) -> Result<u64, ApplicationError> {
        if ctx.is_generating.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        let world_name = ctx.world.name.clone();
        let games = ctx.storage.list_games()?;
        let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        let name = generate_game_name(&world_name, &existing_names);

        let new_id = ctx.storage.create_game(&world_name, &name)?;
        let old_id = ctx.storage.current_game_id();
        ctx.set_game_id(new_id);

        let mut initial_state = build_fresh_initial_state(&ctx);
        let snapshot = GameStateSnapshot::from_game_state(&initial_state);

        let snapshot_id = match ctx.storage.save_snapshot(&snapshot) {
            Ok(id) => id,
            Err(e) => {
                ctx.set_game_id(old_id);
                return Err(ApplicationError::Engine(e));
            }
        };

        if let Some(msg) = initial_state.narrative.history.last_mut() {
            if msg.is_unpersisted() {
                msg.set_snapshot_id(Some(snapshot_id));
                match ctx.storage.insert_message(&*msg) {
                    Ok(id) => msg.id = id,
                    Err(e) => {
                        log::error!("Create game failed: could not persist message: {e}")
                    }
                }
            }
        }

        Ok(new_id)
    }

    pub fn switch_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        match ctx.storage.get_game(id)? {
            Some(game) => {
                if game.world_name != ctx.world.name {
                    return Err(ApplicationError::validation(
                        "Game belongs to a different world",
                    ));
                }
            }
            None => return Err(ApplicationError::validation("Game not found")),
        }

        ctx.set_game_id(id);
        Ok(())
    }

    pub fn delete_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        if id == ctx.storage.current_game_id() {
            return Err(ApplicationError::validation(
                "Cannot delete the active game",
            ));
        }
        ctx.storage.delete_game(id)?;
        Ok(())
    }

    pub fn list_games(&self, ctx: GameServiceContext) -> Result<Vec<Game>, ApplicationError> {
        ctx.storage.list_games().map_err(Into::into)
    }

    pub fn current_game_id(&self, ctx: GameServiceContext) -> u64 {
        ctx.storage.current_game_id()
    }

    pub fn reset(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let current_id = ctx.storage.current_game_id();
        let world_name = ctx.world.name.clone();

        ctx.storage.delete_game(current_id)?;

        let existing_names: Vec<String> = ctx
            .storage
            .list_games()?
            .into_iter()
            .filter(|g| g.world_name == world_name)
            .map(|g| g.name)
            .collect();

        let new_name = generate_game_name(&world_name, &existing_names);
        let new_id = ctx.storage.create_game(&world_name, &new_name)?;
        ctx.set_game_id(new_id);

        let mut initial_state = build_fresh_initial_state(&ctx);
        let snapshot = GameStateSnapshot::from_game_state(&initial_state);
        let snapshot_id = ctx.storage.save_snapshot(&snapshot)?;

        if let Some(msg) = initial_state.narrative.history.last_mut() {
            if msg.is_unpersisted() {
                msg.set_snapshot_id(Some(snapshot_id));
                let id = ctx.storage.insert_message(&*msg)?;
                msg.id = id;
            }
        }

        Ok(())
    }
}
