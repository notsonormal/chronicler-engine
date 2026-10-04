//! HTTP E2E tests for game rename (POST /games/:id/rename).

use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

use crate::support::http_requests::{fetch_body, response_body};

// [docs/specs/games.md] SCENARIO: 17.4
#[tokio::test]
async fn test_rename_game_updates_display_name_in_header_and_fragment() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let active = state
        .game_catalogue
        .current_game()
        .expect("the active game must load")
        .expect("the fixture seeds an active game");
    let stable_name = active.name.clone();

    let req = Request::builder()
        .uri(format!("/games/{}/rename", active.id))
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("display_name=The+Long+Road"))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("HX-Refresh").unwrap(),
        "true",
        "rename must refresh so the header and Games tab re-render"
    );

    let renamed = state
        .game_catalogue
        .current_game()
        .unwrap()
        .expect("the active game must still load");
    assert_eq!(renamed.display_name, "The Long Road");
    assert_eq!(
        renamed.name, stable_name,
        "rename must not disturb the stable generated name"
    );

    let header = fetch_body(&app, "/fragment/header").await;
    assert!(
        header.contains("The Long Road"),
        "the header must show the display name: {header}"
    );

    let games = fetch_body(&app, "/fragment/games").await;
    assert!(
        games.contains(r#"<span class="game-name">The Long Road</span>"#),
        "the Games tab must show the display name: {games}"
    );
    assert!(
        !games.contains(r#"<span class="game-name">Test Game</span>"#),
        "the Games tab must not show the old name: {games}"
    );
}

// [docs/specs/games.md] SCENARIO: 17.5
#[tokio::test]
async fn test_rename_game_blank_display_name_is_rejected() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let active = state
        .game_catalogue
        .current_game()
        .expect("the active game must load")
        .expect("the fixture seeds an active game");

    let req = Request::builder()
        .uri(format!("/games/{}/rename", active.id))
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("display_name=+++"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response_body(response).await;
    assert!(
        body.contains("Display name cannot be empty"),
        "Expected the blank-display-name error: {body}"
    );

    let unchanged = state
        .game_catalogue
        .current_game()
        .unwrap()
        .expect("the active game must still load");
    assert_eq!(
        unchanged.display_name, active.display_name,
        "a rejected rename must not change the stored display name"
    );
}
