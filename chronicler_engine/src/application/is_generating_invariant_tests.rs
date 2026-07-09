//! [DOC: docs/adr/adr-030-is-generating-invariant.md]
//! Property tests enforcing ADR-030: `is_generating` AtomicBool must agree with persisted `GenerationStatus`.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tokio::time::sleep;

use crate::application::application_service::DefaultApplicationService;
use crate::application::ProcessActionResult;
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
                .map(|s| s.narrative.input_buffer.status.is_generating())
                .unwrap_or(false)
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

        // Forbidden at any poll: AtomicBool=false but persisted=Generating — would
        // mean the caller forgot to either release the slot or persist the new state.
        if !cached && persisted {
            panic!(
                "invariant violation during wait_until_idle: cached=false persisted=true. \
                 expected (cached=true, persisted=Idle) as the only allowed transient."
            );
        }

        // Allowed transient: (cached=true, persisted=Idle) — generation in flight but
        // no snapshot yet (early phase of save_message_and_snapshot pipeline).
        if cached && !persisted {
            sleep(Duration::from_millis(50)).await;
            continue;
        }

        // Both idle — generation complete.
        if !cached && !persisted {
            assert!(invariant_holds(app), "both idle but invariant violated");
            return true;
        }

        // (cached=true, persisted=true) — generation persisted + AtomicBool still up.
        sleep(Duration::from_millis(50)).await;
    }
    false
}

#[tokio::test]
async fn test_is_generating_invariant_holds_across_lifecycle() {
    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let app = make_test_app_with_sqlite(state).expect("make_test_app_with_sqlite");

    assert!(
        invariant_holds(&app),
        "Invariant violated at startup: cached={} persisted={}",
        cached_flag(&app),
        persisted_flag(&app)
    );

    let result = app
        .process_action("examine the room".to_string())
        .expect("process_action should succeed");
    assert!(
        matches!(result, ProcessActionResult::Started),
        "process_action should return Started"
    );

    assert!(
        invariant_holds(&app),
        "Invariant violated mid-generation: cached={} persisted={}",
        cached_flag(&app),
        persisted_flag(&app)
    );
    assert!(
        cached_flag(&app),
        "AtomicBool must be true during generation"
    );
    assert!(
        persisted_flag(&app),
        "Persisted status must be Generating during generation"
    );

    let completed = wait_until_idle(&app, Duration::from_secs(10)).await;
    assert!(completed, "Generation did not complete within timeout");

    assert!(
        invariant_holds(&app),
        "Invariant violated after completion: cached={} persisted={}",
        cached_flag(&app),
        persisted_flag(&app)
    );
    assert!(
        !cached_flag(&app),
        "AtomicBool must be false after completion"
    );
    assert!(
        !persisted_flag(&app),
        "Persisted status must be Idle after completion"
    );
}

#[tokio::test]
async fn test_is_generating_invariant_holds_under_concurrent_load() {
    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let app = make_test_app_with_sqlite(state).expect("make_test_app_with_sqlite");

    let mut handles = Vec::new();
    for i in 0..4 {
        let svc = std::sync::Arc::clone(&app);
        handles.push(tokio::task::spawn_blocking(move || {
            svc.process_action(format!("input from thread {i}"))
        }));
    }

    let mut started = 0usize;
    let mut concurrent = 0usize;
    for h in handles {
        let result = h.await.expect("spawn_blocking task panicked");
        match result {
            Ok(ProcessActionResult::Started) => started += 1,
            Ok(ProcessActionResult::ConcurrentGeneration) => concurrent += 1,
            other => panic!(
                "Unexpected result: {}",
                match other {
                    Ok(ProcessActionResult::Started) => "Started".to_string(),
                    Ok(ProcessActionResult::ConcurrentGeneration) =>
                        "ConcurrentGeneration".to_string(),
                    Ok(ProcessActionResult::ShuttingDown) => "ShuttingDown".to_string(),
                    Err(e) => format!("Err({e})"),
                }
            ),
        }
    }

    assert_eq!(started, 1, "Exactly one caller should win generation slot");
    assert_eq!(
        concurrent, 3,
        "Three callers should be rejected as concurrent"
    );

    let completed = wait_until_idle(&app, Duration::from_secs(15)).await;
    assert!(completed, "Generation did not complete within timeout");

    assert!(
        invariant_holds(&app),
        "Invariant violated after concurrent load: cached={} persisted={}",
        cached_flag(&app),
        persisted_flag(&app)
    );
    assert!(
        !cached_flag(&app),
        "AtomicBool must be false after completion"
    );
    assert!(
        !persisted_flag(&app),
        "Persisted status must be Idle after completion"
    );
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

/// New test per plan B4.1: inject `(cached=false, persisted=Generating)` during
/// flight by manually clearing AtomicBool while generation in progress. wait_until_idle
/// must fail-fast (panic) within 1-2 poll cycles, not silently wait for timeout.
#[tokio::test(flavor = "current_thread")]
#[should_panic(expected = "invariant violation during wait_until_idle")]
async fn test_wait_until_idle_fails_fast_on_cached_false_persisted_generating() {
    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let app = std::sync::Arc::new(
        make_test_app_with_sqlite(state).expect("make_test_app_with_sqlite"),
    );

    // Simulate the dangerous state directly: cached=false, persisted status=Generating.
    // (We bypass process_action because the production CAS would forbid cached=false
    // when status=Generating at the same point.)
    let mut gs = app.load_or_fresh().expect("load_or_fresh");
    gs.narrative.input_buffer.status = GenerationStatus::Generating;
    let snapshot_id = app
        .save_state(&gs)
        .expect("save_state should persist Generating");
    // Ensure AtomicBool stays false to simulate the bug.
    app.is_generating().store(false, Ordering::SeqCst);
    let _ = snapshot_id;

    // wait_until_idle must panic on the forbidden (cached=false, persisted=true)
    // state per the B4.1 fail-fast contract.
    let _ = wait_until_idle(&app, Duration::from_secs(2)).await;
}
