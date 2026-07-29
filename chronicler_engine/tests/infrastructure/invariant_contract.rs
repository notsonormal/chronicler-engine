//! Runtime invariant contract tests — fast regression guards.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

use chronicler_engine::application::action_pipeline::PhaseError;
use chronicler_engine::application::game_service::GameService;
use chronicler_engine::domain::model::state::game_state::{FreeActionContext, GameState};

use chronicler_engine::domain::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::domain::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcTransitionType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::{
    ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
};
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::application::GenerationGuard;
use chronicler_engine::application::utils::slot::GenerationSlot;
use chronicler_engine::test_support::make_test_recorder;

#[path = "../test_utils/mod.rs"]
mod test_utils;

#[path = "../helpers/application_ext.rs"]
mod application_ext;
#[path = "../helpers/fixtures.rs"]
mod fixtures;
#[path = "../helpers/sqlite_test_app_builder.rs"]
mod sqlite_test_app_builder;
#[path = "../helpers/storage_ext.rs"]
mod storage_ext;

use fixtures::create_minimal_test_state;
use fixtures::create_test_state;
use application_ext::PipelineHelpers;

fn shopkeeper_npc() -> NpcCard {
    NpcCard {
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
    }
}

fn npc_map(npcs: Vec<NpcCard>) -> HashMap<String, NpcCard> {
    npcs.into_iter().map(|npc| (npc.id.clone(), npc)).collect()
}

#[test]
fn test_inv001_generation_guard_resets_on_drop() {
    let flag = Arc::new(AtomicBool::new(true));
    let registry = Arc::new(RwLock::new(HashMap::from([(
        1u64,
        GenerationSlot::Generating { generation_id: 1 },
    )])));
    {
        let _guard = GenerationGuard::new(1, 1, Arc::clone(&registry), Arc::clone(&flag));
        assert!(
            flag.load(Ordering::SeqCst),
            "flag should be true while guard lives"
        );
    }
    assert!(
        !flag.load(Ordering::SeqCst),
        "INV-001: GenerationGuard did not reset is_generating on drop"
    );
}

