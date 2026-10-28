//! HTTP E2E tests for what every action answer carries: the `X-Revision` stamp, the busy refusal as `409`, and the Game header that a page sends with a Game-bound action. Tagged against `docs/specs/screen.md`.

use std::collections::BTreeSet;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::utils::answer_words::REFUSAL_WORDS;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarratorMode};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{active_game_of, ScriptedModel, TestAppBuilder};

use crate::support::app_wiring::{app_with_model, two_game_app};
use crate::support::http_assertions::assert_still_thinking;
use crate::support::http_fixtures::story_of;
use crate::support::http_requests::{
    post_action, post_form, post_form_with_headers, response_body, wait_for_jobs, wait_settled,
};

const GAME_HEADER: &str = "x-chronicler-game";
const REVISION_HEADER: &str = "x-revision";
const STALE_GAME_LINE: &str = "The page showed a Game that is no longer active.";

async fn stamp_of_changes(app: &axum::Router) -> String {
    let request = Request::builder()
        .uri("/changes")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let answer: Value = serde_json::from_str(&response_body(response).await)
        .expect("GET /changes answers a JSON object");
    format!(
        "{}:{}",
        answer["server"]
            .as_str()
            .expect("the answer names the server"),
        answer["revision"]
            .as_u64()
            .expect("the answer holds the revision"),
    )
}

/// The stamp of the answer, which must have the form `<server>:<revision>`.
fn revision_header_of(response: &axum::response::Response<Body>, label: &str) -> String {
    let stamp = response
        .headers()
        .get(REVISION_HEADER)
        .unwrap_or_else(|| panic!("{label}: the answer carries X-Revision"))
        .to_str()
        .expect("X-Revision is text")
        .to_string();
    let (server, revision) = stamp
        .rsplit_once(':')
        .unwrap_or_else(|| panic!("{label}: the stamp is <server>:<revision>, got {stamp:?}"));
    assert!(
        !server.is_empty(),
        "{label}: the stamp names the server: {stamp:?}"
    );
    revision
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{label}: the revision is a whole number: {stamp:?}"));
    stamp
}

/// The path of each `POST` route in the router source, in the form the router
/// writes it (`/games/:id/rename`).
fn post_routes_of_router() -> BTreeSet<String> {
    let source = include_str!("../../src/adapters/driving/http/builders/router.rs");
    source
        .split(".route(")
        .skip(1)
        .filter_map(|call| {
            let mut parts = call.splitn(3, '"');
            parts.next()?;
            let path = parts.next()?;
            let rest = parts.next()?.trim_start_matches(',').trim_start();
            rest.starts_with("post(").then(|| path.to_string())
        })
        .collect()
}

/// The paths of the `game_bound_routes` function of the router source.
fn game_bound_routes_of_router() -> BTreeSet<&'static str> {
    let source = include_str!("../../src/adapters/driving/http/builders/router.rs");
    let start = source
        .find("fn game_bound_routes")
        .expect("the router builds its Game-bound routes in one function");
    let body = &source[start..];
    let end = body
        .find(".route_layer(")
        .expect("the Game layer follows the routes");
    body[..end]
        .split(".route(")
        .skip(1)
        .filter_map(|call| call.split('"').nth(1))
        .collect()
}

