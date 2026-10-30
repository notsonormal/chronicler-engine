//! HTTP integration tests for fragment rendering.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, empty_sqlite_storage, ScriptedModel,
};
use chronicler_engine::TestAppBuilder;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::adapters::driving::http::AppState;

use crate::support::app_wiring::app_with_model;
use crate::support::http_requests::{
    fetch_answer, fetch_body, fetch_part, post_action, post_empty, wait_settled,
    post_on_last_message,
};
use crate::TEST_PERSONA;

fn one_turn_app() -> (axum::Router, AppState) {
    TestAppBuilder::default_test()
        .model(Arc::new(ScriptedModel::new().turn("You look around.")))
        .build_with_state()
}

#[tokio::test]
async fn test_basic_fragments_return_html() {
    // The fragment renders entries only — the shell owns `#story-log` — so it needs
    // a seeded entry.
    let app = TestAppBuilder::default_test().build();
    let fragments = [
        ("header", "class=\"game-title\""),
        ("story-log", "class=\"log-entry"),
        ("visual-sidebar", "id=\"visual-sidebar\""),
    ];
    for (part, expected) in fragments {
        let body = fetch_part(&app, part).await;
        assert!(
            body.contains(expected),
            "Fragment {part} should contain '{expected}'"
        );
    }
}

#[tokio::test]
async fn test_visual_sidebar_renders_room_image() {
    let body = fetch_part(&TestAppBuilder::default_app(), "visual-sidebar").await;
    assert!(
        body.contains("data/images/test_room.png"),
        "Expected room image in sidebar: {body}"
    );
    assert!(
        !body.contains("No Room Image"),
        "Should not show placeholder when image exists: {body}"
    );
}

#[tokio::test]
async fn test_story_log_fragment_renders_action_buttons_after_actions() {
    let (app, state) = {
        let model = Arc::new(
            ScriptedModel::new()
                .turn("First.")
                .narration("Second.")
                .after_narration(analysis_report(&[], None)),
        );
        app_with_model(model)
    };

    let resp = post_action(&app, "first").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "first action should complete"
    );
    let resp = post_action(&app, "second").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "second action should complete"
    );

    let body = fetch_part(&app, "story-log").await;
    assert!(
        body.contains("edit-btn"),
        "story-log should render edit button: {body}"
    );
    assert!(
        body.contains("delete-btn"),
        "story-log should render delete button: {body}"
    );
    assert!(
        !body.contains("retry-btn"),
        "story-log should not render retry button: {body}"
    );
    let delete_btn_count = body.matches("delete-btn").count();
    assert!(
        delete_btn_count <= 2,
        "at most 2 delete buttons (last narration + last input), got {delete_btn_count}: {body}"
    );
}

#[tokio::test]
async fn test_header_fragment_displays_game_title() {
    let body = fetch_part(&TestAppBuilder::default_app(), "header").await;
    assert!(
        body.contains("game-title"),
        "header should have .game-title element: {body}"
    );
    assert!(
        body.contains("Chronicler Engine"),
        "header should display game title: {body}"
    );
}

#[tokio::test]
async fn test_action_handler_accepts_command() {
    let (app, state) = one_turn_app();

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.is_empty(),
        "an accepted command answers nothing: {body_str}"
    );
    assert!(
        wait_settled(&state, 1000).await,
        "the accepted Action's Job should settle"
    );
}

#[tokio::test]
async fn test_action_handler_empty_command() {
    let (app, state) = one_turn_app();

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.is_empty(),
        "Expected empty command to trigger continuation (a dispatched command answers nothing): {body_str}"
    );
    assert!(
        wait_settled(&state, 1000).await,
        "the continuation Job should settle"
    );
}

#[tokio::test]
async fn test_character_headshots_fragment() {
    let body = fetch_body(
        &TestAppBuilder::default_app(),
        "/fragment/character-headshots",
    )
    .await;

    assert!(
        body.contains("headshot"),
        "Expected headshot in fragment: {body}"
    );
}

#[tokio::test]
async fn test_reset_busy_handler() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/status/reset-busy")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("reset"));
}

#[tokio::test]
async fn test_edit_history_handler_success() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let entry_id = state
        .game_view_query
        .story_messages(active_game_of(&state))
        .unwrap()
        .last()
        .unwrap()
        .id;

    let req = Request::builder()
        .uri(format!("/history/{entry_id}"))
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited+text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.is_empty(), "an edit answers nothing: {body_str}");
}

