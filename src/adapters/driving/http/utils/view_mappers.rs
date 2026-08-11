//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Domain → view aggregators used by HTTP handlers. Distinct from `mappers/`, which convert DB rows ↔ domain.

use crate::adapters::driving::http::games::templates::GameRowView;
use crate::domain::model::game::Game;

pub fn game_to_view(game: Game) -> GameRowView {
    GameRowView {
        id: game.id,
        name: game.name.clone(),
        world_name: game.world_name.clone(),
        persona_name: game.persona_name.clone(),
    }
}

pub fn games_per_world(games: &[Game]) -> std::collections::HashMap<String, usize> {
    let mut game_counts_by_world: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for game in games {
        *game_counts_by_world
            .entry(game.world_key.clone())
            .or_insert(0) += 1;
    }
    game_counts_by_world
}