// [docs/specs/screen.md] SCENARIO: 45.1
#[tokio::test]
async fn test_every_action_route_answers_with_the_revision_stamp() {
    // Idle phase: a Game-bound or other write that starts no Job.
    let (app, state, _storage, second) = two_game_app();
    let active = active_game_of(&state).0;
    let message = story_of(&state).last().expect("a story").id;
    let idle_rows: Vec<(&str, String, String)> = vec![
        (
            "/action/check",
            "/action/check".into(),
            "command=look".into(),
        ),
        (
            "/message/:id/swipe/:index",
            format!("/message/{message}/swipe/0"),
            String::new(),
        ),
        (
            "/history/:id",
            format!("/history/{message}"),
            "text=Edited".into(),
        ),
        (
            "/games/:id/rename",
            format!("/games/{active}/rename"),
            "display_name=Renamed".into(),
        ),
        (
            "/games/:id/mode",
            format!("/games/{active}/mode"),
            "narrator_mode=novel".into(),
        ),
        (
            "/games/:id/posture",
            format!("/games/{active}/posture"),
            "narrative_perspective=second&narrative_tense=past".into(),
        ),
        (
            "/games/:id/presets",
            format!("/games/{active}/presets"),
            String::new(),
        ),
        ("/worlds", "/worlds".into(), String::new()),
        (
            "/worlds/:key",
            "/worlds/no-such-world".into(),
            String::new(),
        ),
        (
            "/worlds/:key/posture",
            "/worlds/no-such-world/posture".into(),
            String::new(),
        ),
        (
            "/worlds/:key/delete",
            "/worlds/no-such-world/delete".into(),
            String::new(),
        ),
        (
            "/connections/add",
            "/connections/add".into(),
            "conn_name=Alpha&conn_provider=mock&conn_model=m&conn_api_key=&conn_base_url=".into(),
        ),
        (
            "/connections/test",
            "/connections/test".into(),
            "conn_name=Alpha&conn_provider=mock&conn_model=m&conn_api_key=&conn_base_url=".into(),
        ),
        (
            "/connections/:id/test",
            "/connections/no-such/test".into(),
            String::new(),
        ),
        (
            "/connections/:id/edit",
            "/connections/no-such/edit".into(),
            String::new(),
        ),
        (
            "/connections/:id/delete",
            "/connections/no-such/delete".into(),
            String::new(),
        ),
        (
            "/connections/set-narrator",
            "/connections/set-narrator".into(),
            String::new(),
        ),
        (
            "/connections/set-quantifier",
            "/connections/set-quantifier".into(),
            String::new(),
        ),
        (
            "/settings/text-check",
            "/settings/text-check".into(),
            String::new(),
        ),
        ("/prompt-presets", "/prompt-presets".into(), String::new()),
        (
            "/prompt-presets/:id",
            "/prompt-presets/no-such".into(),
            String::new(),
        ),
        (
            "/prompt-presets/:id/delete",
            "/prompt-presets/no-such/delete".into(),
            String::new(),
        ),
        (
            "/prompt-presets/:id/duplicate",
            "/prompt-presets/no-such/duplicate".into(),
            String::new(),
        ),
        (
            "/prompt-presets/:id/activate",
            "/prompt-presets/no-such/activate".into(),
            String::new(),
        ),
        ("/history/delete", "/history/delete".into(), String::new()),
        (
            "/games/:id/switch",
            format!("/games/{second}/switch"),
            String::new(),
        ),
        (
            "/games/:id/delete",
            format!("/games/{second}/delete"),
            String::new(),
        ),
        (
            "/status/reset-busy",
            "/status/reset-busy".into(),
            String::new(),
        ),
        ("/reset", "/reset".into(), String::new()),
        ("/games", "/games".into(), String::new()),
    ];
    let mut covered: BTreeSet<String> = BTreeSet::new();
    for (route, uri, body) in &idle_rows {
        let response = post_form(&app, uri, body).await;
        revision_header_of(&response, uri);
        covered.insert((*route).to_string());
    }

    // Busy phase: a write that starts a Job, then each write the busy Game refuses.
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    let (busy_app, busy_state) = app_with_model(Arc::clone(&model));
    let started = post_action(&busy_app, "look").await;
    assert!(
        started.status().is_success(),
        "the first Action is accepted"
    );
    revision_header_of(&started, "POST /action that starts a Job");
    assert!(hold.wait_started(1000), "the Job reaches its narrator call");
    for (route, uri, body) in [
        ("/action", "/action", "command=listen"),
        ("/action/confirm", "/action/confirm", "command=listen"),
        ("/autoplay", "/autoplay", "cycles=1"),
        ("/swipe/new", "/swipe/new", ""),
        ("/retrigger", "/retrigger", ""),
    ] {
        let response = post_form(&busy_app, uri, body).await;
        assert_eq!(
            response.status(),
            StatusCode::CONFLICT,
            "POST {uri} is refused while the Job runs"
        );
        revision_header_of(&response, uri);
        covered.insert(route.to_string());
    }
    hold.release();
    assert!(wait_settled(&busy_state, 2000).await, "the held Job ends");

    let every_post_route = post_routes_of_router();
    assert!(
        every_post_route.len() > 30,
        "the router scan finds the POST routes: {every_post_route:?}"
    );
    let missing: Vec<_> = every_post_route.difference(&covered).collect();
    assert!(
        missing.is_empty(),
        "every POST route is in the table; missing {missing:?}"
    );
}

