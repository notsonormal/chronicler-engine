//! Sequential flow tests using direct service calls with mock backends.

mod test_data;

use std::sync::Arc;

use chronicler_engine::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::model::state::{GameState, LogType};
use chronicler_engine::model::trigger::{
    ComparisonOperator, Trigger, TriggerAction, TriggerCondition,
};
use chronicler_engine::model::world::WorldCard;
#[path = "flow_mock/retry_event.rs"]
mod retry_event;
#[path = "flow_mock/retry_main.rs"]
mod retry_main;
#[path = "flow_mock/sequence.rs"]
mod sequence;

pub fn create_test_state_with_map() -> GameState {
    let world = Arc::new(WorldCard {
        name: "Test World".into(),
        description: "A test world".into(),
        global_rules: vec![],
        starting_room_id: "room1".into(),
        scenarios: vec![],
        default_room_image: None,
    });

    let map = Arc::new(test_data::create_test_map());

    let player = Arc::new(chronicler_engine::model::character::PlayerCard {
        sheet: CharacterSheet {
            name: "Test Player".into(),
            description: "A test player".into(),
            personality: "Brave".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let npcs = vec![NpcCard {
        id: "test_npc".into(),
        sheet: CharacterSheet {
            name: "Innkeeper".into(),
            description: "A friendly innkeeper".into(),
            personality: "Helpful".into(),
            scenario: "Runs the tavern".into(),
            example_dialogue: "Welcome!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    }];

    GameState::new(world, map, player, npcs, "room1".to_string())
}

pub fn create_test_state_with_trigger_npc() -> GameState {
    let world = Arc::new(WorldCard {
        name: "Test World".into(),
        description: "A test world".into(),
        global_rules: vec![],
        starting_room_id: "room1".into(),
        scenarios: vec![],
        default_room_image: None,
    });

    let map = Arc::new(test_data::create_test_map());

    let player = Arc::new(chronicler_engine::model::character::PlayerCard {
        sheet: CharacterSheet {
            name: "Test Player".into(),
            description: "A test player".into(),
            personality: "Brave".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let npcs = vec![NpcCard {
        id: "shopkeeper".into(),
        sheet: CharacterSheet {
            name: "Shopkeeper Sarah".into(),
            description: "A shrewd shopkeeper".into(),
            personality: "Business-minded".into(),
            scenario: "Runs the shop".into(),
            example_dialogue: "Welcome!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![Trigger {
            condition: TriggerCondition::TimesMet(ComparisonOperator::Eq, 0),
            action: TriggerAction {
                name: "Greeting".into(),
                narration_prompt: "The shopkeeper looks up with a smile.".into(),
            },
            repeat: false,
            room_id: None,
        }],
        relationships: vec![],
    }];

    GameState::new(world, map, player, npcs, "room1".to_string())
}

pub fn wait_for_generation_complete(
    ctx: &chronicler_engine::engine::game_service::GameServiceContext,
    timeout_ms: u64,
) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);
    while start.elapsed() < timeout {
        if let Ok(Some(snap)) = ctx.snapshot_storage.load_latest(None) {
            let guard = GameState::from_snapshot(
                &snap,
                ctx.world.clone(),
                ctx.map.clone(),
                ctx.player.clone(),
                (*ctx.npcs).clone(),
            );
            if !guard.narrative.generation.status.is_generating() {
                return true;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    false
}

pub fn latest_state(
    ctx: &chronicler_engine::engine::game_service::GameServiceContext,
) -> GameState {
    let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
    GameState::from_snapshot(
        &snap,
        ctx.world.clone(),
        ctx.map.clone(),
        ctx.player.clone(),
        (*ctx.npcs).clone(),
    )
}

pub fn add_input_and_save(
    ctx: &chronicler_engine::engine::game_service::GameServiceContext,
    text: &str,
) {
    let mut state = latest_state(ctx);
    let player_name = state.player.sheet.name.clone();
    state.add_log(text.to_string(), Some(player_name), LogType::Input);
    let snapshot = chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        uuid::Uuid::new_v4().to_string(),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);
}

pub fn latest_snapshot(
    ctx: &chronicler_engine::engine::game_service::GameServiceContext,
) -> Option<chronicler_engine::model::state_snapshot::GameStateSnapshot> {
    ctx.snapshot_storage.load_latest(None).unwrap_or(None)
}
