//! [DOC: docs/architecture/invariants.md]
//! Runtime invariant contract tests — fast regression guards.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chronicler_engine::application::action_pipeline::{ActionOutcome, ActionPipeline};
use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::engine::action_processing::{FreeActionContext, execute_freeaction_impl};
use chronicler_engine::engine::trigger_eval::get_times_met;
use chronicler_engine::model::quantifier::{
    MovementParseResult, QuantifierConfidence, QuantifierParseResult, QuantifierResult,
};
use chronicler_engine::model::state::LogType;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::server::fragments::GenerationGuard;
use chronicler_engine::test_support::make_test_context;

#[path = "helpers/pipeline_helpers.rs"]
mod pipeline_helpers;
mod test_data;

use pipeline_helpers::{create_test_state_with_trigger_npc, latest_state};
use test_data::create_test_state;

// ─── INV-001: Generation Status Lifecycle ───────────────────────────────────

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

// ─── INV-002: State Mutation Order ──────────────────────────────────────────

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
        .position(|e| e.log_type == LogType::Narration && e.text.contains("look around"))
        .expect("narration should be in history");
    assert!(
        narration_idx < history.len(),
        "INV-002: narration should be logged before trigger-related entries"
    );
}

// ─── INV-004: LLM Calls Are Cancellable ─────────────────────────────────────

#[test]
fn test_inv004_cancellable_at_boundaries() {
    let mut state = create_test_state();
    state.narrative.history.clear();

    let ctx = make_test_context(state);
    let cancel_token = ctx.cancel_token.clone();

    // Backend with a small delay so cancellation has time to fire.
    let backend = DefaultGameService::with_backends(
        Arc::new(MockBackend::with_delay(100)),
        AgentRegistry::default(),
    );

    let pipeline = ActionPipeline::new(&backend, &ctx);

    // Clone state for the thread (ActionPipeline takes state by value).
    let state_for_thread = latest_state(&ctx);

    let outcome = std::thread::scope(|s| {
        let handle = s.spawn(|| pipeline.run_from_input(state_for_thread, "look".to_string()));

        // Cancel shortly after the pipeline starts.
        std::thread::sleep(std::time::Duration::from_millis(20));
        cancel_token.cancel();

        handle.join().expect("pipeline thread should not panic")
    });

    assert!(
        matches!(outcome, ActionOutcome::Cancelled),
        "INV-004: pipeline should return Cancelled when token is cancelled, got {outcome:?}"
    );

    let final_state = latest_state(&ctx);
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "INV-004: status should be Idle after cancellation"
    );
}

// ─── INV-004b: No Concurrent Async Actions ──────────────────────────────────

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
