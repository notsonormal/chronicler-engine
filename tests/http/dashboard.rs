//! HTTP E2E tests for dashboard chrome: the shell's icon sprite and the generating status poll. Tagged against `docs/specs/dashboard.md`.

use chronicler_engine::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::TestStoredTriggerContext;
use chronicler_engine::TestAppBuilder;

use crate::support::http_assertions::{defined_icons, referenced_icons};
use crate::support::http_requests::{fetch_body, fetch_generating_status};

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

// A `<use>` that names a missing symbol draws nothing and raises no error, so
// the reference is the only place a misspelt icon shows up.
// [docs/specs/dashboard.md] SCENARIO: 39.2
#[tokio::test]
async fn test_every_referenced_icon_is_defined_once_by_the_shell() {
    let app = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("look around", MessageType::Input)
        .log("You look around.", MessageType::Narration)
        .build();

    let shell = fetch_body(&app, "/").await;
    let defined = defined_icons(&shell);
    let mut distinct = defined.clone();
    distinct.sort();
    distinct.dedup();
    assert!(!defined.is_empty(), "the shell must define the icon sprite");
    assert_eq!(
        distinct.len(),
        defined.len(),
        "the shell must define each icon once: {defined:?}"
    );
    for icon in referenced_icons(&shell) {
        assert!(
            defined.contains(&icon),
            "the shell references icon {icon}, which it does not define"
        );
    }

    for uri in [
        "/fragment/story-log",
        "/fragment/games",
        "/fragment/connections/new",
        "/fragment/action-area",
    ] {
        let icons = referenced_icons(&fetch_body(&app, uri).await);
        assert!(!icons.is_empty(), "{uri} should show at least one icon");
        for icon in icons {
            assert!(
                defined.contains(&icon),
                "{uri} references icon {icon}, which the shell does not define"
            );
        }
    }
}
