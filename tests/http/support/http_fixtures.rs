//! HTTP test fixtures: storage seeding and world-card builders for the endpoint tests.

use std::sync::Arc;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::map::MapDef;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{active_game_of, TestMap, TestPersona, TestWorld};

/// The offered set an Options call installs, in the shape the parser reads.
pub const OFFERED_SET: &str = "<suggestion>Search the desk</suggestion>\
<suggestion>Question the guard</suggestion><suggestion>Leave the hall</suggestion>";

/// Returns `(storage, world_key, persona_key, initial_game_id)` for a world, map,
/// persona, and one game row.
pub fn seeded_storage_with_initial_game() -> (Arc<Storage>, String, String, i64) {
    let storage = Arc::new(Storage::new_in_memory());

    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPersona::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let initial_game_id = storage
        .create_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Initial Game",
        )
        .unwrap();

    (
        storage,
        world.key,
        player.key,
        initial_game_id.try_into().unwrap(),
    )
}

pub fn if_world() -> WorldCard {
    WorldCard {
        key: "if_world".to_string(),
        name: "IF World".to_string(),
        description: "An Interactive Fiction test world.".to_string(),
        narrator_mode: NarratorMode::InteractiveFiction,
        narrative_perspective: NarrativePerspective::Second,
        narrative_tense: NarrativeTense::Present,
        ..Default::default()
    }
}

/// Posture fields and the options toggle are appended by callers, so each
/// test controls exactly which fields the form carries.
pub fn world_form_body(world: &WorldCard, map: &MapDef) -> String {
    let map_json = serde_json::to_string(map).expect("map serializes");
    let scenarios_json = serde_json::to_string(&world.scenarios).expect("scenarios serialize");
    let pairs = vec![
        ("key", world.key.clone()),
        ("name", world.name.clone()),
        ("description", world.description.clone()),
        ("global_rules", world.global_rules.join("\n")),
        ("map_json", map_json),
        ("scenarios_json", scenarios_json),
    ];
    serde_urlencoded::to_string(pairs).expect("world form body serializes")
}

pub fn seed_offered_set(state: &AppState, options: &[&str]) {
    let mut game_state = state.message_service.load_or_fresh(active_game_of(state));
    game_state.narrative.current_options = options.iter().map(|o| o.to_string()).collect();
    state
        .message_service
        .save_state(active_game_of(state), &game_state)
        .expect("the offered set should persist");
}
