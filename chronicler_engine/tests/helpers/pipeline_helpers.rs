//! Shared pipeline helpers used by integration tests across binaries; builds a minimal `GameState` and a few derived fixtures for action-pipeline scenarios.

#![allow(dead_code)]

use std::sync::Arc;

use chronicler_engine::domain::model::character::{CharacterSheet, NpcCard, PersonaCard};
use chronicler_engine::domain::model::state::game_state::GameState;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::{
    ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
};
use chronicler_engine::domain::model::world::WorldCard;

pub fn create_test_state_with_map() -> GameState {
    let world = Arc::new(WorldCard {
        key: "test".into(),
        name: "Test World".into(),
        description: "A test world".into(),
        ..Default::default()
    });

    let map = Arc::new(crate::fixtures::create_test_map());

    let player = Arc::new(PersonaCard {
        key: "test_player".to_string(),
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
        key: "test".into(),
        name: "Test World".into(),
        description: "A test world".into(),
        ..Default::default()
    });

    let map = Arc::new(crate::fixtures::create_test_map());

    let player = Arc::new(PersonaCard {
        key: "test_player".to_string(),
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
            requirement: TriggerRequirement {
                operator: ComparisonOperator::Eq,
                threshold: 0,
            },
            narration: TriggerNarration {
                name: "Greeting".into(),
                narration_prompt: "The shopkeeper greets you.".into(),
            },
            repeat: false,
            room_id: None,
        }],
        relationships: vec![],
    }];

    GameState::new(world, map, player, npcs, "room1".to_string())
}

use chronicler_engine::application::application_service::DefaultApplicationService;

pub fn wait_for_generation_complete(app: &DefaultApplicationService, timeout_ms: u64) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);
    while start.elapsed() < timeout {
        let state = app.load_or_fresh();
        if !state.narrative.input_buffer.status.is_generating() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    false
}

pub fn latest_state(app: &DefaultApplicationService) -> GameState {
    let mut state = app.load_or_fresh();
    app.load_messages_into_state(&mut state);
    state
}

pub fn save_state(app: &DefaultApplicationService, state: &GameState) {
    use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
    let storage = app.storage();
    let snapshot = GameStateSnapshot::from_game_state(state);
    let snapshot_id = storage.save_snapshot(&snapshot).unwrap();
    let existing = app.load_messages().unwrap_or_default();
    for msg in existing {
        let _ = storage.delete_message(msg.id);
    }
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.snapshot_id().is_none() {
            msg.set_snapshot_id(Some(snapshot_id));
        }
        if let Some(swipe) = msg.swipes.first_mut() {
            swipe.snapshot_id = Some(snapshot_id);
        }
        let id = storage.insert_message(&msg).unwrap();
        for (idx, swipe) in msg.swipes.iter().enumerate() {
            let _ = storage.insert_swipe(id, swipe, idx);
        }
    }
}

pub fn add_input_and_save(app: &DefaultApplicationService, text: &str) {
    let mut state = latest_state(app);
    let player_name = state.persona.sheet.name.clone();
    state.add_message(text.to_string(), Some(player_name), MessageType::Input);
    save_state(app, &state);
}

pub fn latest_snapshot(
    app: &DefaultApplicationService,
) -> chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot {
    let state = app.load_or_fresh();
    chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
        &state,
    )
}

pub fn wait_for_condition<F>(
    timeout: std::time::Duration,
    poll_interval: std::time::Duration,
    condition: F,
) -> bool
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        std::thread::sleep(poll_interval);
    }
    false
}
