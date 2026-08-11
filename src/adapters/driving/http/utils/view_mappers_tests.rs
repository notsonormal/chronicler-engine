//! Tests for `view_mappers.rs` domain → view aggregators.

use std::collections::HashMap;

use crate::adapters::driving::http::utils::view_mappers::games_per_world;
use crate::domain::model::game::Game;

fn make_game(id: u64, world_key: &str, name: &str) -> Game {
    let now = chrono::Utc::now();
    Game {
        id,
        world_name: world_key.to_string(),
        world_key: world_key.to_string(),
        persona_key: "p".to_string(),
        persona_name: "P".to_string(),
        name: name.to_string(),
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn games_per_world_aggregates_counts_by_world_key() {
    let games = vec![
        make_game(1, "alpha", "A1"),
        make_game(2, "alpha", "A2"),
        make_game(3, "beta", "B1"),
    ];
    let counts: HashMap<String, usize> = games_per_world(&games);
    assert_eq!(counts.len(), 2);
    assert_eq!(counts.get("alpha"), Some(&2));
    assert_eq!(counts.get("beta"), Some(&1));
}

#[test]
fn games_per_world_empty_list_returns_empty_map() {
    let counts: HashMap<String, usize> = games_per_world(&[]);
    assert!(counts.is_empty());
}
