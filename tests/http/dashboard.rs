//! HTTP E2E tests for dashboard chrome: the generating status poll. Tagged against `docs/specs/dashboard.md`.

use chronicler_engine::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use chronicler_engine::TestAppBuilder;

use crate::support::http_requests::fetch_generating_status;

// [docs/specs/dashboard.md] SCENARIO: 39.1
#[tokio::test]
async fn test_generating_status_answers_from_live_registry() {
    assert_eq!(
        fetch_generating_status(&TestAppBuilder::default_app()).await,
        "idle",
        "default status should be idle"
    );

    for phase in [GenerationPhase::Narrating, GenerationPhase::Quantifying] {
        let body = fetch_generating_status(
            &TestAppBuilder::default_test()
                .generation_status(GenerationStatus::Generating, phase)
                .build(),
        )
        .await;
        assert_eq!(body, "idle", "stale Generating should answer idle");
    }

    // `is_generating(true)` claims the slot; `generation_status` re-applies the phase.
    for phase in [GenerationPhase::Narrating, GenerationPhase::Quantifying] {
        let expected = phase.as_endpoint_str();
        let body = fetch_generating_status(
            &TestAppBuilder::default_test()
                .generation_status(GenerationStatus::Generating, phase)
                .is_generating(true)
                .build(),
        )
        .await;
        assert_eq!(
            body, expected,
            "live slot should report the persisted phase"
        );
    }
}
