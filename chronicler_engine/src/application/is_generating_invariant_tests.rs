//! [DOC: docs/adr/adr-030-is-generating-invariant.md]
//! Property tests enforcing ADR-030: `is_generating` AtomicBool must agree with persisted `GenerationStatus`.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::context::OpContext;
use crate::application::ProcessActionResult;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::adapters::driven::storage::Storage;
use crate::test_support::make_test_context_with_sqlite;

fn cached_flag(ctx: &OpContext) -> bool {
    ctx.is_generating.load(Ordering::SeqCst)
}

fn persisted_flag(ctx: &OpContext) -> bool {
    ctx.storage
        .load_latest_snapshot()
        .ok()
        .flatten()
        .map(|snap| {
            GameState::from_snapshot(
                &snap,
                ctx.world_snapshot.world.clone(),
                ctx.world_snapshot.map.clone(),
                ctx.world_snapshot.player.clone(),
                (*ctx.world_snapshot.npcs).clone(),
            )
            .narrative
            .input_buffer
            .status
            .is_generating()
        })
        .unwrap_or(false)
}

fn invariant_holds(ctx: &OpContext) -> bool {
    cached_flag(ctx) == persisted_flag(ctx)
}

async fn wait_until_idle(ctx: &OpContext, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if !cached_flag(ctx) && !persisted_flag(ctx) {
            return true;
        }
        sleep(Duration::from_millis(50)).await;
    }
    false
}

fn make_service_from_ctx(ctx: &OpContext) -> DefaultApplicationService {
    let game_service = Arc::new(
        crate::bootstrap::wiring::build_game_service_for_tests(
            ctx.settings.clone(),
            Arc::clone(&ctx.storage),
            Arc::clone(&ctx.preset_storage),
        )
        .expect("build_game_service_for_tests should succeed"),
    );
    DefaultApplicationService::new(
        Arc::clone(&ctx.storage),
        Arc::clone(&ctx.preset_storage),
        ctx.settings.clone(),
        ctx.cancel_token.clone(),
        Arc::clone(&ctx.is_generating),
        game_service,
    )
}

#[tokio::test]
async fn test_is_generating_invariant_holds_across_lifecycle() {
    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let ctx = make_test_context_with_sqlite(state).expect("make_test_context_with_sqlite");

    assert!(
        invariant_holds(&ctx),
        "Invariant violated at startup: cached={} persisted={}",
        cached_flag(&ctx),
        persisted_flag(&ctx)
    );

    let app_service = make_service_from_ctx(&ctx);
    let result = app_service
        .process_action("examine the room".to_string())
        .expect("process_action should succeed");
    assert!(
        matches!(result, ProcessActionResult::Started),
        "process_action should return Started"
    );

    assert!(
        invariant_holds(&ctx),
        "Invariant violated mid-generation: cached={} persisted={}",
        cached_flag(&ctx),
        persisted_flag(&ctx)
    );
    assert!(
        cached_flag(&ctx),
        "AtomicBool must be true during generation"
    );
    assert!(
        persisted_flag(&ctx),
        "Persisted status must be Generating during generation"
    );

    let completed = wait_until_idle(&ctx, Duration::from_secs(10)).await;
    assert!(completed, "Generation did not complete within timeout");

    assert!(
        invariant_holds(&ctx),
        "Invariant violated after completion: cached={} persisted={}",
        cached_flag(&ctx),
        persisted_flag(&ctx)
    );
    assert!(
        !cached_flag(&ctx),
        "AtomicBool must be false after completion"
    );
    assert!(
        !persisted_flag(&ctx),
        "Persisted status must be Idle after completion"
    );
}

#[tokio::test]
async fn test_is_generating_invariant_holds_under_concurrent_load() {
    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let ctx = make_test_context_with_sqlite(state).expect("make_test_context_with_sqlite");

    let app_service = Arc::new(make_service_from_ctx(&ctx));

    let mut handles = Vec::new();
    for i in 0..4 {
        let svc = Arc::clone(&app_service);
        let ctx_clone = ctx.clone();
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

    let completed = wait_until_idle(&ctx, Duration::from_secs(15)).await;
    assert!(completed, "Generation did not complete within timeout");

    assert!(
        invariant_holds(&ctx),
        "Invariant violated after concurrent load: cached={} persisted={}",
        cached_flag(&ctx),
        persisted_flag(&ctx)
    );
    assert!(
        !cached_flag(&ctx),
        "AtomicBool must be false after completion"
    );
    assert!(
        !persisted_flag(&ctx),
        "Persisted status must be Idle after completion"
    );
}

#[test]
fn test_is_generating_invariant_helper_detects_divergence() {
    let mut state = crate::test_support::fixtures::TestGameState::in_room("start");
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let ctx = make_test_context_with_sqlite(state).expect("make_test_context_with_sqlite");

    assert!(
        invariant_holds(&ctx),
        "Invariant should hold initially: cached={} persisted={}",
        cached_flag(&ctx),
        persisted_flag(&ctx)
    );

    ctx.is_generating.store(true, Ordering::SeqCst);

    assert!(
        !invariant_holds(&ctx),
        "Invariant helper must detect divergence: cached=true persisted=false"
    );
    assert!(cached_flag(&ctx), "AtomicBool forced to true");
    assert!(
        !persisted_flag(&ctx),
        "Persisted status should still report Idle"
    );
}
