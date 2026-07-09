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
    let mut gs = app.load_or_fresh().expect("load_or_fresh");
    gs.narrative.input_buffer.status = GenerationStatus::Generating;
    let snapshot_id = app
        .save_state(&gs)
        .expect("save_state should persist Generating");
    app.is_generating().store(false, Ordering::SeqCst);
    let _ = snapshot_id;

    let _ = wait_until_idle(&app, Duration::from_secs(2)).await;
}
