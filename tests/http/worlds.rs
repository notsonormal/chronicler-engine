//! HTTP E2E tests for the worlds endpoints: the update posture merge contract, the options-toggle checkbox grammar, the auto-save posture endpoint, the duplicate-identifier create refusal, and the World delete: its removal, its counted refusal and its storage failure, and the Edit form of an unknown World.

use std::sync::Arc;

use axum::http;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{TestAppBuilder, TestMap, empty_sqlite_storage};

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

fn posture_world_app() -> (
    axum::Router,
    chronicler_engine::adapters::driving::http::AppState,
) {
    let storage = Arc::new(empty_sqlite_storage());
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
    assert_eq!(
        resp.status(),
        http::StatusCode::OK,
        "the auto-save should succeed"
    );
    let body = response_body(resp).await;
    assert_eq!(body, "", "a saved posture answers nothing");

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
    assert_eq!(
        resp.status(),
        http::StatusCode::BAD_REQUEST,
        "an invalid value is a refusal: non-2xx, so the select goes back"
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
    let storage = Arc::new(empty_sqlite_storage());
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
    let storage = Arc::new(empty_sqlite_storage().with_failure(
        StorageCall::UpdateWorld,
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
        html.contains(r#"class="posture-group""#),
        "the posture group must render: {html}"
    );
    assert!(
        html.contains("Posture — saves automatically"),
        "the posture group must be labelled as auto-saving: {html}"
    );
    assert!(
        html.contains("Cancel applies to the details fields only"),
        "the form must state Cancel's scope: {html}"
    );

    for name in ["narrator_mode", "narrative_perspective", "narrative_tense"] {
        assert!(
            html.contains(&format!(r#"<select name="{name}""#)),
            "the {name} select must be rendered: {html}"
        );
    }

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

    assert!(
        html.contains(r#"id="world-posture-status""#),
        "the posture status flag must be rendered: {html}"
    );
    assert!(
        !html.contains(r#"data-error-slot="world-posture""#),
        "the posture group is no form with typed input, so its report shows in the notice: {html}"
    );
    assert!(
        html.contains("data-dialog-form") && html.contains(r#"<h2 id="edit-dialog-title">"#),
        "the editor is a dialog form with the title that names the dialog: {html}"
    );
    // The slot a failed create or update reports into, so the client keeps the
    // panel the failure describes.
    assert!(
        html.contains(r#"data-error-slot="world-form""#),
        "the world form must carry its own error slot: {html}"
    );
    assert!(
        html.contains(r#"hx-post="/worlds/posture_world/posture""#),
        "the posture selects must auto-save to this world: {html}"
    );
    assert!(
        !html.contains("hx-target"),
        "the editor targets no element: its saves swap nothing: {html}"
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
        html.contains(r#"<li class="world-item" id="world-test">"#),
        "a World row carries a stable id, so a load keeps the row: {html}"
    );
    assert!(
        html.contains(r##"hx-get="/fragment/worlds/new" hx-target="#edit-dialog-body""##)
            && html.contains(r##"hx-get="/worlds/test/edit" hx-target="#edit-dialog-body""##),
        "the Create and Edit controls open the dialog: {html}"
    );
    assert!(
        !html.contains("world-editor") && !html.contains("editor-slot"),
        "the list renders no editor slot: {html}"
    );
    assert!(
        !html.contains("worlds-panel"),
        "the shell's .worlds-panel is the part: the fragment repeats no root: {html}"
    );
    assert!(
        html.contains("(1 game)"),
        "a World with one game must show a singular count: {html}"
    );
    assert!(
        !html.contains("(1 games)"),
        "the count must not pluralise a single game: {html}"
    );

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
// shared error fragment with a 500, so the client reports it in the world
// form's own slot rather than swapping it into the panel it describes.
// [docs/specs/worlds.md] SCENARIO: 25.9
#[tokio::test]
async fn test_world_create_storage_failure_renders_error_fragment_http() {
    let storage = Arc::new(empty_sqlite_storage().with_failure(
        StorageCall::CreateWorld,
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
        http::StatusCode::INTERNAL_SERVER_ERROR,
        "a storage failure answers non-2xx so the client keeps the panel in place"
    );
    let body = response_body(resp).await;
    assert!(
        body.contains("error-message") && body.contains("create_world failure"),
        "the error fragment must name the storage failure: {body:?}"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.10
#[tokio::test]
async fn test_world_delete_refusal_pluralises_the_game_count_http() {
    // `default_test` seeds the World "test" with exactly one game in it.
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let resp = post_form(&app, "/worlds/test/delete", "").await;
    assert_eq!(
        resp.status(),
        http::StatusCode::BAD_REQUEST,
        "a refusal is non-2xx, so the row stays and its slot shows the refusal"
    );
    let html = response_body(resp).await;
    assert!(
        html.contains("Cannot delete world with 1 game"),
        "the refusal must count one game in the singular: {html}"
    );
    assert!(
        !html.contains("1 games"),
        "the refusal must not pluralise a single game: {html}"
    );

    state
        .game_catalogue
        .create_game("test", "test_player")
        .expect("create a second game");

    let html = response_body(post_form(&app, "/worlds/test/delete", "").await).await;
    assert!(
        html.contains("Cannot delete world with 2 games"),
        "the refusal must count two games in the plural: {html}"
    );
    assert!(
        state
            .world_catalogue
            .list_worlds()
            .expect("list worlds")
            .iter()
            .any(|world| world.key == "test"),
        "a refused delete keeps the World"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.11
#[tokio::test]
async fn test_world_delete_removes_a_world_with_no_games_http() {
    let (app, state) = posture_world_app();

    let resp = post_form(&app, "/worlds/posture_world/delete", "").await;
    assert_eq!(resp.status(), http::StatusCode::OK);
    assert_eq!(
        response_body(resp).await,
        "",
        "a delete answers an empty body"
    );
    assert!(
        state
            .world_catalogue
            .get_world("posture_world")
            .expect("read the world")
            .is_none(),
        "the deleted World is no longer stored"
    );
    let list = fetch_body(&app, "/fragment/worlds").await;
    assert!(
        !list.contains(r#"id="world-posture_world""#),
        "a following list does not hold the deleted World: {list}"
    );

    let resp = post_form(&app, "/worlds/nonexistent/delete", "").await;
    assert_eq!(resp.status(), http::StatusCode::OK);
    assert_eq!(
        response_body(resp).await,
        "",
        "a delete of an unknown key answers an empty body"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.12
#[tokio::test]
async fn test_world_delete_storage_failure_renders_error_fragment_http() {
    let storage = Arc::new(empty_sqlite_storage().with_failure(
        StorageCall::DeleteWorld,
        TestOverride::internal("delete_world failure"),
    ));
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    let resp = post_form(&app, "/worlds/test/delete", "").await;
    assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
    let body = response_body(resp).await;
    assert!(
        body.contains("error-message") && body.contains("delete_world failure"),
        "the error fragment must name the storage failure: {body:?}"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.14
#[tokio::test]
async fn test_unknown_world_edit_form_answers_not_found_and_opens_no_dialog_http() {
    let (app, _state) = TestAppBuilder::default_test().build_with_state();

    let request = http::Request::builder()
        .uri("/worlds/does-not-exist/edit")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = tower::util::ServiceExt::oneshot(app.clone(), request)
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        http::StatusCode::NOT_FOUND,
        "a gone record answers non-2xx, so no dialog opens"
    );
    let body = response_body(resp).await;
    assert!(
        body.contains("does-not-exist") && body.contains("not found"),
        "the body names the unknown World: {body}"
    );
    assert!(
        !body.contains("<form"),
        "an unknown World must not render an edit form: {body}"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.15
#[tokio::test]
async fn test_world_update_storage_failure_names_the_typed_name_http() {
    let storage = Arc::new(empty_sqlite_storage().with_failure(
        StorageCall::UpdateWorld,
        TestOverride::internal("update_world failure"),
    ));
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let typed = WorldCard {
        name: "Typed Name".to_string(),
        ..world
    };
    let resp = post_form(
        &app,
        "/worlds/posture_world",
        &world_form_body(&typed, &TestMap::single_room("start")),
    )
    .await;

    assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
    let body = response_body(resp).await;
    assert!(
        body.contains("The World &quot;Typed Name&quot; was not saved"),
        "the sentence names the typed name: {body}"
    );
    assert!(
        !body.contains("Posture World"),
        "the sentence does not name the stored name: {body}"
    );
    assert!(
        body.contains("update_world failure"),
        "the engine text follows: {body}"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.13
#[tokio::test]
async fn test_world_row_shows_a_shortened_description_http() {
    let storage_world = WorldCard {
        key: "long_world".to_string(),
        name: "Long World".to_string(),
        description: format!("Opening {} FINAL-WORD", "lantern ".repeat(60)),
        ..posture_world()
    };
    let storage = Arc::new(empty_sqlite_storage());
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    storage
        .seed_world(&storage_world, &TestMap::single_room("start"))
        .expect("seed the long World");

    let html = fetch_body(&app, "/fragment/worlds").await;

    assert!(
        html.contains("Opening lantern") && html.contains('…'),
        "the row shows the start of the description and a cut mark: {html}"
    );
    assert!(
        !html.contains("FINAL-WORD"),
        "the row does not show the end of a long description: {html}"
    );
}