// [docs/specs/screen.md] SCENARIO: 45.2
#[tokio::test]
async fn test_the_revision_stamp_of_an_answer_is_the_stamp_after_its_write() {
    let (app, state, _storage, second) = two_game_app();
    let active = active_game_of(&state).0;

    let response = post_form(
        &app,
        &format!("/games/{active}/rename"),
        "display_name=Renamed",
    )
    .await;
    assert!(response.status().is_success());
    let stamp = revision_header_of(&response, "rename");
    assert_eq!(
        stamp,
        stamp_of_changes(&app).await,
        "a write answers the stamp that GET /changes reads right after it"
    );

    let before = stamp_of_changes(&app).await;
    let nothing_written = post_form(&app, &format!("/games/{active}/switch"), "").await;
    assert!(nothing_written.status().is_success());
    assert_eq!(
        revision_header_of(&nothing_written, "switch to the active Game"),
        before,
        "an action that wrote nothing answers the current stamp"
    );

    let refused = post_form(&app, &format!("/games/{second}/rename"), "display_name=").await;
    assert!(!refused.status().is_success(), "an empty name is refused");
    assert_eq!(
        revision_header_of(&refused, "a refused rename"),
        stamp_of_changes(&app).await,
        "a refusal also answers the current stamp"
    );
}

// [docs/specs/screen.md] SCENARIO: 45.3
#[tokio::test]
async fn test_a_busy_refusal_answers_409_with_the_wait_line_and_writes_nothing() {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    let (app, state) = app_with_model(Arc::clone(&model));
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(hold.wait_started(1000), "the Job reaches its narrator call");
    let before = stamp_of_changes(&app).await;

    let refused = post_action(&app, "listen").await;
    assert_eq!(refused.status(), StatusCode::CONFLICT);
    assert_eq!(revision_header_of(&refused, "the busy refusal"), before);
    assert_eq!(
        refused
            .headers()
            .get("x-refusal")
            .map(|value| value.as_bytes()),
        Some(b"busy".as_slice()),
        "the refusal names its kind"
    );
    let body = response_body(refused).await;
    assert_still_thinking(&body);
    assert_eq!(
        error_lines_of("the busy refusal", &body),
        [
            "The Game is busy, so the Action was not sent.",
            "Still thinking..."
        ],
        "the first line says what was not done, the second is the engine's line"
    );
    assert!(
        !body.contains("<span"),
        "the refusal carries no status markup: {body}"
    );
    assert_eq!(
        stamp_of_changes(&app).await,
        before,
        "the refusal wrote nothing"
    );

    hold.release();
    assert!(wait_settled(&state, 2000).await, "the held Job ends");
}

/// The Game-bound routes: every `POST` route whose handler acts on the active Game.
fn game_bound_rows(message: u64) -> Vec<(&'static str, String, &'static str)> {
    vec![
        ("/action", "/action".to_string(), "command=look"),
        ("/action/check", "/action/check".to_string(), "command=look"),
        (
            "/action/confirm",
            "/action/confirm".to_string(),
            "command=look",
        ),
        ("/autoplay", "/autoplay".to_string(), "cycles=1"),
        ("/swipe/new", "/swipe/new".to_string(), ""),
        ("/retrigger", "/retrigger".to_string(), ""),
        ("/reset", "/reset".to_string(), ""),
        (
            "/message/:id/swipe/:index",
            format!("/message/{message}/swipe/0"),
            "",
        ),
        ("/history/:id", format!("/history/{message}"), "text=Edited"),
        ("/history/delete", "/history/delete".to_string(), ""),
        ("/status/reset-busy", "/status/reset-busy".to_string(), ""),
    ]
}

