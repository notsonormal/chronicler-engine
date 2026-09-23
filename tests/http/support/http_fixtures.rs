//! HTTP test fixtures: storage seeding and world-card builders for the endpoint tests.

use std::sync::Arc;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::domain::model::map::MapDef;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{TestMap, TestPersona, TestWorld};

/// Seed an in-memory storage with a world, map, and persona, then create an
/// initial game and set it current. Returns `(storage, world_key, persona_key,
/// initial_game_id)`. Shared by the `games_create` / `games_switch` /
/// `games_delete` HTTP E2E tests to dedupe their setup shape.
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
    storage.set_game_id(initial_game_id);

    (
        storage,
        world.key,
        player.key,
        initial_game_id.try_into().unwrap(),
    )
}

/// An Interactive Fiction world with second-person present-tense posture.
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

/// Build a urlencoded `WorldForm` body for the worlds update endpoint.
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
