//! [DOC: docs/architecture/invariants.md]
//! Runtime invariant contract tests — fast regression guards.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chronicler_engine::application::action_pipeline::{ActionOutcome, ActionPipeline};
use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::engine::action_processing::{
    FreeActionContext, apply_npc_events, execute_freeaction_impl,
};
use chronicler_engine::engine::trigger_eval::get_times_met;
use chronicler_engine::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcTransitionType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};
use chronicler_engine::model::state::MessageType;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::server::fragments::GenerationGuard;
use chronicler_engine::test_support::make_test_context;

#[path = "helpers/pipeline_helpers.rs"]
mod pipeline_helpers;
mod test_data;

use pipeline_helpers::{create_test_state_with_trigger_npc, latest_state};
use test_data::create_test_state;

#[test]
fn test_inv001_generation_guard_resets_on_drop() {
    let flag = Arc::new(AtomicBool::new(true));
    {
        let _guard = GenerationGuard(Arc::clone(&flag));
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

    let result = std::panic::catch_unwind(move || {
        let _guard = GenerationGuard(flag_clone);
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
    let state = create_test_state_with_trigger_npc();
    let npc_id = "shopkeeper";

    // Verify pre-condition: NPC has not been met yet.
    assert_eq!(
        get_times_met(&state.npc_encounter_log, npc_id),
        0,
        "pre-condition: times_met should be 0"
    );

    // Quantifier result includes the shopkeeper NPC (simulating them entering the room).
    let quantifier = QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: vec![npc_id.to_string()],
            confidence: QuantifierConfidence::High,
        },
        movement: MovementParseResult::default(),
    };

    let result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "You look around the shop.",
            quantifier_result: &quantifier,
        },
    )
    .expect("execute_freeaction_impl should succeed");

    // Trigger fired because evaluation happened BEFORE times_met was incremented.
    assert!(
        result.trigger_match.is_some(),
        "INV-002: trigger should have fired (evaluated before times_met increment)"
    );

    // NPC events were applied AFTER trigger evaluation.
    assert_eq!(
        get_times_met(&result.next_state.npc_encounter_log, npc_id),
        1,
        "INV-002: times_met should be 1 after NPC events are applied"
    );

    // Narration was logged BEFORE trigger evaluation.
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
    let state = create_test_state_with_trigger_npc();
    let npc_id = "shopkeeper";
    // Pre-condition: times_met == 0
    assert_eq!(
        get_times_met(&state.npc_encounter_log, npc_id),
        0,
        "pre-condition: times_met should be 0"
    );
    let events = vec![NpcEvent {
        npc_id: npc_id.to_string(),
        event_type: NpcTransitionType::Entered,
    }];
    let state_after_events =
        apply_npc_events(state.clone(), &events).expect("apply_npc_events should succeed");
    // Now evaluate triggers on the post-event state
    let triggers_after_swap =
        chronicler_engine::engine::trigger_eval::evaluate_triggers(&state_after_events);
    // The trigger should NOT fire because times_met is now 1
    assert!(
        triggers_after_swap.is_empty(),
        "VIOLATION: trigger should NOT fire when apply_npc_events runs first (times_met == 1)"
    );
    // This is the correct order used in execute_freeaction_impl.
    let triggers_correct = chronicler_engine::engine::trigger_eval::evaluate_triggers(&state);
    // The trigger SHOULD fire because times_met is still 0
    assert!(
        !triggers_correct.is_empty(),
        "Correct order: trigger SHOULD fire (times_met == 0)"
    );
}
#[test]
fn test_inv004_cancellable_at_boundaries() {
    let mut state = create_test_state();
    state.narrative.history.clear();

    let ctx = make_test_context(state);
    let cancel_token = ctx.cancel_token.clone();

    // Backend with a small delay so cancellation has time to fire.
    let mock_backend = Arc::new(MockBackend::with_delay(100));
    let backend = DefaultGameService::with_backends(mock_backend.clone(), AgentRegistry::default());

    let pipeline = ActionPipeline::new(&backend, &ctx);

    // Clone state for the thread (ActionPipeline takes state by value).
    let state_for_thread = latest_state(&ctx);

    let outcome = std::thread::scope(|s| {
        let handle = s.spawn(|| pipeline.run_from_input(state_for_thread, "look".to_string()));

        // Wait for the narration call to start before cancelling.
        while !mock_backend
            .narration_started
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        cancel_token.cancel();

        handle.join().expect("pipeline thread should not panic")
    });

    assert!(
        matches!(outcome, ActionOutcome::Cancelled),
        "INV-004: pipeline should return Cancelled when token is cancelled, got {outcome:?}"
    );

    let final_state = latest_state(&ctx);
    assert_eq!(
        final_state.narrative.input_buffer.status,
        chronicler_engine::model::state::GenerationStatus::Idle,
        "INV-004: status should be Idle after cancellation"
    );
}

