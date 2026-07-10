//! [DOC: docs/adr/adr-030-is-generating-invariant.md]
//! Tests for ADR-030: `is_generating` AtomicBool must agree with persisted `GenerationStatus`.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tokio::time::sleep;

use crate::application::application_service::DefaultApplicationService;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::test_support::make_test_app_with_sqlite;

fn cached_flag(app: &DefaultApplicationService) -> bool {
    app.is_generating().load(Ordering::SeqCst)
}

fn persisted_flag(app: &DefaultApplicationService) -> bool {
    app.storage()
        .load_latest_snapshot()
        .ok()
        .flatten()
        .map(|_snap| {
            app.load_or_fresh()
                .narrative
                .input_buffer
                .status
                .is_generating()
        })
        .unwrap_or(false)
}

fn invariant_holds(app: &DefaultApplicationService) -> bool {
    cached_flag(app) == persisted_flag(app)
}

async fn wait_until_idle(app: &DefaultApplicationService, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let cached = cached_flag(app);
        let persisted = persisted_flag(app);

        if !cached && persisted {
            panic!(
                "invariant violation during wait_until_idle: cached=false persisted=true. \
                 expected (cached=true, persisted=Idle) as the only allowed transient."
            );
        }

        if cached && !persisted {
            sleep(Duration::from_millis(50)).await;
            continue;
        }

        if !cached && !persisted {
            assert!(invariant_holds(app), "both idle but invariant violated");
            return true;
        }

        sleep(Duration::from_millis(50)).await;
    }
    false
}

#[test]
fn test_is_generating_invariant_helper_detects_divergence() {
    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let app = make_test_app_with_sqlite(state).expect("make_test_app_with_sqlite");

    assert!(
        invariant_holds(&app),
        "Invariant should hold initially: cached={} persisted={}",
        cached_flag(&app),
        persisted_flag(&app)
    );

    app.is_generating().store(true, Ordering::SeqCst);

    assert!(
        !invariant_holds(&app),
        "Invariant helper must detect divergence: cached=true persisted=false"
    );
    assert!(cached_flag(&app), "AtomicBool forced to true");
    assert!(
        !persisted_flag(&app),
        "Persisted status should still report Idle"
    );
}

#[tokio::test(flavor = "current_thread")]
#[should_panic(expected = "invariant violation during wait_until_idle")]
async fn test_wait_until_idle_fails_fast_on_cached_false_persisted_generating() {
    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let app =
        std::sync::Arc::new(make_test_app_with_sqlite(state).expect("make_test_app_with_sqlite"));

    // Bypass process_action: production CAS would forbid cached=false with status=Generating.
    let mut gs = app.load_or_fresh();
    gs.narrative.input_buffer.status = GenerationStatus::Generating;
    let snapshot_id = app
        .save_state(&gs)
        .expect("save_state should persist Generating");
    app.is_generating().store(false, Ordering::SeqCst);
    let _ = snapshot_id;

    let _ = wait_until_idle(&app, Duration::from_secs(2)).await;
}

// P4 regression: `is_generating` must stay `true` while any registry slot is
// `Generating`. Asserts post-fix lock-order holds across an interleaved
// release + claim cycle.
#[tokio::test]
async fn test_projection_invariant_under_interleaved_release() {
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use crate::adapters::driven::llm::providers::MockBackend;
    use crate::application::agents::registry::AgentRegistry;
    use crate::application::errors::ProcessActionResult;
    use crate::application::game_service::GameService;
    use crate::test_support::{make_test_app_with_game_service, make_test_recorder};

    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;

    // 300ms LLM delay keeps A in-flight across the reset + drop window.
    // After reset, A's pipeline hits the α-check at the next phase
    // boundary, sees the new game id, and aborts via `ActionOutcome::Cancelled`.
    // Two narrations queued: A (discarded by α-check) + B (persisted).
    let mock_backend_raw = std::sync::Arc::new(
        MockBackend::default()
            .with_delay(300)
            .with_narrations(vec!["GEN_A_OUTPUT".to_string(), "GEN_B_OUTPUT".to_string()]),
    );
    let app = make_test_app_with_game_service(state, |_storage| {
        let recorder = make_test_recorder(mock_backend_raw.clone());
        std::sync::Arc::new(GameService::with_backends(
            recorder,
            AgentRegistry::default(),
        ))
    })
    .expect("make_test_app_with_game_service");

    let game_a_id = app.current_game_id();

    // Claim A on the initial game.
    let result_a = app
        .process_action("look".to_string())
        .expect("process_action A should not error");
    assert!(
        matches!(result_a, ProcessActionResult::Started),
        "gen A claim should return Started, got {result_a:?}"
    );
    assert!(
        app.is_generating().load(Ordering::SeqCst),
        "projection must be true after A claim"
    );

    // Wait for A's narration call to begin — α-check that aborts A runs
    // AFTER this call returns.
    let narration_started =
        wait_for_condition(Duration::from_secs(5), Duration::from_millis(25), || {
            mock_backend_raw.narration_started.load(Ordering::SeqCst)
        })
        .await;
    assert!(
        narration_started,
        "gen A's narration call should start within timeout"
    );

    // Reset → game B; A's pipeline will abort at the next phase boundary
    // when the α-check sees the game id mismatch.
    let game_b_id = app
        .create_game("test", "hero")
        .expect("create_game(B) should succeed");
    assert_ne!(game_b_id, game_a_id, "reset must produce distinct game id");

    // Claim B while A's LLM call is still in flight — this is the race window
    // that exercises the post-fix lock-order invariant.
    let result_b = app
        .process_action("go north".to_string())
        .expect("process_action B should not error");
    assert!(
        matches!(result_b, ProcessActionResult::Started),
        "gen B claim should return Started, got {result_b:?}"
    );

    // INVARIANT ASSERTION: projection must be `true` because B's slot is
    // `Generating` in the registry. Poll to give A's pipeline time to abort
    // and drop — pre-fix TOCTOU would have flipped projection to `false`
    // here when A's release_owned_slot clobbered B's store(true).
    let projection_held =
        wait_for_condition(Duration::from_secs(5), Duration::from_millis(25), || {
            app.is_generating().load(Ordering::SeqCst)
        })
        .await;
    assert!(
        projection_held,
        "TOCTOU regression: projection must stay true after B claims \
         while A is still in flight. Pre-fix this could be false because \
         A's release_owned_slot stored false on the projection OUTSIDE the \
         registry write lock, racing B's claim and clobbering B's store(true)."
    );

    // Wait for B's pipeline to complete + drop. Final assertion:
    // projection back to `false` once the registry is empty.
    let b_completed =
        wait_for_condition(Duration::from_secs(10), Duration::from_millis(50), || {
            !app.is_generating().load(Ordering::SeqCst)
        })
        .await;
    assert!(b_completed, "gen B's pipeline must complete within timeout");
    assert!(
        !app.is_generating().load(Ordering::SeqCst),
        "projection must be false after B completes"
    );

    app.cancel_token().cancel();
}

async fn wait_for_condition(
    timeout: Duration,
    poll: Duration,
    mut cond: impl FnMut() -> bool,
) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if cond() {
            return true;
        }
        tokio::time::sleep(poll).await;
    }
    false
}
