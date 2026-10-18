//! HTTP E2E tests for dashboard chrome: the shell's icon sprite and the generating-status surfaces. Tagged against `docs/specs/dashboard.md`.

use std::sync::Arc;

use chronicler_engine::test_support::{analysis_report, ModelRole, ScriptedModel, TestAppBuilder};

use crate::support::http_assertions::{defined_icons, referenced_icons};
use crate::support::http_fixtures::seed_offered_set;
use crate::support::http_requests::{fetch_body, fetch_generating_status, post_action, wait_settled};

// [docs/specs/dashboard.md] SCENARIO: 39.1
#[tokio::test]
async fn test_generating_status_answers_from_live_registry() {
    assert_eq!(
        fetch_generating_status(&TestAppBuilder::default_app()).await,
        "idle",
        "a Game with no Job and no failure line answers idle"
    );

    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(ModelRole::Narration);
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::clone(&model))
        .build_with_state();
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held narrator call should reach the model"
    );
    assert_eq!(
        fetch_generating_status(&app).await,
        "narrating",
        "a Job at its narrator call answers its step name"
    );
    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the released Job should complete"
    );

    let model = Arc::new(
        ScriptedModel::new()
            .narration("You look around.")
            .analysis(analysis_report(&[], None)),
    );
    let hold = model.hold(ModelRole::Analysis);
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::clone(&model))
        .build_with_state();
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held analysis call should reach the model"
    );
    assert_eq!(
        fetch_generating_status(&app).await,
        "quantifying",
        "a Job at its analysis answers its step name"
    );
    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the released Job should complete"
    );

    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(ScriptedModel::new().failed(ModelRole::Narration)))
        .build_with_state();
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the failing Job should end"
    );
    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("answer could not be read"),
        "a Game with no Job and a failure line answers that line, got: {status}"
    );
}

// A `<use>` that names a missing symbol draws nothing and raises no error, so
// the reference is the only place a misspelt icon shows up.
// [docs/specs/dashboard.md] SCENARIO: 39.2
#[tokio::test]
async fn test_every_referenced_icon_is_defined_once_by_the_shell() {
    // A real turn gives the shell a story of more than its opening Narration.
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(ScriptedModel::new().turn("You look around.")))
        .build_with_state();
    assert!(post_action(&app, "look around").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the Action should complete"
    );

    let shell = fetch_body(&app, "/").await;
    let defined = defined_icons(&shell);
    let mut distinct = defined.clone();
    distinct.sort();
    distinct.dedup();
    assert!(!defined.is_empty(), "the shell must define the icon sprite");
    assert!(
        defined.iter().any(|icon| icon == "pencil"),
        "the sprite's ids read as bare names, not `i-` prefixed ids: {defined:?}"
    );
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

// [docs/specs/dashboard.md] SCENARIO: 39.3
#[tokio::test]
async fn test_the_dock_and_the_debug_endpoint_answer_from_the_live_registry() {
    // A held Job is the busy state: the Game holds a Job entry.
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(ModelRole::Narration);
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::clone(&model))
        .build_with_state();
    seed_offered_set(&state, &["Ask the bartender"]);
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held narrator call should reach the model"
    );

    assert_eq!(
        fetch_body(&app, "/debug/is_generating").await,
        "true",
        "a Game with a Job reports true"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("disabled"),
        "a Game with a Job disables the dock controls: {dock}"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the released Job should complete"
    );

    let (app, state) = TestAppBuilder::default_test().build_with_state();
    seed_offered_set(&state, &["Ask the bartender"]);

    assert_eq!(
        fetch_body(&app, "/debug/is_generating").await,
        "false",
        "a Game with no Job reports false"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("disabled"),
        "a Game with no Job leaves the dock controls enabled: {dock}"
    );
}
