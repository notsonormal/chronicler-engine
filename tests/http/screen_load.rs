//! HTTP E2E tests for the screen load (`GET /screen?parts=`): the templates it answers, its stamp and Game, its refusals, and the failure of one part. Tagged against `docs/specs/screen.md`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::adapters::driving::http::screen::handlers::screen_part_names;
use chronicler_engine::test_support::{active_game_of, empty_sqlite_storage, TestAppBuilder};

use crate::support::app_wiring::two_game_app;
use crate::support::http_requests::{post_form, response_body, templates_of};
use crate::SettingsTestGuard;

/// The parts that have a route of their own. The two action control parts have none.
const PART_ROUTES: [(&str, &str); 11] = [
    ("header", "/fragment/header"),
    ("failure-banner", "/fragment/failure-banner"),
    ("story-log", "/fragment/story-log"),
    ("visual-sidebar", "/fragment/visual-sidebar"),
    ("options-dock", "/fragment/options-dock"),
    ("status-display", "/status/busy"),
    ("settings", "/fragment/settings"),
    ("prompt-presets", "/fragment/prompt-presets"),
    ("worlds", "/fragment/worlds"),
    ("games", "/fragment/games"),
    ("llm-messages", "/fragment/llm-messages"),
];

const ACTION_CONTROL_PARTS: [&str; 2] = ["send-control", "autoplay-controls"];

async fn get(app: &axum::Router, uri: &str) -> axum::response::Response<Body> {
    let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    app.clone().oneshot(request).await.unwrap()
}

/// The stamp of `GET /changes`, in the form of `X-Revision`.
async fn stamp_of_changes(app: &axum::Router) -> String {
    let answer: serde_json::Value =
        serde_json::from_str(&response_body(get(app, "/changes").await).await).unwrap();
    format!(
        "{}:{}",
        answer["server"].as_str().unwrap(),
        answer["revision"].as_u64().unwrap()
    )
}

fn header_text(response: &axum::response::Response<Body>, name: &str) -> String {
    response
        .headers()
        .get(name)
        .unwrap_or_else(|| panic!("the answer carries {name}"))
        .to_str()
        .unwrap()
        .to_string()
}

fn revision_of(stamp: &str) -> u64 {
    stamp.rsplit_once(':').unwrap().1.parse().unwrap()
}

// [docs/specs/screen.md] SCENARIO: 45.8
#[tokio::test]
async fn test_a_screen_load_answers_one_template_for_each_named_part_in_the_order_asked() {
    let (app, state, _storage, _second) = two_game_app();

    let response = get(&app, "/screen?parts=status-display,header,story-log").await;

    assert_eq!(response.status(), StatusCode::OK);
    let stamp = header_text(&response, "x-revision");
    assert!(
        stamp.rsplit_once(':').is_some_and(|(server, revision)| {
            !server.is_empty() && revision.parse::<u64>().is_ok()
        }),
        "X-Revision has the form <server>:<revision>, got {stamp:?}"
    );
    assert_eq!(
        header_text(&response, "x-game"),
        active_game_of(&state).0.to_string()
    );
    let names: Vec<String> = templates_of(&response_body(response).await)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, ["status-display", "header", "story-log"]);
}

// [docs/specs/screen.md] SCENARIO: 45.9
#[tokio::test]
async fn test_a_part_of_the_screen_equals_the_answer_of_its_own_part_route() {
    let (app, _state, _storage, _second) = two_game_app();
    let named: Vec<&str> = screen_part_names()
        .filter(|name| !ACTION_CONTROL_PARTS.contains(name))
        .collect();
    assert_eq!(
        named,
        PART_ROUTES.map(|(name, _)| name),
        "the table of this test names every part that has a route, in the order of SCREEN_PARTS"
    );

    for (name, route) in PART_ROUTES {
        let screen = response_body(get(&app, &format!("/screen?parts={name}")).await).await;
        let template = templates_of(&screen);
        assert_eq!(template.len(), 1, "{name}: one template");
        let own = response_body(get(&app, route).await).await;
        assert_eq!(
            template[0].1, own,
            "{name}: the screen part is the body of {route}"
        );
    }
}

// [docs/specs/screen.md] SCENARIO: 45.10
#[tokio::test]
async fn test_a_missing_unknown_or_repeated_part_name_answers_400_and_renders_nothing() {
    let (app, _state, _storage, _second) = two_game_app();
    let before = stamp_of_changes(&app).await;

    for uri in [
        "/screen",
        "/screen?parts=",
        "/screen?parts=story-log,not-a-part",
        "/screen?parts=header,story-log,header",
    ] {
        let response = get(&app, uri).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        let body = response_body(response).await;
        assert!(!body.contains("<template"), "{uri} renders no part: {body}");
    }
    assert_eq!(stamp_of_changes(&app).await, before);
}