// [docs/specs/screen.md] SCENARIO: 45.4
#[tokio::test]
async fn test_a_stale_game_header_is_refused_on_each_game_bound_route() {
    let (app, state, _storage, second) = two_game_app();
    let message = story_of(&state).last().expect("a story").id;
    let story_before = story_of(&state);
    let stamp_before = stamp_of_changes(&app).await;
    let stale = second.to_string();

    let rows = game_bound_rows(message);
    assert_eq!(rows.len(), 11, "the router has 11 Game-bound routes");
    assert_eq!(
        rows.iter()
            .map(|(route, _, _)| *route)
            .collect::<BTreeSet<_>>(),
        game_bound_routes_of_router(),
        "the table holds the routes that the router puts behind the Game layer"
    );
    assert_eq!(
        crate::test_utils::stub_action_layers::GAME_BOUND_ROUTES
            .into_iter()
            .collect::<BTreeSet<_>>(),
        game_bound_routes_of_router(),
        "the stub server refuses a stale Game on the routes that the router does"
    );
    for (route, uri, body) in rows {
        let response =
            post_form_with_headers(&app, &uri, body, &[(GAME_HEADER, stale.as_str())]).await;
        assert_eq!(
            response.status(),
            StatusCode::CONFLICT,
            "POST {route} with a stale Game header is refused"
        );
        revision_header_of(&response, route);
        assert_eq!(
            response
                .headers()
                .get("x-refusal")
                .map(|value| value.as_bytes()),
            Some(b"game-changed".as_slice()),
            "POST {route}: the refusal names its kind"
        );
        let text = response_body(response).await;
        let lines = error_lines_of(route, &text);
        assert!(
            lines[0].starts_with("The active Game changed, so "),
            "POST {route} says why: {text}"
        );
        assert_eq!(
            lines[1], STALE_GAME_LINE,
            "POST {route} carries the engine's line"
        );
        assert_eq!(
            stamp_of_changes(&app).await,
            stamp_before,
            "POST {route}: the refusal wrote nothing"
        );
    }
    assert_eq!(
        story_of(&state).len(),
        story_before.len(),
        "no refused request touched the story"
    );
}

// [docs/specs/screen.md] SCENARIO: 45.5
#[tokio::test]
async fn test_a_stale_stop_does_not_stop_the_job_of_the_active_game() {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    let (app, state) = app_with_model(Arc::clone(&model));
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(hold.wait_started(1000), "the Job reaches its narrator call");
    let stale = (active_game_of(&state).0 + 1000).to_string();

    let refused = post_form_with_headers(
        &app,
        "/status/reset-busy",
        "",
        &[(GAME_HEADER, stale.as_str())],
    )
    .await;
    assert_eq!(refused.status(), StatusCode::CONFLICT);

    hold.release();
    assert!(wait_settled(&state, 2000).await, "the Job ends by itself");
    let narrations = story_of(&state)
        .iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 2,
        "the Job was not stopped: the opening Narration and the Narration of the Action"
    );
}

// [docs/specs/screen.md] SCENARIO: 45.6
#[tokio::test]
async fn test_a_request_without_the_header_or_with_the_active_game_acts_on_the_active_game() {
    let (app, state, _storage, _second) = two_game_app();
    let active = active_game_of(&state).0.to_string();
    let message = story_of(&state).last().expect("a story").id;

    let without = post_form(&app, &format!("/history/{message}"), "text=First").await;
    assert_eq!(
        without.status(),
        StatusCode::OK,
        "no header: the edit lands"
    );
    let with_active = post_form_with_headers(
        &app,
        &format!("/history/{message}"),
        "text=Second",
        &[(GAME_HEADER, active.as_str())],
    )
    .await;
    assert_eq!(
        with_active.status(),
        StatusCode::OK,
        "the active Game: the edit lands"
    );
    let entry = story_of(&state)
        .into_iter()
        .find(|entry| entry.id == message)
        .expect("the Message is still there");
    let text = entry.swipes[entry.active_swipe_index].text.clone();
    assert_eq!(text, "Second");
}