#[test]
fn test_inv001_generation_guard_resets_on_panic() {
    let flag = Arc::new(AtomicBool::new(true));
    let flag_clone = Arc::clone(&flag);
    let registry = Arc::new(RwLock::new(HashMap::from([(
        1u64,
        GenerationSlot::Generating { generation_id: 1 },
    )])));
    let registry_clone = Arc::clone(&registry);

    let result = std::panic::catch_unwind(move || {
        let _guard = GenerationGuard::new(1, 1, registry_clone, flag_clone);
        panic!("intentional panic to test guard drop");
    });

    assert!(result.is_err(), "panic should have occurred");
    assert!(
        !flag.load(Ordering::SeqCst),
        "INV-001: GenerationGuard did not reset is_generating on panic"
    );
}
#[test]
fn test_inv002_state_mutation_order() {
    let mut state = create_minimal_test_state();
    let npc_id = "shopkeeper";

    state.add_message(
        "You look around the shop.".to_string(),
        None,
        MessageType::Narration,
    );

    assert_eq!(
        state.npc_encounter_log.get_times_met(npc_id),
        0,
        "pre-condition: times_met should be 0"
    );

    let quantifier = QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: vec![npc_id.to_string()],
            confidence: QuantifierConfidence::High,
        },
        movement: MovementParseResult::default(),
    };

    let map = Arc::new(fixtures::create_test_map());
    let player = Arc::new(fixtures::create_test_player());
    let npcs = npc_map(vec![shopkeeper_npc()]);

    let result = state
        .execute_freeaction_impl(
            &FreeActionContext {
                narration_text: "You look around the shop.",
                quantifier_result: &quantifier,
            },
            &map,
            &player,
            &npcs,
        )
        .expect("execute_freeaction_impl should succeed");
    assert!(
        result.trigger_match.is_some(),
        "INV-002: trigger should have fired (evaluated before times_met increment)"
    );
    assert_eq!(
        result.next_state.npc_encounter_log.get_times_met(npc_id),
        1,
        "INV-002: times_met should be 1 after NPC events are applied"
    );
    let history = &result.next_state.narrative.history;
    let narration_idx = history
        .iter()
        .position(|e| e.message_type == MessageType::Narration && e.text().contains("look around"))
        .expect("narration should be in history");
    assert!(
        narration_idx < history.len(),
        "INV-002: narration should be logged before trigger-related entries"
    );
}
#[test]
fn test_inv002_violation_demo() {
    let state = create_minimal_test_state();
    let npc_id = "shopkeeper";
    assert_eq!(
        state.npc_encounter_log.get_times_met(npc_id),
        0,
        "pre-condition: times_met should be 0"
    );
    let events = vec![NpcEvent {
        npc_id: npc_id.to_string(),
        event_type: NpcTransitionType::Entered,
    }];
    let map = Arc::new(fixtures::create_test_map());
    let npcs = npc_map(vec![shopkeeper_npc()]);
    let state_after_events = state
        .clone()
        .apply_npc_events(&events, &map, &npcs)
        .expect("apply_npc_events should succeed");
    let triggers_after_swap = state_after_events.evaluate_triggers(&npcs);
    assert!(
        triggers_after_swap.is_empty(),
        "VIOLATION: trigger should NOT fire when apply_npc_events runs first (times_met == 1)"
    );
    let triggers_correct = state.evaluate_triggers(&npcs);
    assert!(
        !triggers_correct.is_empty(),
        "Correct order: trigger SHOULD fire (times_met == 0)"
    );
}
#[test]
fn test_inv004_cancellable_at_boundaries() {
    use chronicler_engine::adapters::driven::llm::providers::MockBackend;

    let mock_backend_raw = Arc::new(MockBackend::default().with_delay(100));
    let backend_for_closure = Arc::clone(&mock_backend_raw);
    let (app, pg) = sqlite_test_app_builder::SqliteTestAppBuilder::default_test()
        .game_service_fn(move |_storage| {
            let recorder = make_test_recorder(backend_for_closure.clone());
            Arc::new(GameService::with_backends(
                recorder,
                AgentRegistry::default(),
            ))
        })
        .build_with_state()
        .unwrap();
    let started_for = app.current_game_id();
    let switched_to = started_for.wrapping_add(99);
    let pipeline = app.pipeline().clone();
    let state_for_thread = app.latest_state(&pg);

    let outcome = std::thread::scope(|s| {
        let handle = s.spawn(|| pipeline.run_from_input(state_for_thread, "look".to_string()));
        assert!(
            test_utils::wait::wait_for_condition_sync(
                std::time::Duration::from_secs(5),
                std::time::Duration::from_millis(50),
                || {
                    mock_backend_raw
                        .narration_started
                        .load(std::sync::atomic::Ordering::SeqCst)
                }
            ),
            "narration should start within timeout"
        );
        pg.set_game_id(switched_to);

        handle.join().expect("pipeline thread should not panic")
    });

    assert!(
        matches!(outcome, Err(PhaseError::Cancelled)),
        "INV-004: pipeline should return Cancelled on game_id mismatch at boundary, got {outcome:?}"
    );

    let final_state = app.latest_state(&pg);
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "INV-004: status should be Idle after cancellation"
    );
}

