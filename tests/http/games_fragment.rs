//! HTTP E2E tests for the games panel fragment (`GET /fragment/games`) — the posture fragment's rendered selects and preset pickers.

use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::test_support::active_game_of;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::test_support::TestAppBuilder;

use crate::support::http_assertions::assert_option_selected;
use crate::support::http_requests::{fetch_body, response_body};
use crate::test_utils::panel_section_html_slice;

// The fragment is a plain server render, so a GET observes the rendered
// posture controls directly, without a browser round-trip.
// [docs/specs/games.md] SCENARIO: 20.8
#[tokio::test]
async fn test_games_fragment_renders_posture_controls_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let game_id = active_game_of(&state);

    let html = fetch_body(&app, "/fragment/games").await;

    assert!(
        html.contains(r#"id="game-posture-controls""#),
        "the fragment must render the posture controls: {html}"
    );

    assert_option_selected(
        &html,
        "novel",
        "the active game's Novel mode must render selected",
    );
    assert_option_selected(
        &html,
        "third",
        "the active game's Third-person perspective must render selected",
    );
    assert_option_selected(
        &html,
        "past",
        "the active game's Past tense must render selected",
    );

    for name in [
        "narrator_mode",
        "narrative_perspective",
        "narrative_tense",
        "system_preset_id",
        "quantifier_preset_id",
        "impersonate_preset_id",
        "options_preset_id",
    ] {
        assert!(
            html.contains(&format!(r#"<select name="{name}""#)),
            "the {name} select must be rendered: {html}"
        );
    }
    assert!(
        html.contains(&format!(r#"hx-post="/games/{game_id}/mode""#)),
        "the mode select must auto-save to this game: {html}"
    );
    assert!(
        html.contains(&format!(r#"hx-post="/games/{game_id}/posture""#)),
        "the posture selects must auto-save to this game: {html}"
    );
    assert!(
        html.contains(&format!(r#"hx-post="/games/{game_id}/presets""#)),
        "the preset pickers must auto-save to this game: {html}"
    );

    assert!(
        !html.contains("Presets unavailable"),
        "the preset pickers must render without a load error: {html}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.9
#[tokio::test]
async fn test_games_fragment_lists_only_other_saved_games_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let active_name = state
        .game_catalogue
        .active_game()
        .expect("the active game must load")
        .expect("the fixture seeds an active game")
        .name;

    let html = fetch_body(&app, "/fragment/games").await;

    assert!(
        html.contains("No other saved games."),
        "the empty Saved Games list must say there are no *other* saved games: {html}"
    );
    let active_section = panel_section_html_slice(&html, "Active Game")
        .expect("the fragment must render the Active Game section");
    assert!(
        active_section.contains(&format!(r#"<span class="game-name">{active_name}</span>"#)),
        "the active game card must render the active game: {active_section}"
    );
    assert!(
        !active_section.contains(">Current<"),
        "the active game card must not repeat the section heading with a badge: {active_section}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.10
#[tokio::test]
async fn test_games_fragment_lists_other_saved_games_http() {
    let (state, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let active = state
        .game_catalogue
        .active_game()
        .expect("the active game must load")
        .expect("the fixture seeds an active game");
    // Seeding a row does not switch the active game, so the new game lands in
    // the Saved Games list beside the active one.
    chronicler_engine::test_support::seed_game_row(
        &storage,
        &chronicler_engine::test_support::test_new_game(
            &active.world_name,
            &active.world_key,
            &active.persona_key,
            &active.persona_name,
            "Second Game",
        ),
    );

    let app = build_router(state);
    let html = fetch_body(&app, "/fragment/games").await;

    let saved_section = panel_section_html_slice(&html, "Saved Games")
        .expect("the fragment must render the Saved Games section");
    assert!(
        saved_section.contains(r#"<span class="game-name">Second Game</span>"#),
        "the Saved Games section must list the other game: {saved_section}"
    );
    assert!(
        !saved_section.contains(&format!(
            r#"<span class="game-name">{}</span>"#,
            active.name
        )),
        "the Saved Games section must not list the active game: {saved_section}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.13
#[tokio::test]
async fn test_games_fragment_is_the_panel_content_with_stable_row_ids_http() {
    let (state, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let active = state
        .game_catalogue
        .active_game()
        .expect("the active game must load")
        .expect("the fixture seeds an active game");
    let second = chronicler_engine::test_support::seed_game_row(
        &storage,
        &chronicler_engine::test_support::test_new_game(
            &active.world_name,
            &active.world_key,
            &active.persona_key,
            &active.persona_name,
            "Second Game",
        ),
    )
    .0;

    let app = build_router(state);
    let html = fetch_body(&app, "/fragment/games").await;

    assert!(
        !html.contains("games-panel"),
        "the fragment is the content of the panel, and repeats no games-panel element: {html}"
    );
    for id in [active.id, second] {
        assert!(
            html.contains(&format!(r#"id="game-row-{id}""#)),
            "the row of game {id} must carry its stable id: {html}"
        );
    }
    for id in [active.id, second] {
        assert!(
            html.contains(&format!(
                r##"hx-get="/fragment/games/{id}/rename" hx-target="#edit-dialog-body""##
            )),
            "the row of game {id} must have a Rename control that opens the dialog: {html}"
        );
    }
    assert!(
        html.contains(r##"hx-get="/fragment/games/new" hx-target="#edit-dialog-body""##),
        "the fragment must have a New Game control that opens the dialog: {html}"
    );
    assert!(
        !html.contains("<details")
            && !html.contains(r#"name="display_name""#)
            && !html.contains(r#"id="new-game-world""#),
        "the fragment must hold no disclosure and no form that waits for a Save: {html}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.37
#[tokio::test]
async fn test_rename_form_is_a_dialog_fragment_with_the_display_name() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let game_id = active_game_of(&state);
    state
        .game_catalogue
        .rename_game(game_id.0, "The Long Road")
        .expect("rename the seeded game");

    let html = fetch_body(&app, &format!("/fragment/games/{}/rename", game_id.0)).await;

    assert!(
        html.contains("data-dialog-form") && html.contains(r#"id="edit-dialog-title""#),
        "the form is a dialog form with the title that names the dialog: {html}"
    );
    assert!(
        html.contains(&format!(r#"hx-post="/games/{}/rename""#, game_id.0)),
        "the form posts to the rename route: {html}"
    );
    assert!(
        html.contains(r#"name="display_name" value="The Long Road""#),
        "the field holds the display name: {html}"
    );
    assert!(
        html.contains("data-error-slot=") && html.contains("data-close-dialog"),
        "the form has an error slot and a Cancel control that closes the dialog: {html}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.38
#[tokio::test]
async fn test_rename_form_of_an_unknown_game_is_refused() {
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/fragment/games/9999/rename")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = response_body(response).await;
    assert!(
        body.contains("error-message") && body.contains("Game not found"),
        "the refusal names the missing Game: {body}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.39
#[tokio::test]
async fn test_new_game_form_lists_the_worlds_and_the_personas() {
    let app = TestAppBuilder::default_app();

    let html = fetch_body(&app, "/fragment/games/new").await;

    assert!(
        html.contains("data-dialog-form") && html.contains(r#"id="edit-dialog-title""#),
        "the form is a dialog form with the title that names the dialog: {html}"
    );
    assert!(
        html.contains(r#"hx-post="/games""#),
        "the form posts to the create route: {html}"
    );
    assert!(
        html.contains(r#"<option value="test""#),
        "the World select lists the seeded World: {html}"
    );
    assert!(
        html.contains(r#"<option value="test_player""#),
        "the Persona select lists the seeded Persona: {html}"
    );
    assert!(
        html.contains("data-error-slot=") && html.contains("data-close-dialog"),
        "the form has an error slot and a Cancel control that closes the dialog: {html}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.40
#[tokio::test]
async fn test_new_game_form_has_no_form_when_no_world_exists() {
    let app = TestAppBuilder::default_test().skip_seeding(true).build();

    let html = fetch_body(&app, "/fragment/games/new").await;

    assert!(
        html.contains("No worlds available"),
        "the body says that no World is available: {html}"
    );
    assert!(!html.contains("<form"), "the body has no form: {html}");
}
