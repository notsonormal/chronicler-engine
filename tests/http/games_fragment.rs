//! HTTP E2E tests for the games panel fragment (`GET /fragment/games`) — the posture fragment's rendered selects and preset pickers.

use chronicler_engine::test_support::active_game_of;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::test_support::TestAppBuilder;

use crate::support::http_assertions::assert_option_selected;
use crate::support::http_requests::fetch_body;
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
