//! HTTP E2E tests for the games panel fragment (`GET /fragment/games`) — the posture fragment's rendered selects and preset pickers.

use chronicler_engine::test_support::TestAppBuilder;

use crate::support::http_assertions::assert_option_selected;
use crate::support::http_requests::fetch_body;

// The fragment is a plain server render, so a GET observes the rendered
// posture controls directly, without a browser round-trip.
// [docs/specs/games.md] SCENARIO: 20.8
#[tokio::test]
async fn test_games_fragment_renders_posture_controls_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let game_id = state.game_catalogue.current_game_id();

    let html = fetch_body(&app, "/fragment/games").await;

    assert!(
        html.contains(r#"id="game-posture-controls""#),
        "the fragment must render the posture controls: {html}"
    );

    // The selected option proves the fragment rendered the game's *stored*
    // posture, not just the select shells.
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

    // Every select carries the auto-save route for *this* game, so the
    // fragment's wiring is pinned to the active game's id.
    for name in [
        "narrator_mode",
        "narrative_perspective",
        "narrative_tense",
        "system_preset_id",
        "quantifier_preset_id",
        "impersonate_preset_id",
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

    // A preset-library load failure degrades the picker row to an error span
    // instead of removing the posture controls (asserted in
    // `games::templates::games_tests`); this is the healthy path.
    assert!(
        !html.contains("Presets unavailable"),
        "the preset pickers must render without a load error: {html}"
    );
}