// [docs/specs/screen.md] SCENARIO: 45.11
#[tokio::test]
async fn test_one_part_that_fails_to_render_fails_the_whole_screen() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(empty_sqlite_storage());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();
    storage.add_failure(
        StorageCall::GetSettings,
        TestOverride::internal("settings read failure"),
    );

    let response = get(&app, "/screen?parts=header,settings").await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response_body(response).await;
    assert!(
        body.contains("settings read failure"),
        "the body names the failure: {body}"
    );
    assert!(!body.contains("<template"), "no part is answered: {body}");

    let header_only = get(&app, "/screen?parts=header").await;
    assert_eq!(header_only.status(), StatusCode::OK);
}

// [docs/specs/screen.md] SCENARIO: 45.16
#[tokio::test]
async fn test_a_part_whose_read_fails_answers_the_failure_of_that_read_not_a_configuration_error() {
    let _guard = SettingsTestGuard::new();
    for part in ["failure-banner", "settings", "llm-messages"] {
        let storage = Arc::new(empty_sqlite_storage());
        let app = TestAppBuilder::default_test()
            .storage(Arc::clone(&storage))
            .build();
        storage.add_failure(
            StorageCall::ListLatestLlmMessages,
            TestOverride::internal("messages read failure"),
        );

        let response = get(&app, &format!("/screen?parts={part}")).await;

        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{part}"
        );
        let body = response_body(response).await;
        assert!(
            body.contains("messages read failure"),
            "{part}: the body names the failure: {body}"
        );
        assert!(
            !body.contains("Configuration error"),
            "{part}: a read failure is not a configuration error: {body}"
        );
    }
}

// [docs/specs/screen.md] SCENARIO: 45.14
#[tokio::test]
async fn test_the_worlds_part_fails_when_the_games_that_it_counts_cannot_be_read() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(empty_sqlite_storage());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();
    storage.add_failure(
        StorageCall::ListGames,
        TestOverride::internal("games read failure"),
    );

    let response = get(&app, "/screen?parts=worlds").await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response_body(response).await;
    assert!(
        body.contains("games read failure"),
        "the body names the failure: {body}"
    );
    assert!(!body.contains("<template"), "no part is answered: {body}");
}

// [docs/specs/screen.md] SCENARIO: 45.12
#[tokio::test]
async fn test_a_screen_carries_the_revision_of_the_changes_route_and_a_write_moves_it() {
    let (app, state, _storage, _second) = two_game_app();
    let active = active_game_of(&state).0;

    let first = get(&app, "/screen?parts=header").await;
    let first_stamp = header_text(&first, "x-revision");
    assert_eq!(first_stamp, stamp_of_changes(&app).await);

    let renamed = post_form(
        &app,
        &format!("/games/{active}/rename"),
        "display_name=Renamed+Realm",
    )
    .await;
    assert_eq!(renamed.status(), StatusCode::OK);
    let second = get(&app, "/screen?parts=header").await;
    let second_stamp = header_text(&second, "x-revision");
    assert!(
        revision_of(&second_stamp) > revision_of(&first_stamp),
        "a write moves the revision: {first_stamp} then {second_stamp}"
    );
    let header = templates_of(&response_body(second).await);
    assert!(
        header[0].1.contains("Renamed Realm"),
        "the header shows the new name: {:?}",
        header[0].1
    );
}

// [docs/specs/screen.md] SCENARIO: 45.13
#[tokio::test]
async fn test_with_no_active_game_the_game_parts_render_empty_and_the_others_render() {
    let app = TestAppBuilder::default_test()
        .storage(Arc::new(empty_sqlite_storage()))
        .skip_seeding(true)
        .build();

    let response = get(
        &app,
        "/screen?parts=header,story-log,visual-sidebar,options-dock,status-display,games",
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header_text(&response, "x-game"), "");
    let parts = templates_of(&response_body(response).await);
    let content = |name: &str| {
        parts
            .iter()
            .find(|(part, _)| part == name)
            .unwrap_or_else(|| panic!("the screen answers {name}"))
            .1
            .clone()
    };
    assert!(content("header").contains("No active game"));
    for empty in ["story-log", "visual-sidebar", "options-dock"] {
        assert_eq!(content(empty), "", "{empty} renders empty");
    }
    assert!(content("status-display").contains("Ready"));
    assert!(
        content("games").contains("New Game"),
        "the Games panel still offers a new Game: {}",
        content("games")
    );
}