// [docs/specs/screen.md] SCENARIO: 45.7
#[tokio::test]
async fn test_a_header_that_names_no_game_is_refused_and_a_read_ignores_the_header() {
    let (app, state, _storage, _second) = two_game_app();
    let message = story_of(&state).last().expect("a story").id;
    let stamp_before = stamp_of_changes(&app).await;

    let refused = post_form_with_headers(
        &app,
        &format!("/history/{message}"),
        "text=Edited",
        &[(GAME_HEADER, "not-a-game")],
    )
    .await;
    assert_eq!(refused.status(), StatusCode::CONFLICT);
    assert_eq!(stamp_of_changes(&app).await, stamp_before);

    let request = Request::builder()
        .uri("/fragment/story-log")
        .header(GAME_HEADER, "999999")
        .body(Body::empty())
        .unwrap();
    let read = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        read.status(),
        StatusCode::OK,
        "a read carries no Game check"
    );

    let no_game_app = TestAppBuilder::default_test()
        .storage(Arc::new(
            chronicler_engine::test_support::empty_sqlite_storage(),
        ))
        .skip_seeding(true)
        .build();
    let refused =
        post_form_with_headers(&no_game_app, "/history/delete", "", &[(GAME_HEADER, "1")]).await;
    assert_eq!(
        refused.status(),
        StatusCode::CONFLICT,
        "a header names a Game and no Game is active"
    );
}

/// An error body is one `error-message` element whose text has no `Error: ` prefix.
fn error_text_of(label: &str, body: &str) -> String {
    const OPEN: &str = "<div class=\"error-message\">";
    let text = body
        .strip_prefix(OPEN)
        .and_then(|rest| rest.strip_suffix("</div>"))
        .unwrap_or_else(|| panic!("{label}: the body is one error-message element: {body}"));
    assert!(
        !text.starts_with("Error: "),
        "{label}: the text carries no prefix: {text}"
    );
    text.to_string()
}

async fn get_status_and_body(app: &axum::Router, uri: &str) -> (StatusCode, String) {
    let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    (status, response_body(response).await)
}

// [docs/specs/screen.md] SCENARIO: 45.18
#[tokio::test]
async fn test_each_kind_of_failed_action_answers_its_status_and_one_body() {
    let (app, state, _storage, second) = two_game_app();
    let active = active_game_of(&state).0;

    let invalid_value = post_form(&app, &format!("/games/{active}/rename"), "display_name=").await;
    assert_eq!(
        invalid_value.status(),
        StatusCode::BAD_REQUEST,
        "an invalid value"
    );
    error_text_of("an invalid value", &response_body(invalid_value).await);

    let gone_by_post = post_form(&app, "/history/999999", "text=Changed.").await;
    assert_eq!(
        gone_by_post.status(),
        StatusCode::BAD_REQUEST,
        "a POST that names a gone Message"
    );
    let text = error_text_of("a gone Message", &response_body(gone_by_post).await);
    assert!(text.contains("Message not found"), "{text}");

    for uri in [
        "/fragment/games/999999/rename",
        "/fragment/prompt-presets/no-such-preset/edit",
        "/fragment/prompt-presets/no-such-preset/view",
    ] {
        let (status, body) = get_status_and_body(&app, uri).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "GET {uri}: an opener for a gone record"
        );
        error_text_of(uri, &body);
    }

    let stale = second.to_string();
    let refused =
        post_form_with_headers(&app, "/reset", "", &[(GAME_HEADER, stale.as_str())]).await;
    assert_eq!(refused.status(), StatusCode::CONFLICT, "a refusal");
    let text = error_text_of("a refusal", &response_body(refused).await);
    assert!(text.contains("The active Game changed"), "{text}");

    let (world_app, _state) = TestAppBuilder::default_test().build_with_state();
    let state_refusal = post_form(&world_app, "/worlds/test/delete", "").await;
    assert_eq!(
        state_refusal.status(),
        StatusCode::BAD_REQUEST,
        "a World that still has Games"
    );
    let text = error_text_of("a World with Games", &response_body(state_refusal).await);
    assert!(text.contains("Cannot delete world with 1 game"), "{text}");

    let (failing_storage, failures) =
        chronicler_engine::test_support::empty_sqlite_storage().with_test_failures();
    let (failing_app, _state) = TestAppBuilder::default_test()
        .storage(Arc::new(failing_storage))
        .build_with_state();
    failures.set(
        chronicler_engine::adapters::driven::storage::StorageCall::SavePreset,
        chronicler_engine::adapters::driven::storage::TestOverride::internal("preset save failure"),
    );
    let failed = post_form(
        &failing_app,
        "/prompt-presets",
        "name=Fail+Preset&instructions=Will+fail.&preset_type=system",
    )
    .await;
    assert_eq!(
        failed.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "an engine failure"
    );
    let text = error_text_of("an engine failure", &response_body(failed).await);
    assert!(text.contains("preset save failure"), "{text}");
}

