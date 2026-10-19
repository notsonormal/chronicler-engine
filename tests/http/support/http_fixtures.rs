//! HTTP test fixtures: storage seeding and world-card builders for the endpoint tests.

use std::sync::Arc;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::dock::DockRecord;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::message::{Maker, Message, Swipe, SwipeAt};
use chronicler_engine::domain::model::state::swipe_state::SwipeState;
use chronicler_engine::domain::model::state::message_types::{MessageKind, MessageType};
use chronicler_engine::domain::model::map::MapDef;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{
    active_game_of, empty_sqlite_storage, tx_token, TestMap, TestPersona, TestWorld,
};

/// The offered set an Options call installs, in the shape the parser reads.
pub const OFFERED_SET: &str = "<suggestion>Search the desk</suggestion>\
<suggestion>Question the guard</suggestion><suggestion>Leave the hall</suggestion>";

/// Returns `(storage, world_key, persona_key, initial_game_id)` for a world, map,
/// persona, and one game row.
pub fn seeded_storage_with_initial_game() -> (Arc<Storage>, String, String, i64) {
    let storage = Arc::new(empty_sqlite_storage());

    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPersona::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let initial_game_id = chronicler_engine::test_support::seed_game_row(
        &storage,
        &chronicler_engine::test_support::test_new_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Initial Game",
        ),
    )
    .0;

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
    let record = DockRecord {
        offered: options.iter().map(|option| (*option).to_string()).collect(),
        notice: None,
    };
    state
        .job_deps
        .storage
        .transaction(active_game_of(state), tx_token(), |tx| tx.put_dock(&record))
        .expect("the offered set should persist");
}

/// Pads the story with `count` Narration Messages, each with one Swipe that holds
/// the current state. They carry no Input and no room report, so a test that
/// needs a long story reads padding and not a turned scene.
pub fn append_narrations(
    storage: &Storage,
    game: GameId,
    count: usize,
    text: impl Fn(usize) -> String,
) {
    let tail = storage
        .stories()
        .tail(game)
        .expect("the story reads back")
        .expect("a Game holds its story");
    let state = tail.current_state().cloned().expect("a current state");
    storage
        .transaction(game, tx_token(), |tx| {
            for index in 0..count {
                let narration = text(index);
                let swipe = Swipe {
                    room_report: None,
                    ..Swipe::new(narration, Maker::Narrator, state.clone())
                };
                let message = Message::new(
                    MessageType::Narration,
                    MessageKind::Narration,
                    swipe.clone(),
                );
                let id = tx.insert_message(&message)?;
                tx.insert_swipe(
                    SwipeAt {
                        message: id,
                        swipe: 0,
                    },
                    &swipe,
                )?;
            }
            Ok(())
        })
        .expect("the seeded story should persist");
}

/// Rewrites the Swipe state that the next generation starts from, so a test can
/// present a broken record that no client request writes.
pub fn corrupt_start_state(storage: &Storage, game: GameId, edit: impl FnOnce(&mut SwipeState)) {
    let tail = storage
        .stories()
        .tail(game)
        .expect("the story reads back")
        .expect("a Game holds its story");
    let at = tail.start_at();
    let mut broken = storage
        .stories()
        .swipe_state(game, at)
        .expect("the Swipe state reads back")
        .expect("a generated Swipe holds a state");
    edit(&mut broken);
    storage
        .transaction(game, tx_token(), |tx| tx.set_swipe_state(at, &broken, None))
        .expect("the broken state should persist");
}

pub fn break_start_room(storage: &Storage, game: GameId, room: &str) {
    corrupt_start_state(storage, game, |state| {
        state.movement.current_room_id = room.to_string();
    });
}