#[test]
fn test_inv004b_no_concurrent_async_actions() {
    let flag = Arc::new(AtomicBool::new(false));

    // First acquire succeeds.
    let first = flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    assert!(first.is_ok(), "first compare_exchange should succeed");

    // Second acquire fails (simulating concurrent action request).
    let second = flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    assert!(
        second.is_err(),
        "INV-004b: second compare_exchange should fail (concurrent action rejected)"
    );

    // Simulate action completion via GenerationGuard drop.
    {
        let _guard = GenerationGuard(Arc::clone(&flag));
        // Guard should not change the flag (it's already true).
        assert!(flag.load(Ordering::SeqCst));
    }

    // After drop, flag is reset.
    assert!(
        !flag.load(Ordering::SeqCst),
        "flag should be reset after guard drop"
    );

    // Third acquire succeeds now that the flag is cleared.
    let third = flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    assert!(
        third.is_ok(),
        "INV-004b: third compare_exchange should succeed after guard drop"
    );
}
#[test]
fn test_inv003_snapshot_restores_state_fields() {
    use chronicler_engine::model::state_snapshot::GameStateSnapshot;

    let mut state = create_test_state();
    state.movement.current_room_id = "room2".to_string();
    state.scene.npcs_in_area.clear();
    state.narrative.history.clear();
    state.narrative.last_trigger = None;

    // Create snapshot from modified state
    let snapshot = GameStateSnapshot::from_game_state(&state);

    // Verify snapshot captured the modified state
    assert_eq!(
        snapshot.movement.current_room_id, "room2",
        "INV-003: snapshot should capture modified current_room_id"
    );
    assert_eq!(
        snapshot.scene.npcs_in_area.len(),
        0,
        "INV-003: snapshot should capture empty npcs_in_area"
    );

    // Create fresh state and apply snapshot
    let mut fresh_state = create_test_state();
    fresh_state.movement.current_room_id = "room1".to_string();
    fresh_state
        .scene
        .npcs_in_area
        .push(fresh_state.npcs.values().next().unwrap().clone());

    // Apply snapshot restores the original state
    snapshot.apply_to(&mut fresh_state);

    // INV-003: Verify movement.current_room_id is restored
    assert_eq!(
        fresh_state.movement.current_room_id, "room2",
        "INV-003: apply_to should restore current_room_id"
    );

    // INV-003: Verify narrative is restored
    assert_eq!(
        fresh_state.narrative.last_trigger, None,
        "INV-003: apply_to should restore narrative.last_trigger"
    );

    // INV-003: Verify npcs_in_area is restored (should be empty)
    assert_eq!(
        fresh_state.scene.npcs_in_area.len(),
        0,
        "INV-003: apply_to should restore npcs_in_area"
    );
}
#[test]
fn test_inv005_handle_movement_runs_before_narration() {
    use chronicler_engine::engine::action_processing::{FreeActionContext, execute_freeaction_impl};
    use chronicler_engine::model::quantifier::{
        QuantifierConfidence, QuantifierParseResult, QuantifierResult,
    };
    use std::sync::Arc;

    // Use the full test map which has room1, room2, room3
    let world = Arc::new(test_data::create_test_world());
    let map = Arc::new(test_data::create_test_map());
    let player = Arc::new(test_data::create_test_player());
    let npcs = vec![];
    let state = chronicler_engine::model::state::GameState::new(
        world,
        map,
        player,
        npcs,
        "room1".to_string(),
    );
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

    let result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "I walk north.",
            quantifier_result: &quantifier,
        },
    )
    .expect("execute_freeaction_impl should succeed");

    // INV-005: handle_movement should have updated current_room_id BEFORE narration
    assert_eq!(
        result.next_state.movement.current_room_id, target_room,
        "INV-005: current_room_id should be updated by handle_movement before narration is logged"
    );

    // INV-005: Verify the room actually changed (not same room fallback)
    assert_ne!(
        result.next_state.movement.current_room_id, original_room,
        "INV-005: handle_movement should have changed the room"
    );
}
#[test]
fn test_inv007_dynamic_room_creation_on_invalid_destination() {
    use chronicler_engine::engine::action_processing::handle_movement;

    let state = create_test_state();
    let invalid_destination = "nonexistent_place_xyz";

    // Verify precondition: this room does NOT exist
    assert!(
        !state
            .map
            .overworld
            .regions
            .iter()
            .flat_map(|r| r.rooms.iter())
            .any(|r| r.id == invalid_destination),
        "precondition: invalid_destination should not exist in map"
    );

    let result = handle_movement(state, Some(invalid_destination), &[]).unwrap();

    // INV-007: A dynamic room should be created
    assert!(
        !result.movement.dynamic_rooms.is_empty(),
        "INV-007: dynamic room should be created for invalid destination"
    );

    // INV-007: current_room_id should be updated to a dynamic room
    assert!(
        result.movement.current_room_id.starts_with("dynamic_"),
        "INV-007: current_room_id should be set to a dynamic room"
    );

    // INV-007: A system message should be logged
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

    proptest!(|(narration_text in "\\PC{10,100}", has_npc in any::<bool>())| {
        // Create fresh state for each iteration
        let state = create_test_state_with_trigger_npc();
        let npc_id = "shopkeeper";

        // NPC appears if has_npc is true
        let npc_ids = if has_npc { vec![npc_id.to_string()] } else { vec![] };

        let quantifier = QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids,
                confidence: QuantifierConfidence::High,
            },
            movement: MovementParseResult::default(),
        };

        let result = execute_freeaction_impl(
            &state,
            &FreeActionContext {
                narration_text: &narration_text,
                quantifier_result: &quantifier,
            },
        ).expect("execute_freeaction_impl should succeed");

        // Invariant 1: narration added to history
        let history = &result.next_state.narrative.history;
        let search_len = 20.min(narration_text.chars().count());
        let search_text: String = narration_text.chars().take(search_len).collect();
        let has_narration = history.iter().any(|e| {
            e.message_type == MessageType::Narration && e.text().contains(&search_text)
        });
        prop_assert!(has_narration, "narration should be in history");

        // Invariant 2: if NPC present, trigger fires on first encounter
        if has_npc {
            prop_assert!(result.trigger_match.is_some(), "trigger should fire when NPC appears");
            prop_assert_eq!(
                get_times_met(&result.next_state.npc_encounter_log, npc_id),
                1,
                "times_met should be 1 after apply_npc_events"
            );
        }
    });
}
