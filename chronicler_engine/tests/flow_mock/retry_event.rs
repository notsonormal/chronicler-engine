use std::sync::Arc;

use chronicler_engine::engine::game_service::{DefaultGameService, GameService};
use chronicler_engine::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::model::state::GameState;
use chronicler_engine::model::trigger::{
    ComparisonOperator, Trigger, TriggerAction, TriggerCondition,
};
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::narrative::agents::quantifier::{
    MockQuantifierBackend, MovementParseResult, MovementType, QuantifierConfidence,
};
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::make_test_context_with_sqlite;

use crate::test_data::create_test_map;
use crate::{
    add_input_and_save, create_test_state_with_trigger_npc, latest_state,
    wait_for_generation_complete,
};

#[test]
fn test_retry_event_continuation_preserves_quantifier_result() {
    // Setup: Action fires trigger → event continuation.
    // Mock quantifier returns movement on first (and only) call.
    // Flow: Execute → player moves → event added
    //       → Retry event → player STILL in same room (quantifier NOT rerun)
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.turns.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "enter shop");

    let quantifier = Arc::new(MockQuantifierBackend {
        per_call_movements: vec![Some(MovementParseResult {
            movement_type: Some(MovementType::Entering),
            destination: Some("room2".to_string()),
            confidence: QuantifierConfidence::High,
        })],
        ..Default::default()
    });

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.llm_message_storage)))),
        quantifier,
    );

    // Execute: quantifier runs, player moves, trigger fires
    service.execute_action(ctx.clone(), "enter shop".to_string(), "Player".to_string());
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

    // Retry event continuation: room should stay the same because quantifier is NOT rerun
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

    // Verify LLM calls were logged to SQLite storage
    let messages = ctx.llm_message_storage.list_latest(50).unwrap();
    assert!(
        !messages.is_empty(),
        "LLM messages should be logged during gameplay"
    );
}

#[test]
fn test_trigger_continuation_runs_quantifier_and_detects_new_npc() {
    // Setup: Shopkeeper has trigger (times_met == 0). Gabriella is present in the world
    // but NOT detected by first quantifier.
    // Flow: Execute → first quantifier returns empty → trigger fires
    //       → trigger continuation mentions Gabriella → second quantifier detects her
    //       → Gabriella appears in scene.npcs_in_area and times_met increments.
    let world = Arc::new(WorldCard {
        name: "Test World".into(),
        description: "A test world".into(),
        global_rules: vec![],
        starting_room_id: "room1".into(),
        scenarios: vec![],
        default_room_image: None,
    });

    let map = Arc::new(create_test_map());

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
            condition: TriggerCondition::TimesMet(ComparisonOperator::Eq, 0),
            action: TriggerAction {
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
    state.narrative.turns.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "enter shop");

    // Mock LLM: first call = main narration, second call = trigger continuation mentioning Gabriella
    let llm_backend = Arc::new(MockBackend {
        per_call_narrations: vec![
            "You step into the shop.".to_string(),
            "Gabriella emerges from the shadows behind the counter.".to_string(),
        ],
        ..Default::default()
    });

    // Mock quantifier: first call = no NPCs, second call = Gabriella detected
    let quantifier = Arc::new(MockQuantifierBackend {
        per_call_npcs: vec![
            vec![],                        // first call: after main narration
            vec!["gabriella".to_string()], // second call: after trigger continuation
        ],
        ..Default::default()
    });

    let service = DefaultGameService::with_mock_quantifier(llm_backend, quantifier);

    service.execute_action(ctx.clone(), "enter shop".to_string(), "Player".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Execute should complete"
    );

    let guard = latest_state(&ctx);

    // Verify trigger fired
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

    // Verify Gabriella was detected and added to scene.npcs_in_area
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

    // Verify times_met was incremented for Gabriella via Entered event
    let gabriella_state = guard
        .character_state
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