/// The lines of one error body: the sentence, then the engine's own text.
fn error_lines_of(label: &str, body: &str) -> Vec<String> {
    error_text_of(label, body)
        .split('\n')
        .map(str::to_string)
        .collect()
}

/// What the words table says was not done on one route, as a sentence part.
fn words_of(route: &str) -> String {
    let row = REFUSAL_WORDS
        .iter()
        .find(|row| row.route == route)
        .unwrap_or_else(|| panic!("the words table has a row for {route}"));
    format!("the {} {}", row.noun, row.predicate)
}

// [docs/specs/screen.md] SCENARIO: 45.17
#[tokio::test]
async fn test_a_refusal_names_what_was_not_done() {
    let (app, state, _storage, second) = two_game_app();
    let message = story_of(&state).last().expect("a story").id;
    let stale = second.to_string();
    for (route, uri, body) in game_bound_rows(message) {
        let response =
            post_form_with_headers(&app, &uri, body, &[(GAME_HEADER, stale.as_str())]).await;
        assert_eq!(response.status(), StatusCode::CONFLICT, "stale {route}");
        assert_eq!(
            error_lines_of(route, &response_body(response).await),
            [
                format!("The active Game changed, so {}.", words_of(route)),
                STALE_GAME_LINE.to_string()
            ],
            "stale {route}"
        );
    }

    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    let (busy_app, busy_state) = app_with_model(Arc::clone(&model));
    assert!(post_action(&busy_app, "look").await.status().is_success());
    assert!(hold.wait_started(1000), "the Job reaches its narrator call");
    let busy_message = story_of(&busy_state).last().expect("a story").id;
    let busy_rows = [
        ("/action", "/action".to_string(), "command=listen"),
        (
            "/action/check",
            "/action/check".to_string(),
            "command=/options",
        ),
        (
            "/action/confirm",
            "/action/confirm".to_string(),
            "command=listen",
        ),
        ("/autoplay", "/autoplay".to_string(), "cycles=1"),
        ("/swipe/new", "/swipe/new".to_string(), ""),
        ("/retrigger", "/retrigger".to_string(), ""),
        (
            "/history/:id",
            format!("/history/{busy_message}"),
            "text=Edited",
        ),
        ("/history/delete", "/history/delete".to_string(), ""),
        (
            "/message/:id/swipe/:index",
            format!("/message/{busy_message}/swipe/0"),
            "",
        ),
    ];
    for (route, uri, body) in busy_rows {
        let response = post_form(&busy_app, &uri, body).await;
        assert_eq!(response.status(), StatusCode::CONFLICT, "busy {route}");
        assert_eq!(
            error_lines_of(route, &response_body(response).await),
            [
                format!("The Game is busy, so {}.", words_of(route)),
                "Still thinking...".to_string()
            ],
            "busy {route}"
        );
    }
    hold.release();
    assert!(wait_settled(&busy_state, 2000).await, "the held Job ends");

    let (run_app, run_state, _storage, _second) = two_game_app();
    let game = active_game_of(&run_state);
    let pause = run_state.job_deps.pause.hold(game);
    assert!(
        post_form(&run_app, "/autoplay", "cycles=2")
            .await
            .status()
            .is_success()
    );
    assert!(
        pause.wait_paused(5000),
        "the run reaches its pause before its first Job"
    );
    let before = run_state
        .game_catalogue
        .active_game()
        .expect("the Game reads back")
        .expect("the active Game exists");
    let other_mode = match before.narrator_mode {
        NarratorMode::Novel => NarratorMode::InteractiveFiction,
        NarratorMode::InteractiveFiction => NarratorMode::Novel,
    };
    let other_perspective = match before.narrative_perspective {
        NarrativePerspective::Second => NarrativePerspective::Third,
        NarrativePerspective::Third => NarrativePerspective::Second,
    };
    let config_writes = [
        (
            "/games/:id/mode",
            format!("narrator_mode={}", other_mode.as_str()),
        ),
        (
            "/games/:id/posture",
            format!(
                "narrative_perspective={}&narrative_tense={}",
                other_perspective.as_str(),
                before.narrative_tense.as_str()
            ),
        ),
        (
            "/games/:id/presets",
            format!(
                "system_preset_id={}&quantifier_preset_id={}&impersonate_preset_id={}&options_preset_id={}",
                before.active_system_prompt_preset_id,
                before.active_quantifier_prompt_preset_id,
                before.active_impersonate_prompt_preset_id,
                before.active_options_prompt_preset_id,
            ),
        ),
    ];
    for (route, body) in config_writes {
        let uri = route.replace(":id", &game.0.to_string());
        let response = post_form(&run_app, &uri, &body).await;
        assert_eq!(
            response.status(),
            StatusCode::CONFLICT,
            "{route} while the run holds the Game"
        );
        assert_eq!(
            error_lines_of(route, &response_body(response).await),
            [
                "The Game is busy, so the change was not saved.".to_string(),
                "Still thinking...".to_string()
            ],
            "{route} while the run holds the Game"
        );
    }
    post_form(&run_app, "/status/reset-busy", "").await;
    pause.release();
    assert!(wait_for_jobs(&run_state, 5000).await, "the run ends");
}

