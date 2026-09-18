//! HTTP E2E tests for the worlds update endpoint: the posture merge contract, the options-toggle checkbox grammar, and the auto-save posture endpoint.

use std::sync::Arc;

use axum::http;

use chronicler_engine::adapters::driven::storage::{Storage, TestOverride};
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{TestAppBuilder, TestMap};

use crate::test_helpers::{fetch_body, post_form, post_form_with_hx, response_body, world_form_body};

fn posture_world() -> WorldCard {
    WorldCard {
        key: "posture_world".to_string(),
        name: "Posture World".to_string(),
        description: "A world for the update contract tests.".to_string(),
        narrator_mode: NarratorMode::InteractiveFiction,
        narrative_perspective: NarrativePerspective::Second,
        narrative_tense: NarrativeTense::Past,
        ..Default::default()
    }
}

async fn updated_world(state: &chronicler_engine::adapters::driving::http::AppState) -> WorldCard {
    let (_world_id, card, _map) = state
        .world_catalogue
        .get_world("posture_world")
        .expect("get_world should succeed")
        .expect("the world should exist");
    card
}

// [docs/specs/worlds.md] SCENARIO: 25.1
#[tokio::test]
async fn test_world_update_without_posture_fields_preserves_posture_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let resp = post_form(
        &app,
        "/worlds/posture_world",
        &world_form_body(&world, &TestMap::single_room("start")),
    )
    .await;
    assert!(resp.status().is_success(), "the update should succeed");

    let stored = updated_world(&state).await;
    assert_eq!(stored.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(stored.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(stored.narrative_tense, NarrativeTense::Past);
}

// [docs/specs/worlds.md] SCENARIO: 25.2
#[tokio::test]
async fn test_world_update_partial_posture_merges_per_field_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let body = format!(
        "{}&narrative_tense=present",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &body).await;
    assert!(resp.status().is_success());

    let stored = updated_world(&state).await;
    assert_eq!(stored.narrative_tense, NarrativeTense::Present);
    assert_eq!(stored.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(stored.narrative_perspective, NarrativePerspective::Second);
}

// [docs/specs/worlds.md] SCENARIO: 25.3
#[tokio::test]
async fn test_world_update_unknown_posture_value_falls_back_to_default_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let body = format!(
        "{}&narrator_mode=warp_drive",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &body).await;
    assert!(
        resp.status().is_success(),
        "the update should still succeed"
    );

    let stored = updated_world(&state).await;
    assert_eq!(
        stored.narrator_mode,
        NarratorMode::Novel,
        "an unknown value must fall back to the domain default"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.5
#[tokio::test]
async fn test_world_posture_autosave_returns_saved_span_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let resp = post_form_with_hx(
        &app,
        "/worlds/posture_world/posture",
        "narrative_tense=present",
    )
    .await;
    assert!(resp.status().is_success(), "the auto-save should succeed");
    let body = response_body(resp).await;
    assert!(
        body.contains("Saved") && body.contains("posture-status"),
        "the auto-save must return the Saved status span, got: {body:?}"
    );

    assert_eq!(
        updated_world(&state).await.narrative_tense,
        NarrativeTense::Present,
        "the tense patch must be persisted"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.5
#[tokio::test]
async fn test_world_posture_invalid_value_returns_error_span_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let resp = post_form_with_hx(
        &app,
        "/worlds/posture_world/posture",
        "narrator_mode=warp_drive",
    )
    .await;
    assert!(
        resp.status().is_success(),
        "an invalid value is a form error, not a 500"
    );
    let body = response_body(resp).await;
    assert!(
        body.contains("error"),
        "an invalid posture value must render an error span, got: {body:?}"
    );

    assert_eq!(
        updated_world(&state).await.narrator_mode,
        NarratorMode::InteractiveFiction,
        "an invalid patch must mutate nothing"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.5
#[tokio::test]
async fn test_world_posture_unknown_world_returns_bad_request_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    let resp = post_form_with_hx(
        &app,
        "/worlds/absent_world/posture",
        "narrative_tense=present",
    )
    .await;
    assert_eq!(
        resp.status(),
        http::StatusCode::BAD_REQUEST,
        "an unknown world key must be a 400"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.5
#[tokio::test]
async fn test_world_posture_storage_failure_returns_500_http() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "update_world",
        TestOverride::internal("update_world posture failure"),
    ));
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let resp = post_form_with_hx(
        &app,
        "/worlds/posture_world/posture",
        "narrative_tense=present",
    )
    .await;
    assert_eq!(
        resp.status(),
        http::StatusCode::INTERNAL_SERVER_ERROR,
        "a storage failure must surface as a 500"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.4
#[tokio::test]
async fn test_world_update_options_toggle_checkbox_grammar_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let checked = format!(
        "{}&options_always_on=true",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &checked).await;
    assert!(resp.status().is_success());
    assert!(
        updated_world(&state).await.options_always_on,
        "a checked box must set the toggle"
    );

    let resp = post_form(
        &app,
        "/worlds/posture_world",
        &world_form_body(&world, &TestMap::single_room("start")),
    )
    .await;
    assert!(resp.status().is_success());
    assert!(
        !updated_world(&state).await.options_always_on,
        "an absent field must reset the toggle to false"
    );
}

// The only prior coverage of the full edit-form render was the browser copy
// (SCENARIO 29.1), which reached it by clicking the worlds tab and the Edit
// button; the quarantine covers only the not-found path. The form is a plain
// server render, so a GET observes the same fact without the race.
// [docs/specs/worlds.md] SCENARIO: 25.6
#[tokio::test]
async fn test_world_edit_form_renders_posture_selects_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let html = fetch_body(&app, "/worlds/posture_world/edit").await;
    assert!(
        html.contains("Edit World"),
        "the edit form must render, not the create form: {html}"
    );
    assert!(
        html.contains(r#"class="form-group posture-group""#),
        "the posture group must render: {html}"
    );

    for name in ["narrator_mode", "narrative_perspective", "narrative_tense"] {
        assert!(
            html.contains(&format!(r#"<select name="{name}""#)),
            "the {name} select must be rendered: {html}"
        );
    }

    // The selected options prove the form rendered the world's stored
    // posture, which is what the browser copy asserted by reading
    // `select.value` after the swap.
    assert!(
        html.contains(r#"<option value="interactive_fiction" selected"#),
        "the stored Interactive Fiction mode must render selected: {html}"
    );
    assert!(
        html.contains(r#"<option value="second" selected"#),
        "the stored Second-person perspective must render selected: {html}"
    );
    assert!(
        html.contains(r#"<option value="past" selected"#),
        "the stored Past tense must render selected: {html}"
    );

    // The auto-save target the browser test's settle gate waited on.
    assert!(
        html.contains(r#"id="world-posture-status""#),
        "the posture status target must be rendered: {html}"
    );
    assert!(
        html.contains(r#"hx-post="/worlds/posture_world/posture""#),
        "the posture selects must auto-save to this world: {html}"
    );
}