#[test]
fn test_inv004b_no_concurrent_async_actions() {
    let flag = Arc::new(AtomicBool::new(false));

    let first = flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    assert!(first.is_ok(), "first compare_exchange should succeed");

    let second = flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    assert!(
        second.is_err(),
        "INV-004b: second compare_exchange should fail (concurrent action rejected)"
    );

    {
        let registry = Arc::new(RwLock::new(HashMap::from([(
            1u64,
            GenerationSlot::Generating { generation_id: 1 },
        )])));
        let _guard = GenerationGuard::new(1, 1, registry, Arc::clone(&flag));
        assert!(flag.load(Ordering::SeqCst));
    }

    assert!(
        !flag.load(Ordering::SeqCst),
        "flag should be reset after guard drop"
    );
    let third = flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    assert!(
        third.is_ok(),
        "INV-004b: third compare_exchange should succeed after guard drop"
    );
}
#[test]
fn test_inv003_snapshot_captures_state_fields() {
    use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;

    let mut state = create_test_state();
    state.movement.current_room_id = "room2".to_string();
    state.scene.npcs_in_area.clear();
    state.narrative.last_trigger = None;

    let snapshot = GameStateSnapshot::from_game_state(&state);
    assert_eq!(
        snapshot.movement.current_room_id, "room2",
        "INV-003: snapshot should capture modified current_room_id"
    );
    assert_eq!(
        snapshot.scene.npcs_in_area.len(),
        0,
        "INV-003: snapshot should capture empty npcs_in_area"
    );
    assert_eq!(
        snapshot.narrative.last_trigger, None,
        "INV-003: snapshot should capture None last_trigger"
    );
}
#[test]
fn test_inv005_handle_movement_runs_before_narration() {
    use chronicler_engine::domain::model::quantifier::{
        QuantifierConfidence, QuantifierParseResult, QuantifierResult,
    };
    use std::sync::Arc;

    let map = Arc::new(fixtures::create_test_map());
    let player = Arc::new(fixtures::create_test_player());
    let npcs = HashMap::new();
    let state = GameState::new("room1".to_string());
    let original_room = state.movement.current_room_id.clone();
    let target_room = "room2";

    let quantifier = QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: vec![],
            confidence: QuantifierConfidence::High,
        },
        movement: MovementParseResult {
            movement_type: Some(MovementType::Leaving),
            destination: Some(target_room.to_string()),
            confidence: QuantifierConfidence::High,
        },
    };

    let result = state
        .execute_freeaction_impl(
            &FreeActionContext {
                narration_text: "I walk north.",
                quantifier_result: &quantifier,
            },
            &map,
            &player,
            &npcs,
        )
        .expect("execute_freeaction_impl should succeed");

    assert_eq!(
        result.next_state.movement.current_room_id, target_room,
        "INV-005: current_room_id should be updated by handle_movement before narration is logged"
    );
    assert_ne!(
        result.next_state.movement.current_room_id, original_room,
        "INV-005: handle_movement should have changed the room"
    );
}
#[test]
fn test_inv007_dynamic_room_creation_on_invalid_destination() {
    let state = create_test_state();
    let invalid_destination = "nonexistent_place_xyz";

    let map = Arc::new(fixtures::create_test_map());
    let npcs = HashMap::new();

    assert!(
        !map.overworld
            .regions
            .iter()
            .flat_map(|r| r.rooms.iter())
            .any(|r| r.id == invalid_destination),
        "precondition: invalid_destination should not exist in map"
    );

    let result = state
        .handle_movement(Some(invalid_destination), &[], &map, &npcs)
        .unwrap();
    assert!(
        !result.movement.dynamic_rooms.is_empty(),
        "INV-007: dynamic room should be created for invalid destination"
    );
    assert!(
        result.movement.current_room_id.starts_with("dynamic_"),
        "INV-007: current_room_id should be set to a dynamic room"
    );
    let history = result.narrative.history();
    let system_messages: Vec<_> = history
        .iter()
        .filter(|m| m.message_type == MessageType::System)
        .collect();
    assert!(
        !system_messages.is_empty(),
        "INV-007: system message should be logged for dynamic room creation"
    );
    assert!(
        system_messages
            .iter()
            .any(|m| m.text.contains("Entered unknown location")),
        "INV-007: system message should mention 'Entered unknown location'"
    );
}
#[test]
fn test_inv002_mutation_order_property() {
    use proptest::prelude::*;

    proptest!(|(narration_text in r"[^\s]{10,100}", has_npc in any::<bool>())| {
        let mut state = create_minimal_test_state();
        let npc_id = "shopkeeper";
        state.add_message(narration_text.clone(), None, MessageType::Narration);
        let npc_ids = if has_npc { vec![npc_id.to_string()] } else { vec![] };

        let quantifier = QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids,
                confidence: QuantifierConfidence::High,
            },
            movement: MovementParseResult::default(),
        };

        let map = Arc::new(fixtures::create_test_map());
        let player = Arc::new(fixtures::create_test_player());
        let npcs = npc_map(vec![shopkeeper_npc()]);
        let result = state.execute_freeaction_impl(
            &FreeActionContext {
                narration_text: &narration_text,
                quantifier_result: &quantifier,
            },
            &map,
            &player,
            &npcs,
        ).expect("execute_freeaction_impl should succeed");
        let history = &result.next_state.narrative.history;
        let search_len = 20.min(narration_text.chars().count());
        let search_text: String = narration_text.chars().take(search_len).collect();
        let has_narration = history.iter().any(|e| {
            e.message_type == MessageType::Narration && e.text().contains(&search_text)
        });
        prop_assert!(has_narration, "narration should be in history");
        if has_npc {
            prop_assert!(result.trigger_match.is_some(), "trigger should fire when NPC appears");
            prop_assert_eq!(
                result.next_state.npc_encounter_log.get_times_met(npc_id),
                1,
                "times_met should be 1 after apply_npc_events"
            );
        }
    });
}