#[tokio::test]
async fn test_edit_history_handler_not_found() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/history/999")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "a Message the story does not hold is a refused edit"
    );
}

#[tokio::test]
async fn test_delete_history_handler_success() {
    // The story keeps its opening, so the Message the delete removes is the Job's
    // Narration.
    let (app, state) = one_turn_app();
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 1000).await, "the turn should settle");
    assert!(
        fetch_part(&app, "story-log")
            .await
            .contains("You look around.")
    );

    let response = post_on_last_message(&app, "/history/delete").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = fetch_part(&app, "story-log").await;
    assert!(
        !body.contains("You look around."),
        "Log entry should be deleted from rendered story log"
    );
}

#[tokio::test]
async fn test_delete_history_handler_empty() {
    // A Game holds its opening Narration, and the delete keeps the last Message.
    let app = TestAppBuilder::default_app();

    let response = post_on_last_message(&app, "/history/delete").await;

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "a story that holds only its opening Narration refuses the delete"
    );
}

#[tokio::test]
async fn test_action_confirm_empty_command() {
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(ScriptedModel::new().turn("The scene continues.")))
        .build_with_state();

    let req = Request::builder()
        .uri("/action/confirm")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "the continuation Job should settle"
    );
}

#[tokio::test]
async fn test_action_async_inventory() {
    let (app, state) = one_turn_app();

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=inventory"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let hx_trigger = response.headers().get("HX-Trigger");
    assert!(hx_trigger.is_none(), "Inventory should be async, not sync");
    assert!(
        wait_settled(&state, 1000).await,
        "the inventory Action's Job should settle"
    );
}

#[tokio::test]
async fn test_list_games_fragment_populated() {
    let app = TestAppBuilder::default_app();
    let world_key = "test"; // Default test app world key

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key={world_key}&persona_key={TEST_PERSONA}"
        )))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key={world_key}&persona_key={TEST_PERSONA}"
        )))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let html = fetch_part(&app, "games").await;
    assert!(
        html.contains("game-item"),
        "Should render game item: {html}"
    );
    assert!(html.contains("Switch"), "Should have switch button: {html}");
    assert!(html.contains("Delete"), "Should have delete button: {html}");
}

#[tokio::test]
async fn test_list_games_fragment_escapes_html() {
    let storage = Arc::new(empty_sqlite_storage());

    let _ = chronicler_engine::test_support::seed_game_row(
        &storage,
        &chronicler_engine::test_support::test_new_game(
            "Test World",
            "Test World",
            "test_player",
            "Test Player",
            "<script>alert('xss')</script>",
        ),
    );

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let html = fetch_part(&app, "games").await;
    assert!(
        !html.contains("<script>"),
        "Should escape HTML in game name: {html}"
    );

    assert!(
        html.contains("&#60;script&#62;") || html.contains("&lt;script&gt;"),
        "Should contain escaped script tag: {html}"
    );
}

#[tokio::test]
async fn test_list_games_fragment_storage_error() {
    let storage = Arc::new(empty_sqlite_storage().with_failure(
        StorageCall::ListSavedGames,
        TestOverride::config("list_games failed"),
    ));

    let app = TestAppBuilder::default_test().storage(storage).build();

    let (status, _body) = fetch_answer(&app, "/screen?parts=games").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_list_games_fragment_shows_world_badge() {
    let storage = Arc::new(empty_sqlite_storage());
    let world_a = chronicler_engine::test_support::TestWorld::minimal();
    let map_a = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_a, &map_a).unwrap();
    let player = chronicler_engine::test_support::TestPersona::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let mut world_b = chronicler_engine::test_support::TestWorld::minimal();
    world_b.key = "world_b_test".to_string();
    world_b.name = "World B Test".to_string();
    let map_b = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_b, &map_b).unwrap();
    storage.seed_persona(&player.key, &player).unwrap();

    let _game_a = chronicler_engine::test_support::seed_game_row(
        &storage,
        &chronicler_engine::test_support::test_new_game(
            &world_a.name,
            &world_a.key,
            &player.key,
            &player.sheet.name,
            "Game A",
        ),
    );
    let _game_b = chronicler_engine::test_support::seed_game_row(
        &storage,
        &chronicler_engine::test_support::test_new_game(
            &world_b.name,
            &world_b.key,
            &player.key,
            &player.sheet.name,
            "Game B",
        ),
    );

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let html = fetch_part(&app, "games").await;
    assert!(
        html.contains("world-badge"),
        "Should render world-badge class: {html}"
    );
    assert!(
        html.contains(&world_a.name),
        "Should show world A name: {html}"
    );
    assert!(
        html.contains(&world_b.name),
        "Should show world B name: {html}"
    );
}

