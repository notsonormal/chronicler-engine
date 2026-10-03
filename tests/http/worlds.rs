//! HTTP E2E tests for the worlds endpoints: the update posture merge contract, the options-toggle checkbox grammar, the auto-save posture endpoint, and the duplicate-identifier create refusal.

use std::sync::Arc;

use axum::http;

use chronicler_engine::adapters::driven::storage::{Storage, TestOverride};
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{TestAppBuilder, TestMap};

use crate::support::http_fixtures::world_form_body;
use crate::support::http_requests::{fetch_body, post_form, post_form_with_hx, response_body};
use crate::support::http_assertions::assert_option_selected;

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

/// A default app with `posture_world` seeded and one room.
///
/// Every posture-contract test starts from this state, so the storage handle is
/// dropped by the caller unless a test needs it (the storage-failure test
/// builds its own).
fn posture_world_app() -> (
    axum::Router,
    chronicler_engine::adapters::driving::http::AppState,
) {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    storage
        .seed_world(&posture_world(), &TestMap::single_room("start"))
        .expect("seed posture world");
    (app, state)
}

async fn updated_world(
    state: &chronicler_engine::adapters::driving::http::AppState,
) -> (WorldCard, chronicler_engine::domain::model::map::MapDef) {
    let (_world_id, card, map) = state
        .world_catalogue
        .get_world("posture_world")
        .expect("get_world should succeed")
        .expect("the world should exist");
    (card, map)
}

