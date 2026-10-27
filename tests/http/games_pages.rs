//! HTTP E2E tests for the pages of the saved Games on `GET /screen?parts=games`: the page size, the order, the clamp and the pager. Tagged against `docs/specs/games.md` and `docs/specs/screen.md`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::test_support::{seed_game_row, test_new_game, TestAppBuilder};

use crate::support::http_requests::post_form;
use crate::test_utils::panel_section_html_slice;

const PAGE_SIZE: usize = 50;

/// An app with an active Game and `saved` more Games named "Saga 01", "Saga 02", ...
/// The oldest is "Saga 01": each row is written after the one before it.
fn app_with_saved_games(saved: usize) -> (axum::Router, Vec<u64>) {
    let (state, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let active = state
        .game_catalogue
        .active_game()
        .expect("the active game must load")
        .expect("the fixture seeds an active game");
    let ids = (1..=saved)
        .map(|number| {
            seed_game_row(
                &storage,
                &test_new_game(
                    &active.world_name,
                    &active.world_key,
                    &active.persona_key,
                    &active.persona_name,
                    &format!("Saga {number:02}"),
                ),
            )
            .0
        })
        .collect();
    (build_router(state), ids)
}

/// A page of 50 Games is larger than the body limit of `response_body`.
async fn get(app: &axum::Router, uri: &str) -> (StatusCode, String) {
    let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .expect("read the response body");
    (status, String::from_utf8_lossy(&bytes).to_string())
}

fn games_part(screen: &str) -> &str {
    let opening = r#"<template data-part="games">"#;
    let start = screen
        .find(opening)
        .expect("the answer holds the games part")
        + opening.len();
    let end = screen[start..]
        .find("</template>")
        .expect("the games part ends");
    &screen[start..start + end]
}

fn saved_section(part: &str) -> &str {
    panel_section_html_slice(part, "Saved Games").expect("the part has a Saved Games section")
}

fn saved_names(part: &str) -> Vec<String> {
    saved_section(part)
        .split(r#"<span class="game-name">"#)
        .skip(1)
        .map(|rest| rest.split("</span>").next().unwrap_or_default().to_string())
        .collect()
}

/// The opening tag of the first element whose class list holds `class`.
fn tag_with_class<'a>(html: &'a str, class: &str) -> &'a str {
    let at = html
        .find(class)
        .unwrap_or_else(|| panic!("no element has the class {class}: {html}"));
    let start = html[..at].rfind('<').expect("a tag opens before its class");
    let end = at + html[at..].find('>').expect("the tag closes");
    &html[start..=end]
}

fn pager_position(part: &str) -> String {
    let rest = part
        .split(r#"class="games-page-position">"#)
        .nth(1)
        .unwrap_or_else(|| panic!("the part has a page position: {part}"));
    rest.split("</span>").next().unwrap_or_default().to_string()
}

async fn games_page(app: &axum::Router, query: &str) -> String {
    let (status, screen) = get(app, &format!("/screen?parts=games{query}")).await;
    assert_eq!(status, StatusCode::OK, "{screen}");
    games_part(&screen).to_string()
}

// [docs/specs/games.md] SCENARIO: 20.41
#[tokio::test]
async fn test_the_first_page_lists_50_saved_games_and_offers_the_next_page() {
    let (app, _ids) = app_with_saved_games(PAGE_SIZE + 1);

    let part = games_page(&app, "").await;

    let names = saved_names(&part);
    assert_eq!(names.len(), PAGE_SIZE, "page 1 lists 50 saved Games");
    assert_eq!(
        names.first().map(String::as_str),
        Some("Saga 51"),
        "the newest first"
    );
    assert_eq!(names.last().map(String::as_str), Some("Saga 02"));
    let active_section = panel_section_html_slice(&part, "Active Game")
        .expect("the part has an Active Game section");
    assert!(
        active_section.contains(r#"class="game-item active""#),
        "the Active Game section holds the active Game: {active_section}"
    );
    assert!(
        !saved_section(&part).contains(r#"game-item active"#),
        "the active Game is not in the Saved Games section"
    );
    assert_eq!(pager_position(&part), "Page 1 of 2");
    let previous = tag_with_class(&part, "games-page-previous");
    assert!(
        previous.contains("disabled"),
        "Previous is disabled on page 1: {previous}"
    );
    let next = tag_with_class(&part, "games-page-next");
    assert!(
        !next.contains("disabled") && next.contains(r#"data-page="2""#),
        "Next is enabled and goes to page 2: {next}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.42
#[tokio::test]
async fn test_the_last_page_lists_the_games_that_remain_and_offers_the_previous_page() {
    let (app, _ids) = app_with_saved_games(PAGE_SIZE + 1);

    let part = games_page(&app, "&games_page=2").await;

    assert_eq!(
        saved_names(&part),
        ["Saga 01"],
        "the oldest Game is alone on page 2"
    );
    assert_eq!(pager_position(&part), "Page 2 of 2");
    let next = tag_with_class(&part, "games-page-next");
    assert!(
        next.contains("disabled"),
        "Next is disabled on the last page: {next}"
    );
    let previous = tag_with_class(&part, "games-page-previous");
    assert!(
        !previous.contains("disabled") && previous.contains(r#"data-page="1""#),
        "Previous is enabled and goes to page 1: {previous}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.43
#[tokio::test]
async fn test_a_game_moves_to_the_first_page_when_a_write_updates_it() {
    let (app, ids) = app_with_saved_games(PAGE_SIZE + 1);
    let oldest = ids[0];
    assert_eq!(
        saved_names(&games_page(&app, "&games_page=2").await),
        ["Saga 01"]
    );

    let response = post_form(
        &app,
        &format!("/games/{oldest}/rename"),
        "display_name=Renamed+Oldest",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let first_page = saved_names(&games_page(&app, "").await);
    assert_eq!(
        first_page.first().map(String::as_str),
        Some("Renamed Oldest")
    );
    assert_eq!(
        saved_names(&games_page(&app, "&games_page=2").await),
        ["Saga 02"]
    );
}

// [docs/specs/games.md] SCENARIO: 20.44
#[tokio::test]
async fn test_a_page_past_the_end_shows_the_last_page() {
    let (app, _ids) = app_with_saved_games(PAGE_SIZE + 1);

    let part = games_page(&app, "&games_page=9").await;

    assert_eq!(pager_position(&part), "Page 2 of 2");
    assert_eq!(saved_names(&part).len(), 1);
}

// [docs/specs/games.md] SCENARIO: 20.45
#[tokio::test]
async fn test_a_list_of_50_saved_games_is_one_page_with_no_pager() {
    let (app, _ids) = app_with_saved_games(PAGE_SIZE);

    let part = games_page(&app, "").await;

    assert_eq!(saved_names(&part).len(), PAGE_SIZE);
    assert!(
        !part.contains("games-pager"),
        "one page shows no pager: {part}"
    );
}

// [docs/specs/screen.md] SCENARIO: 45.15
#[tokio::test]
async fn test_a_games_page_that_is_not_a_page_number_answers_400_and_renders_nothing() {
    let (app, _ids) = app_with_saved_games(1);

    for value in ["0", "-1", "two", ""] {
        let (status, body) = get(&app, &format!("/screen?parts=games&games_page={value}")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "games_page={value:?}");
        assert!(
            body.contains("games_page"),
            "games_page={value:?}: the body names it: {body}"
        );
        assert!(
            !body.contains("<template"),
            "games_page={value:?}: no part: {body}"
        );
    }
}
