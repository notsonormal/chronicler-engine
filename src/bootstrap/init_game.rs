//! [DOC: docs/diataxis/reference/startup.md]
//! Game state initialization and game resolution

use crate::application::errors::ApplicationError;
use crate::application::games::catalogue::GameCatalogue;
use crate::domain::model::game::GameId;

pub(crate) fn resolve_game_id(
    catalogue: &GameCatalogue,
    world_key: &str,
    persona_key: &str,
) -> Result<GameId, ApplicationError> {
    match catalogue.newest_game_of_world(world_key)? {
        Some(game_id) => {
            tracing::info!("Loaded existing game (id={game_id})");
            Ok(game_id)
        }
        None => {
            let id = catalogue.create_game(world_key, persona_key)?;
            tracing::info!("Created new game (id={id}) with persona '{persona_key}'");
            Ok(GameId(id))
        }
    }
}
