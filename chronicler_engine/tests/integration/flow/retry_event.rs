use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::application::llm_recorder::LlmCallRecorder;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::application::ports::llm_message_repository::LlmMessageRepository;
use chronicler_engine::domain::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::domain::model::state::game_state::GameState;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::{
    ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::error::EngineError;
use chronicler_engine::test_support::make_test_context_with_sqlite;

use crate::pipeline_helpers::{
    add_input_and_save, create_test_state_with_trigger_npc, latest_state,
    wait_for_generation_complete,
};
use crate::fixtures::create_test_map;

fn make_test_recorder(provider: Arc<dyn LlmProvider>) -> Arc<LlmCallRecorder> {
    struct NoopForensics;
    impl LlmMessageRepository for NoopForensics {
        fn save_llm_message(
            &self,
            _: &chronicler_engine::application::ports::llm_message_repository::LlmMessage,
        ) -> Result<(), EngineError> {
            Ok(())
        }
        fn list_latest_llm_messages(
            &self,
            _: usize,
        ) -> Result<
            Vec<chronicler_engine::application::ports::llm_message_repository::LlmMessage>,
            EngineError,
        > {
            Ok(vec![])
        }
    }
    Arc::new(LlmCallRecorder::new(provider, Arc::new(NoopForensics)))
}

#[test]
fn test_event_retry_does_not_create_extra_swipe_on_narration() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "enter shop");

    let quantifier = make_test_recorder(Arc::new(MockBackend::default().with_prompt_responses(
        vec![
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ],
    )));

    let service = GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage))))),
        quantifier,
    );

    service.execute_action(ctx.clone(), "enter shop".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Execute should complete"
    );

    service.retry_last_response(ctx.clone());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Event retry should complete"
    );

    let guard = latest_state(&ctx);
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
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "enter shop");

    let quantifier = make_test_recorder(Arc::new(MockBackend::default().with_prompt_responses(
        vec![
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ],
    )));

    let service = GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage))))),
        quantifier,
    );

    service.execute_action(ctx.clone(), "enter shop".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Execute should complete"
    );
    let guard = latest_state(&ctx);
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

    service.retry_last_response(ctx.clone());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Event retry should complete"
    );
    let guard = latest_state(&ctx);
    assert_eq!(
        guard.movement.current_room_id, "room2",
        "Event retry: room should be unchanged (quantifier not rerun)"
    );

    let messages = ctx.storage.list_latest_llm_messages(50).unwrap();
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

    let player = Arc::new(chronicler_engine::domain::model::character::PlayerCard {
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
    let mut state = GameState::new(world, map, player, npcs, "room1".to_string());
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "enter shop");

    let llm_backend = make_test_recorder(Arc::new(MockBackend::default().with_narrations(vec![
        "You step into the shop.".to_string(),
        "Gabriella emerges from the shadows behind the counter.".to_string(),
    ])));

    let quantifier = make_test_recorder(Arc::new(MockBackend::default().with_prompt_responses(
        vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": ["gabriella"]}"#.to_string(),
        ],
    )));

    let service = GameService::with_mock_quantifier(llm_backend, quantifier);

    service.execute_action(ctx.clone(), "enter shop".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Execute should complete"
    );

    let guard = latest_state(&ctx);

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