// [docs/specs/screen.md] SCENARIO: 45.19
#[tokio::test]
async fn test_the_first_line_of_a_failed_answer_says_what_was_not_done() {
    let (app, state, _storage, _second) = two_game_app();
    let active = active_game_of(&state).0;

    let invalid_value = post_form(&app, &format!("/games/{active}/rename"), "display_name=").await;
    assert_eq!(invalid_value.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        error_lines_of("an invalid value", &response_body(invalid_value).await),
        ["The Game was not renamed: Display name cannot be empty."],
        "an invalid answer keeps the engine's message in its one line"
    );

    let active_game = state
        .game_catalogue
        .game(chronicler_engine::domain::model::game::GameId(active))
        .expect("read the Game")
        .expect("the active Game exists");
    let row_button = post_form(&app, &format!("/games/{active}/delete"), "").await;
    assert_eq!(row_button.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        error_lines_of("a row button", &response_body(row_button).await),
        [format!(
            "The Game &quot;{}&quot; was not deleted: Cannot delete the active game.",
            active_game.display_name
        )],
        "a row button handler reads the name of its item on its failure path"
    );

    let message_action = post_form(&app, "/history/999999", "text=Changed.").await;
    assert_eq!(message_action.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        error_lines_of("a Message action", &response_body(message_action).await),
        ["The edit was not saved: Message not found."],
        "a Message action names no item"
    );

    let (status, body) = get_status_and_body(&app, "/fragment/games/999999/rename").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        error_lines_of("an opener", &body),
        ["The form was not opened: Game not found."]
    );

    let (failing_storage, failures) =
        chronicler_engine::test_support::empty_sqlite_storage().with_test_failures();
    let (failing_app, _state) = TestAppBuilder::default_test()
        .storage(Arc::new(failing_storage))
        .build_with_state();
    failures.set(
        chronicler_engine::adapters::driven::storage::StorageCall::SavePreset,
        chronicler_engine::adapters::driven::storage::TestOverride::internal("preset save failure"),
    );
    let failed = post_form(
        &failing_app,
        "/prompt-presets",
        "name=Fail+%3Cb%3E&instructions=Will+fail.&preset_type=system",
    )
    .await;
    assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let lines = error_lines_of("a failure", &response_body(failed).await);
    assert_eq!(
        lines[0], "The preset &quot;Fail &lt;b&gt;&quot; was not saved.",
        "a failure says what was not done, with the item as escaped text"
    );
    assert!(
        lines[1].contains("preset save failure"),
        "the engine's text is the second line: {lines:?}"
    );

    let rejected = post_form(&app, "/games/abc/rename", "display_name=Renamed").await;
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    let lines = error_lines_of("a rejection", &response_body(rejected).await);
    assert_eq!(
        lines[0], "The Game was not renamed.",
        "an answer that no constructor built also gets the words of its route"
    );
    assert!(
        lines[1].contains("abc"),
        "the text of the rejection is the second line: {lines:?}"
    );
}