#[tokio::test]
async fn test_create_game_with_world_key() {
    let storage = Arc::new(empty_sqlite_storage());
    let world_a = chronicler_engine::test_support::TestWorld::minimal();
    let map_a = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_a, &map_a).unwrap();
    let player = chronicler_engine::test_support::TestPersona::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key={}&persona_key={}",
            world_a.key, player.key
        )))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers().get("HX-Refresh").is_none(),
        "a create must not reload the page"
    );

    let games = storage.list_games().unwrap();

    for game in &games {
        assert_eq!(
            game.world_key, world_a.key,
            "All games should be in world '{}'",
            world_a.key
        );
    }
    assert!(!games.is_empty(), "Should have created at least one game");
}

#[tokio::test]
async fn test_create_game_with_invalid_world_key() {
    let storage = Arc::new(empty_sqlite_storage());
    let world_a = chronicler_engine::test_support::TestWorld::minimal();
    let map_a = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_a, &map_a).unwrap();
    let player = chronicler_engine::test_support::TestPersona::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key=nonexistent_world&persona_key={TEST_PERSONA}",
        )))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should return bad_request for non-existent world"
    );
}

#[tokio::test]
async fn test_switch_swipe_handler_success() {
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(
            ScriptedModel::new()
                .turn("First.")
                .narration("Second.")
                .after_narration(analysis_report(&[], None)),
        ))
        .build_with_state();

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the Action should complete"
    );
    assert!(
        post_on_last_message(&app, "/swipe/new")
            .await
            .status()
            .is_success()
    );
    assert!(
        wait_settled(&state, 1000).await,
        "the Retry should complete"
    );

    let game = active_game_of(&state);
    let entries: Vec<_> = state
        .game_view_query
        .story_window(game)
        .expect("the story reads back")
        .entries
        .into_iter()
        .map(|entry| entry.row)
        .collect();
    let last = entries.last().expect("the Retry's Narration is the last");
    assert_eq!(last.swipe_count, 2, "the Retry appended a Swipe");

    let response = post_empty(&app, &format!("/message/{}/swipe/0", last.id)).await;
    assert_eq!(response.status(), StatusCode::OK);

    let entries: Vec<_> = state
        .game_view_query
        .story_window(game)
        .expect("the story reads back")
        .entries
        .into_iter()
        .map(|entry| entry.row)
        .collect();
    assert_eq!(
        entries
            .last()
            .expect("the Narration stays last")
            .active_swipe_index,
        0,
        "the switch makes the first Swipe the active one"
    );
}

#[tokio::test]
async fn test_reset_handler_success() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers().get("HX-Refresh").is_none(),
        "a reset must not reload the page"
    );
}

#[tokio::test]
async fn test_switch_swipe_handler_concurrent() {
    // The held narrator call is the busy state a swipe switch meets.
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::clone(&model))
        .build_with_state();

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held narrator call should reach the model"
    );
    let id = active_game_of(&state).0;

    let req = Request::builder()
        .uri(format!("/message/{id}/swipe/0"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CONFLICT,
        "a swipe switch while the Game is busy is refused with 409"
    );
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    assert!(
        String::from_utf8_lossy(&body).contains("Still thinking..."),
        "the answer carries the busy refusal line"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the released Job should complete"
    );
}

#[tokio::test]
async fn test_header_fragment_has_no_hardcoded_connection_status() {
    let body = fetch_part(&TestAppBuilder::default_app(), "header").await;
    assert!(
        !body.contains("id=\"connection-status\""),
        "the header no longer claims a live connection: {body}"
    );
    assert!(
        !body.contains("Connected"),
        "the hardcoded Connected signal is deleted: {body}"
    );
}