#[tokio::test]
async fn test_p4_concurrent_happy_path() {
    use chronicler_engine::adapters::driven::llm::providers::MockBackend;
    use chronicler_engine::application::agents::registry::AgentRegistry;
    use chronicler_engine::application::errors::ProcessActionResult;
    use chronicler_engine::application::game_service::GameService;

    let mock_backend_raw = Arc::new(
        MockBackend::default()
            .with_delay(300)
            .with_narrations(vec!["GEN_A_OUTPUT".to_string(), "GEN_B_OUTPUT".to_string()]),
    );
    let backend_for_closure = Arc::clone(&mock_backend_raw);
    let (app, pg) = sqlite_test_app_builder::SqliteTestAppBuilder::default_test()
        .game_service_fn(move |_storage| {
            let recorder = make_test_recorder(backend_for_closure.clone());
            Arc::new(GameService::with_backends(
                recorder,
                AgentRegistry::default(),
            ))
        })
        .build_with_state()
        .unwrap();
    let game1 = app.current_game_id();

    let result_a = app
        .process_action("look".to_string())
        .expect("process_action should not error");
    assert!(
        matches!(result_a, ProcessActionResult::Started),
        "gen A claim should return Started, got {result_a:?}"
    );

    assert!(
        test_utils::wait::wait_for_condition_sync(
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(50),
            || mock_backend_raw.narration_started.load(Ordering::SeqCst),
        ),
        "gen A's narration call should start within timeout"
    );

    let game2 = app
        .create_game("test", "test_player")
        .expect("create_game(game2) should succeed");
    assert_ne!(game2, game1, "reset must produce a distinct game id");

    assert!(
        test_utils::wait::wait_for_condition_sync(
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(50),
            || !app.is_generating_now(),
        ),
        "gen A's pipeline must complete (slot released) within timeout"
    );

    let state_after_a = app.latest_state(&pg);
    let a_present = state_after_a
        .narrative
        .history
        .iter()
        .any(|e| e.text().contains("GEN_A_OUTPUT"));
    assert!(
        !a_present,
        "game 2 state must NOT contain gen A's narration; history: {:?}",
        state_after_a
            .narrative
            .history
            .iter()
            .map(|e| e.text().to_string())
            .collect::<Vec<_>>()
    );

    let result_b = app
        .process_action("go north".to_string())
        .expect("process_action should not error");
    assert!(
        matches!(result_b, ProcessActionResult::Started),
        "gen B claim must succeed after gen A aborts (slot available), got {result_b:?}"
    );

    assert!(
        test_utils::wait::wait_for_condition_sync(
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(50),
            || !app.is_generating_now(),
        ),
        "gen B's pipeline must complete within timeout"
    );

    let state_after_b = app.latest_state(&pg);
    let b_present = state_after_b
        .narrative
        .history
        .iter()
        .any(|e| e.text().contains("GEN_B_OUTPUT"));
    let a_still_absent = !state_after_b
        .narrative
        .history
        .iter()
        .any(|e| e.text().contains("GEN_A_OUTPUT"));
    assert!(
        b_present,
        "game 2 state MUST contain gen B's narration; history: {:?}",
        state_after_b
            .narrative
            .history
            .iter()
            .map(|e| e.text().to_string())
            .collect::<Vec<_>>()
    );
    assert!(
        a_still_absent,
        "gen A's narration must remain absent from game 2 after gen B completes"
    );

    assert!(
        !app.is_generating_now(),
        "is_generating projection must be false after both pipelines complete (registry clean)"
    );

    app.cancel_token().cancel();
}

