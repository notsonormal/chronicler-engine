//! Integration flow tests for the retry-event handler: no extra swipe on narration retry, quantifier-result preservation on continuation, and trigger continuations re-running the quantifier to detect newly-relevant NPCs.

use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::domain::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::domain::model::state::game_state::GameState;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::{
    ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::test_support::make_test_recorder;
use chronicler_engine::test_support::TestData;
use chronicler_engine::TestDataBuilder;
use crate::make_test_recorder_with_storage;
use chronicler_engine::application::action_pipeline::{execute_action_impl, retry_last_response_impl};

use crate::pipeline_helpers::{add_input_and_save, latest_state, wait_for_generation_complete};
use crate::fixtures::create_test_map;
use crate::sqlite_test_app_builder::SqliteTestAppBuilder;

fn trigger_npc_test_data() -> TestData {
    let shopkeeper = NpcCard {
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
    };

    TestDataBuilder::default_test()
        .world(crate::fixtures::create_test_world())
        .map(crate::fixtures::create_test_map())
        .persona(crate::fixtures::create_test_player())
        .npcs(vec![shopkeeper])
        .build()
}

#[test]
fn test_event_retry_does_not_create_extra_swipe_on_narration() {
    let data = trigger_npc_test_data();

    let app1 = SqliteTestAppBuilder::with_data(data.clone())
        .mock_backend(MockBackend::new)
        .build_service()
        .unwrap();

    add_input_and_save(&app1, "enter shop");

    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ]));

    let app2 = SqliteTestAppBuilder::with_data(data)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
                quantifier.clone(),
            ))
        })
        .build_service()
        .unwrap();

    execute_action_impl(&app2, "enter shop".to_string());
    assert!(
        wait_for_generation_complete(&app2, 1000),
        "Execute should complete"
    );

    retry_last_response_impl(&app2);
    assert!(
        wait_for_generation_complete(&app2, 1000),
        "Event retry should complete"
    );

    let guard = latest_state(&app2);
    let narration_msgs: Vec<_> = guard
        .narrative
        .history
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narration_msgs.len(),
        2,
        "Should have exactly 2 Narration messages"
    );
    assert!(
        !narration_msgs[0].text().is_empty(),
        "Main narration must have text after event retry"
    );
}

#[test]
fn test_retry_event_continuation_preserves_quantifier_result() {
    let data = trigger_npc_test_data();

    let app1 = SqliteTestAppBuilder::with_data(data.clone())
        .mock_backend(MockBackend::new)
        .build_service()
        .unwrap();

    add_input_and_save(&app1, "enter shop");

    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ]));

    let app2 = SqliteTestAppBuilder::with_data(data)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
                quantifier.clone(),
            ))
        })
        .build_service()
        .unwrap();

    execute_action_impl(&app2, "enter shop".to_string());
    assert!(
        wait_for_generation_complete(&app2, 1000),
        "Execute should complete"
    );
    let guard = latest_state(&app2);
    assert_eq!(
        guard.movement.current_room_id, "room2",
        "Execute: player should have moved to room2"
    );
    let event_count = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.event_header.is_some())
        .count();
    assert_eq!(
        event_count, 1,
        "Trigger should have fired and added an Event"
    );

    retry_last_response_impl(&app2);
    assert!(
        wait_for_generation_complete(&app2, 1000),
        "Event retry should complete"
    );
    let guard = latest_state(&app2);
    assert_eq!(
        guard.movement.current_room_id, "room2",
        "Event retry: room should be unchanged (quantifier not rerun)"
    );

    let messages = app2.storage().list_latest_llm_messages(50).unwrap();
    assert!(
        !messages.is_empty(),
        "LLM messages should be logged during gameplay"
    );
}

#[test]
fn test_trigger_continuation_runs_quantifier_and_detects_new_npc() {
    let world = Arc::new(WorldCard {
        key: "test".into(),
        name: "Test World".into(),
        description: "A test world".into(),
        ..Default::default()
    });

    let map = Arc::new(create_test_map());

    let player = Arc::new(chronicler_engine::domain::model::character::PersonaCard {
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

    let shopkeeper = NpcCard {
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
                narration_prompt: "The shopkeeper looks up with a smile.".into(),
            },
            repeat: false,
            room_id: None,
        }],
        relationships: vec![],
    };

    let gabriella = NpcCard {
        id: "gabriella".into(),
        sheet: CharacterSheet {
            name: "Gabriella".into(),
            description: "A mysterious woman".into(),
            personality: "Enigmatic".into(),
            scenario: "Appears from shadows".into(),
            example_dialogue: "Hello there.".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };

    let npcs = vec![shopkeeper, gabriella];
    let state = GameState::new(world, map, player, npcs, "room1".to_string());

    let data = TestData {
        world: Arc::clone(&state.world),
        map: Arc::clone(&state.map),
        persona: Arc::clone(&state.persona),
        npcs: state.npcs.values().cloned().collect(),
        room_npcs: vec!["gabriella".to_string()],
    };

    let app1 = SqliteTestAppBuilder::with_data(data.clone())
        .mock_backend(MockBackend::new)
        .build_service()
        .unwrap();

    add_input_and_save(&app1, "enter shop");

    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": ["gabriella"]}"#.to_string(),
        ]));

    let llm_backend = make_test_recorder(Arc::new(MockBackend::default().with_narrations(vec![
        "You step into the shop.".to_string(),
        "Gabriella emerges from the shadows behind the counter.".to_string(),
    ])));

    let app2 = SqliteTestAppBuilder::with_data(data)
        .game_service_fn(move |_storage| {
            Arc::new(GameService::with_mock_quantifier(
                llm_backend.clone(),
                quantifier.clone(),
            ))
        })
        .build_service()
        .unwrap();

    execute_action_impl(&app2, "enter shop".to_string());
    assert!(
        wait_for_generation_complete(&app2, 1000),
        "Execute should complete"
    );

    let guard = latest_state(&app2);

    let event_count = guard
        .narrative
        .history()
        .iter()
        .filter(|e| e.event_header.is_some())
        .count();
    assert_eq!(
        event_count, 1,
        "Trigger should have fired and added an Event"
    );

    let npc_ids_in_area: Vec<String> = guard
        .scene
        .npcs_in_area
        .iter()
        .map(|n| n.id.clone())
        .collect();
    assert!(
        npc_ids_in_area.contains(&"gabriella".to_string()),
        "Gabriella should appear in scene.npcs_in_area after post-trigger quantifier. Got: {npc_ids_in_area:?}"
    );

    let gabriella_state = guard
        .npc_encounter_log
        .npcs
        .get("gabriella")
        .expect("Gabriella should have character state");
    assert_eq!(
        gabriella_state.times_met, 1,
        "Gabriella's times_met should be 1 after Entered event"
    );
    assert!(
        gabriella_state.currently_meeting,
        "Gabriella should be currently_meeting"
    );
}