// [docs/specs/worlds.md] SCENARIO: 25.1
#[tokio::test]
async fn test_world_update_without_posture_fields_preserves_posture_http() {
    let (app, state) = posture_world_app();
    let world = posture_world();

    let resp = post_form(
        &app,
        "/worlds/posture_world",
        &world_form_body(&world, &TestMap::single_room("start")),
    )
    .await;
    assert!(resp.status().is_success(), "the update should succeed");

    let (stored, _) = updated_world(&state).await;
    assert_eq!(stored.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(stored.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(stored.narrative_tense, NarrativeTense::Past);
}

// [docs/specs/worlds.md] SCENARIO: 25.2
#[tokio::test]
async fn test_world_update_partial_posture_merges_per_field_http() {
    let (app, state) = posture_world_app();
    let world = posture_world();

    let body = format!(
        "{}&narrative_tense=present",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &body).await;
    assert!(resp.status().is_success());

    let (stored, _) = updated_world(&state).await;
    assert_eq!(stored.narrative_tense, NarrativeTense::Present);
    assert_eq!(stored.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(stored.narrative_perspective, NarrativePerspective::Second);
}

// [docs/specs/worlds.md] SCENARIO: 25.3
#[tokio::test]
async fn test_world_update_unknown_posture_value_falls_back_to_default_http() {
    let (app, state) = posture_world_app();
    let world = posture_world();

    let body = format!(
        "{}&narrator_mode=warp_drive",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &body).await;
    assert!(
        resp.status().is_success(),
        "the update should still succeed"
    );

    let (stored, _) = updated_world(&state).await;
    assert_eq!(
        stored.narrator_mode,
        NarratorMode::Novel,
        "an unknown value must fall back to the domain default"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.5
#[tokio::test]
async fn test_world_posture_autosave_returns_saved_span_http() {
    let (app, state) = posture_world_app();

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

    let (stored, _) = updated_world(&state).await;
    assert_eq!(
        stored.narrative_tense,
        NarrativeTense::Present,
        "the tense patch must be persisted"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.5
#[tokio::test]
async fn test_world_posture_invalid_value_returns_error_span_http() {
    let (app, state) = posture_world_app();

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
        "an invalid posture value must render an error fragment, got: {body:?}"
    );

    let (stored, _) = updated_world(&state).await;
    assert_eq!(
        stored.narrator_mode,
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
    let (app, state) = posture_world_app();
    let world = posture_world();

    let checked = format!(
        "{}&options_always_on=true",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &checked).await;
    assert!(resp.status().is_success());
    let (stored, _) = updated_world(&state).await;
    assert!(
        stored.options_always_on,
        "a checked box must set the toggle"
    );

    let resp = post_form(
        &app,
        "/worlds/posture_world",
        &world_form_body(&world, &TestMap::single_room("start")),
    )
    .await;
    assert!(resp.status().is_success());
    let (reset_stored, _) = updated_world(&state).await;
    assert!(
        !reset_stored.options_always_on,
        "an absent field must reset the toggle to false"
    );
}

// The form is a plain server render, so a GET observes the rendered posture
// selects; the quarantine covers only the not-found path.
// [docs/specs/worlds.md] SCENARIO: 25.6
#[tokio::test]
async fn test_world_edit_form_renders_posture_selects_http() {
    let (app, _state) = posture_world_app();

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

    // The selected options prove the form rendered the world's stored posture.
    assert_option_selected(
        &html,
        "interactive_fiction",
        "the stored Interactive Fiction mode must render selected",
    );
    assert_option_selected(
        &html,
        "second",
        "the stored Second-person perspective must render selected",
    );
    assert_option_selected(&html, "past", "the stored Past tense must render selected");

    // The auto-save target the posture select swaps into.
    assert!(
        html.contains(r#"id="world-posture-status""#),
        "the posture status target must be rendered: {html}"
    );
    assert!(
        html.contains(r#"hx-post="/worlds/posture_world/posture""#),
        "the posture selects must auto-save to this world: {html}"
    );
}

// The world list is a plain server render; a GET observes the pluralised
// game count directly, with no browser round-trip.
// [docs/specs/worlds.md] SCENARIO: 25.8
#[tokio::test]
async fn test_worlds_fragment_pluralises_game_count_http() {
    // `default_test` seeds one World with exactly one game in it.
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let html = fetch_body(&app, "/fragment/worlds").await;

    assert!(
        html.contains("(1 game)"),
        "a World with one game must show a singular count: {html}"
    );
    assert!(
        !html.contains("(1 games)"),
        "the count must not pluralise a single game: {html}"
    );

    // A second game in the same World switches the count to the plural arm.
    state
        .game_catalogue
        .create_game("test", "test_player")
        .expect("create a second game");

    let html = fetch_body(&app, "/fragment/worlds").await;
    assert!(
        html.contains("(2 games)"),
        "a World with two games must use the plural count: {html}"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.7
#[tokio::test]
async fn test_world_create_existing_key_is_refused_http() {
    let (app, state) = posture_world_app();

    let second = WorldCard {
        key: "posture_world".to_string(),
        name: "Second World".to_string(),
        description: "A second world that must never be stored.".to_string(),
        ..Default::default()
    };
    let resp = post_form(
        &app,
        "/worlds",
        &world_form_body(&second, &TestMap::single_room("second_room")),
    )
    .await;

    assert_eq!(
        resp.status(),
        http::StatusCode::BAD_REQUEST,
        "a duplicate-key create must be refused"
    );
    let body = response_body(resp).await;
    assert!(
        body.contains("posture_world") && body.contains("already exists"),
        "the refusal must name the key and say it exists: {body:?}"
    );

    let (stored, stored_map) = updated_world(&state).await;
    assert_eq!(
        stored.name, "Posture World",
        "the existing world must not be overwritten"
    );
    assert_eq!(stored.key, "posture_world");
    assert_eq!(
        stored.description, "A world for the update contract tests.",
        "the existing description must not be overwritten"
    );
    assert_eq!(
        stored_map.overworld.regions[0].rooms[0].id, "start",
        "the existing map must not be overwritten"
    );
}

// A storage failure is not a client refusal: the create handler answers the
// shared error fragment with a 200, unlike the duplicate-key refusal's 400 and
// the posture endpoint's 500 on the same storage seam.
// [docs/specs/worlds.md] SCENARIO: 25.9
#[tokio::test]
async fn test_world_create_storage_failure_renders_error_fragment_http() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "create_world",
        TestOverride::internal("create_world failure"),
    ));
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    let world = WorldCard {
        key: "new_world".to_string(),
        name: "New World".to_string(),
        description: "A world that must not be stored.".to_string(),
        ..Default::default()
    };
    let resp = post_form(
        &app,
        "/worlds",
        &world_form_body(&world, &TestMap::single_room("start")),
    )
    .await;

    assert_eq!(
        resp.status(),
        http::StatusCode::OK,
        "a storage failure is rendered in-fragment with a 200, not a 500"
    );
    let body = response_body(resp).await;
    assert!(
        body.contains("error-message") && body.contains("create_world failure"),
        "the error fragment must name the storage failure: {body:?}"
    );
}