#[tokio::test]
async fn test_p4_concurrent_triple_overlap() {
    use chronicler_engine::adapters::driven::llm::providers::MockBackend;
    use chronicler_engine::application::agents::registry::AgentRegistry;
    use chronicler_engine::application::errors::ProcessActionResult;
    use chronicler_engine::application::game_service::GameService;

    let mock_backend_raw = Arc::new(MockBackend::default().with_delay(300).with_narrations(vec![
        "GEN_A_OUTPUT".to_string(),
        "GEN_B_OUTPUT".to_string(),
        "GEN_C_OUTPUT".to_string(),
    ]));
    let backend_for_closure = Arc::clone(&mock_backend_raw);
    let (app, pg) = sqlite_test_app_builder::SqliteTestAppBuilder::default_test()
        .game_service_fn(move |_storage| {
            let recorder = make_test_recorder(backend_for_closure.clone());
            Arc::new(GameService::with_backends(
                recorder,
                AgentRegistry::default(),
            ))
        })
        .build_with_state()
        .unwrap();
    let game1 = app.current_game_id();

    let result_a = app
        .process_action("look".to_string())
        .expect("process_action should not error");
    assert!(
        matches!(result_a, ProcessActionResult::Started),
        "gen A claim should return Started, got {result_a:?}"
    );

    assert!(
        test_utils::wait::wait_for_condition_sync(
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(50),
            || mock_backend_raw.narration_started.load(Ordering::SeqCst),
        ),
        "gen A's narration call should start within timeout"
    );

    let game2 = app
        .create_game("test", "test_player")
        .expect("create_game(game2) should succeed");
    assert_ne!(game2, game1, "reset must produce a distinct game id");

    let result_b = app
        .process_action("go north".to_string())
        .expect("process_action should not error");
    assert!(
        matches!(result_b, ProcessActionResult::Started),
        "gen B claim should succeed for game 2, got {result_b:?}"
    );
    std::thread::sleep(std::time::Duration::from_millis(100));

    let game3 = app
        .create_game("test", "test_player")
        .expect("create_game(game3) should succeed");
    assert_ne!(game3, game2, "second reset must produce a distinct game id");

    let result_c = app
        .process_action("look around".to_string())
        .expect("process_action should not error");
    assert!(
        matches!(result_c, ProcessActionResult::Started),
        "gen C claim should succeed for game 3, got {result_c:?}"
    );
    std::thread::sleep(std::time::Duration::from_millis(100));

    assert!(
        test_utils::wait::wait_for_condition_sync(
            std::time::Duration::from_secs(10),
            std::time::Duration::from_millis(100),
            || !app.is_generating_now(),
        ),
        "all three pipelines must complete within timeout"
    );

    assert_eq!(
        app.current_game_id(),
        game3,
        "active game must be game 3 at end of test"
    );

    let state3 = app.latest_state(&pg);
    let history_texts: Vec<String> = state3
        .narrative
        .history
        .iter()
        .map(|e| e.text().to_string())
        .collect();
    let has_a = history_texts.iter().any(|t| t.contains("GEN_A_OUTPUT"));
    let has_b = history_texts.iter().any(|t| t.contains("GEN_B_OUTPUT"));
    let has_c = history_texts.iter().any(|t| t.contains("GEN_C_OUTPUT"));

    assert!(
        !has_a,
        "game 3 state must NOT contain gen A's narration; history: {history_texts:?}"
    );
    assert!(
        !has_b,
        "game 3 state must NOT contain gen B's narration; history: {history_texts:?}"
    );
    assert!(
        has_c,
        "game 3 state MUST contain gen C's narration; history: {history_texts:?}"
    );

    assert!(
        !app.is_generating_now(),
        "is_generating projection must be false (no stale slots for games 1/2/3)"
    );

    let result_after = app
        .process_action("examine".to_string())
        .expect("process_action should not error after all gens complete");
    assert!(
        matches!(result_after, ProcessActionResult::Started),
        "fresh claim after triple-overlap must succeed (slot registry clean), got {result_after:?}"
    );

    app.cancel_token().cancel();
}
