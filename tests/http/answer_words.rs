//! HTTP E2E tests for the words of a failed answer: every row of the words table pinned by a hand-written sentence. They also cover the second line that holds only the engine's text, and the two answers that an extractor writes (`415` and `422`). Tagged against `docs/specs/screen.md`.

use std::collections::BTreeSet;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::adapters::driving::http::utils::answer_words::ANSWER_WORDS;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::error::{EngineError, InternalError};
use chronicler_engine::test_support::{
    active_game_of, empty_sqlite_storage, ScriptedModel, TestAppBuilder, TestMap, TestPromptPreset,
};

use crate::screen::{game_bound_rows, GAME_HEADER, STALE_GAME_LINE};
use crate::support::app_wiring::{app_with_model, two_game_app};
use crate::support::http_fixtures::{story_of, world_form_body};
use crate::support::http_requests::{
    fetch_answer, post_action, post_form, post_form_with_headers, response_body, wait_for_jobs,
    wait_settled,
};

async fn send(
    app: &axum::Router,
    method: Method,
    uri: &str,
    form: &str,
) -> axum::response::Response<Body> {
    let request = Request::builder()
        .uri(uri)
        .method(method)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form.to_string()))
        .unwrap();
    app.clone().oneshot(request).await.unwrap()
}

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

// [docs/specs/screen.md] SCENARIO: 45.18
#[tokio::test]
async fn test_each_kind_of_failed_request_answers_its_status_and_one_body() {
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
        let (status, body) = fetch_answer(&app, uri).await;
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

pub(crate) fn error_lines_of(label: &str, body: &str) -> Vec<String> {
    error_text_of(label, body)
        .split('\n')
        .map(str::to_string)
        .collect()
}

fn words_of(route: &str) -> String {
    let row = ANSWER_WORDS
        .iter()
        .find(|row| row.route == route)
        .unwrap_or_else(|| panic!("the words table has a row for {route}"));
    format!("the {} {}", row.noun, row.predicate)
}

// [docs/specs/screen.md] SCENARIO: 45.17
#[tokio::test]
async fn test_a_stale_game_refusal_names_what_was_not_done_on_each_game_bound_route() {
    let (app, state, _storage, second) = two_game_app();
    let message = story_of(&state).last().expect("a story").id;
    let stale = second.to_string();
    for (route, uri, body) in game_bound_rows(message) {
        let response =
            post_form_with_headers(&app, &uri, &body, &[(GAME_HEADER, stale.as_str())]).await;
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
}

// [docs/specs/screen.md] SCENARIO: 45.17
#[tokio::test]
async fn test_a_busy_refusal_names_what_was_not_done_on_each_route_that_a_job_refuses() {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    let (busy_app, busy_state) = app_with_model(Arc::clone(&model));
    assert!(post_action(&busy_app, "look").await.status().is_success());
    assert!(hold.wait_started(1000), "the Job reaches its narrator call");
    let busy_message = story_of(&busy_state).last().expect("a story").id;
    let busy_named = format!("message={busy_message}");
    let busy_rows = [
        ("/action", "/action".to_string(), "command=listen"),
        (
            "/action/check",
            "/action/check".to_string(),
            "command=/options",
        ),
        ("/autoplay", "/autoplay".to_string(), "cycles=1"),
        ("/swipe/new", "/swipe/new".to_string(), busy_named.as_str()),
        ("/retrigger", "/retrigger".to_string(), busy_named.as_str()),
        (
            "/history/:id",
            format!("/history/{busy_message}"),
            "text=Edited",
        ),
        (
            "/history/delete",
            "/history/delete".to_string(),
            busy_named.as_str(),
        ),
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
}

// [docs/specs/screen.md] SCENARIO: 45.17
#[tokio::test]
async fn test_a_busy_refusal_names_what_was_not_done_on_each_config_save_during_an_autoplay_run() {
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

    let message_request = post_form(&app, "/history/999999", "text=Changed.").await;
    assert_eq!(message_request.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        error_lines_of("a Message request", &response_body(message_request).await),
        ["The edit was not saved: Message not found."],
        "a Message request names no item"
    );

    let (status, body) = fetch_answer(&app, "/fragment/games/999999/rename").await;
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

// [docs/specs/screen.md] SCENARIO: 45.20
#[tokio::test]
async fn test_a_request_that_names_no_last_message_is_invalid_on_each_route_that_needs_one() {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let (app, state) = app_with_model(Arc::clone(&model));
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Job settles");
    let before = story_of(&state).len();

    for (uri, sentence) in [
        (
            "/history/delete",
            "The Message was not deleted: The request names no Message.",
        ),
        (
            "/swipe/new",
            "The Retry was not started: The request names no Message.",
        ),
        (
            "/retrigger",
            "The retrigger was not started: The request names no Message.",
        ),
    ] {
        let response = post_form(&app, uri, "").await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "POST {uri}");
        let body = response_body(response).await;
        assert_eq!(error_lines_of(uri, &body), [sentence], "POST {uri}");
    }
    assert_eq!(story_of(&state).len(), before, "the refusals wrote nothing");
}

/// One row for each row of the words table: the route as the router writes it, an
/// address that the route matches, and the sentence that the first line of its
/// failed answer starts with. The test sends the method that the route does not take.
const PINNED_WORDS: &[(&str, &str, &str)] = &[
    ("/action", "/action", "The Action was not sent."),
    ("/action/check", "/action/check", "The Action was not sent."),
    ("/autoplay", "/autoplay", "The Autoplay run did not start."),
    ("/swipe/new", "/swipe/new", "The Retry was not started."),
    ("/retrigger", "/retrigger", "The retrigger was not started."),
    ("/reset", "/reset", "The Game was not reset."),
    (
        "/message/:id/swipe/:index",
        "/message/1/swipe/0",
        "The Swipe was not switched.",
    ),
    ("/history/:id", "/history/1", "The edit was not saved."),
    (
        "/history/delete",
        "/history/delete",
        "The Message was not deleted.",
    ),
    (
        "/status/reset-busy",
        "/status/reset-busy",
        "The Autoplay run was not stopped.",
    ),
    ("/games", "/games", "The Game was not started."),
    (
        "/games/:id/switch",
        "/games/1/switch",
        "The Game was not switched.",
    ),
    (
        "/games/:id/rename",
        "/games/1/rename",
        "The Game was not renamed.",
    ),
    (
        "/games/:id/delete",
        "/games/1/delete",
        "The Game was not deleted.",
    ),
    (
        "/games/:id/mode",
        "/games/1/mode",
        "The change was not saved.",
    ),
    (
        "/games/:id/posture",
        "/games/1/posture",
        "The change was not saved.",
    ),
    (
        "/games/:id/presets",
        "/games/1/presets",
        "The change was not saved.",
    ),
    ("/worlds", "/worlds", "The World was not created."),
    ("/worlds/:key", "/worlds/test", "The World was not saved."),
    (
        "/worlds/:key/posture",
        "/worlds/test/posture",
        "The posture was not saved.",
    ),
    (
        "/worlds/:key/delete",
        "/worlds/test/delete",
        "The World was not deleted.",
    ),
    (
        "/connections/add",
        "/connections/add",
        "The Connection was not saved.",
    ),
    (
        "/connections/:id/edit",
        "/connections/c1/edit",
        "The Connection was not saved.",
    ),
    (
        "/connections/test",
        "/connections/test",
        "The Connection was not tested.",
    ),
    (
        "/connections/:id/test",
        "/connections/c1/test",
        "The Connection was not tested.",
    ),
    (
        "/connections/:id/delete",
        "/connections/c1/delete",
        "The Connection was not deleted.",
    ),
    (
        "/connections/set-narrator",
        "/connections/set-narrator",
        "The Narrator was not changed.",
    ),
    (
        "/connections/set-quantifier",
        "/connections/set-quantifier",
        "The Quantifier was not changed.",
    ),
    (
        "/settings/text-check",
        "/settings/text-check",
        "The text-check setting was not saved.",
    ),
    (
        "/prompt-presets",
        "/prompt-presets",
        "The preset was not saved.",
    ),
    (
        "/prompt-presets/:id",
        "/prompt-presets/p1",
        "The preset was not saved.",
    ),
    (
        "/prompt-presets/:id/delete",
        "/prompt-presets/p1/delete",
        "The preset was not deleted.",
    ),
    (
        "/prompt-presets/:id/duplicate",
        "/prompt-presets/p1/duplicate",
        "The preset was not duplicated.",
    ),
    (
        "/prompt-presets/:id/activate",
        "/prompt-presets/p1/activate",
        "The preset was not activated.",
    ),
    (
        "/fragment/games/new",
        "/fragment/games/new",
        "The form was not opened.",
    ),
    (
        "/fragment/games/:id/rename",
        "/fragment/games/1/rename",
        "The form was not opened.",
    ),
    (
        "/fragment/worlds/new",
        "/fragment/worlds/new",
        "The form was not opened.",
    ),
    (
        "/worlds/:key/edit",
        "/worlds/test/edit",
        "The form was not opened.",
    ),
    (
        "/fragment/connections/new",
        "/fragment/connections/new",
        "The form was not opened.",
    ),
    (
        "/fragment/connections/:id/edit",
        "/fragment/connections/c1/edit",
        "The form was not opened.",
    ),
    (
        "/fragment/prompt-presets/new",
        "/fragment/prompt-presets/new",
        "The form was not opened.",
    ),
    (
        "/fragment/prompt-presets/:id/edit",
        "/fragment/prompt-presets/p1/edit",
        "The form was not opened.",
    ),
    (
        "/fragment/prompt-presets/:id/view",
        "/fragment/prompt-presets/p1/view",
        "The form was not opened.",
    ),
    (
        "/fragment/llm-messages/:id",
        "/fragment/llm-messages/1",
        "The form was not opened.",
    ),
];

/// The routes that a `GET` opener takes. Every other row of the table is a `POST` route.
fn takes_get(route: &str) -> bool {
    route.starts_with("/fragment/") || route == "/worlds/:key/edit"
}

// [docs/specs/screen.md] SCENARIO: 45.21
#[tokio::test]
async fn test_each_row_of_the_words_table_words_its_route() {
    assert_eq!(
        PINNED_WORDS
            .iter()
            .map(|(route, _, _)| *route)
            .collect::<BTreeSet<_>>(),
        ANSWER_WORDS
            .iter()
            .map(|row| row.route)
            .collect::<BTreeSet<_>>(),
        "the pinned list holds one sentence for each row of the words table, and no other"
    );
    assert_eq!(
        PINNED_WORDS.len(),
        ANSWER_WORDS.len(),
        "no route is pinned twice"
    );

    let (app, _state, _storage, _second) = two_game_app();
    for (route, uri, sentence) in PINNED_WORDS {
        let method = if takes_get(route) {
            Method::POST
        } else {
            Method::GET
        };
        let response = send(&app, method.clone(), uri, "").await;
        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} {uri} is a request that the route does not take"
        );
        assert_eq!(
            error_lines_of(route, &response_body(response).await),
            [*sentence],
            "the words of {route}"
        );
    }
}

/// The text of the engine error that `TestOverride::internal` plants.
fn planted_text(message: &str) -> String {
    EngineError::Internal(InternalError::new(message)).to_string()
}

struct HoldCase<'a> {
    label: &'static str,
    method: Method,
    uri: &'static str,
    form: &'a str,
    call: StorageCall,
}

fn world_form(name: &str) -> String {
    let world = WorldCard {
        key: "test".to_string(),
        name: name.to_string(),
        ..WorldCard::default()
    };
    world_form_body(&world, &TestMap::single_room("start"))
}

// [docs/specs/screen.md] SCENARIO: 45.22
#[tokio::test]
async fn test_the_second_line_of_a_failed_answer_holds_only_the_engine_text() {
    let world_update = world_form("Typed");
    let cases = [
        HoldCase {
            label: "the text-check save",
            method: Method::POST,
            uri: "/settings/text-check",
            form: "check_mode=spell&enable_auto_check=true",
            call: StorageCall::UpdateSettings,
        },
        HoldCase {
            label: "the LLM record opener",
            method: Method::GET,
            uri: "/fragment/llm-messages/1",
            form: "",
            call: StorageCall::GetLlmMessage,
        },
        HoldCase {
            label: "the preset View opener",
            method: Method::GET,
            uri: "/fragment/prompt-presets/seed_ref/view",
            form: "",
            call: StorageCall::GetPreset,
        },
        HoldCase {
            label: "the preset Duplicate",
            method: Method::POST,
            uri: "/prompt-presets/seed_ref/duplicate",
            form: "",
            call: StorageCall::GetPreset,
        },
        HoldCase {
            label: "the preset Activate",
            method: Method::POST,
            uri: "/prompt-presets/seed_ref/activate",
            form: "",
            call: StorageCall::UpdateSettings,
        },
        HoldCase {
            label: "the Rename opener",
            method: Method::GET,
            uri: "/fragment/games/1/rename",
            form: "",
            call: StorageCall::GetGame,
        },
        HoldCase {
            label: "the New Game opener",
            method: Method::GET,
            uri: "/fragment/games/new",
            form: "",
            call: StorageCall::ListWorlds,
        },
        HoldCase {
            label: "the World Edit opener",
            method: Method::GET,
            uri: "/worlds/test/edit",
            form: "",
            call: StorageCall::GetWorld,
        },
        HoldCase {
            label: "the World update, read",
            method: Method::POST,
            uri: "/worlds/test",
            form: &world_update,
            call: StorageCall::GetWorld,
        },
        HoldCase {
            label: "the World update, write",
            method: Method::POST,
            uri: "/worlds/test",
            form: &world_update,
            call: StorageCall::UpdateWorld,
        },
        HoldCase {
            label: "the World posture, read",
            method: Method::POST,
            uri: "/worlds/test/posture",
            form: "narrative_tense=present",
            call: StorageCall::GetWorld,
        },
        HoldCase {
            label: "the World posture, write",
            method: Method::POST,
            uri: "/worlds/test/posture",
            form: "narrative_tense=present",
            call: StorageCall::UpdateWorld,
        },
    ];

    for case in cases {
        let (storage, failures) = empty_sqlite_storage().with_test_failures();
        let storage = Arc::new(storage);
        let (app, _state) = TestAppBuilder::default_test()
            .storage(Arc::clone(&storage))
            .build_with_state();
        storage
            .save_preset(&TestPromptPreset::system("seed_ref", "Seed"))
            .expect("seed a preset");
        failures.set(case.call, TestOverride::internal("planted failure"));

        let response = send(&app, case.method.clone(), case.uri, case.form).await;
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{}: the engine failed",
            case.label
        );
        let lines = error_lines_of(case.label, &response_body(response).await);
        assert_eq!(
            lines.get(1).map(String::as_str),
            Some(planted_text("planted failure").as_str()),
            "{}: the second line is the engine's text and no more: {lines:?}",
            case.label
        );
    }
}

// [docs/specs/screen.md] SCENARIO: 45.23
#[tokio::test]
async fn test_a_message_that_is_not_a_number_answers_422_with_the_words_of_its_route() {
    let (app, state, _storage, _second) = two_game_app();
    let before = story_of(&state).len();

    for (uri, sentence) in [
        ("/history/delete", "The Message was not deleted."),
        ("/swipe/new", "The Retry was not started."),
        ("/retrigger", "The retrigger was not started."),
    ] {
        let response = post_form(&app, uri, "message=latest").await;
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "POST {uri} with a message that is not a number"
        );
        let lines = error_lines_of(uri, &response_body(response).await);
        assert_eq!(lines[0], sentence, "the words of {uri}");
        assert!(
            lines.len() == 2 && lines[1].contains("form body"),
            "the second line holds the text of the rejection: {lines:?}"
        );
    }
    assert_eq!(story_of(&state).len(), before, "the answers wrote nothing");
}

// [docs/specs/screen.md] SCENARIO: 45.24
#[tokio::test]
async fn test_a_form_post_with_no_content_type_answers_415_with_the_words_of_its_route() {
    let (app, state, _storage, _second) = two_game_app();
    let active = active_game_of(&state).0;
    let before = story_of(&state).len();

    for (uri, sentence) in [
        (
            "/history/delete".to_string(),
            "The Message was not deleted.",
        ),
        (
            format!("/games/{active}/rename"),
            "The Game was not renamed.",
        ),
    ] {
        let request = Request::builder()
            .uri(&uri)
            .method(Method::POST)
            .body(Body::from("message=1"))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "POST {uri} with no form Content-Type"
        );
        let lines = error_lines_of(&uri, &response_body(response).await);
        assert_eq!(lines[0], sentence, "the words of {uri}");
        assert!(
            lines.len() == 2 && lines[1].contains("Content-Type"),
            "the second line holds the text of the rejection: {lines:?}"
        );
    }
    assert_eq!(story_of(&state).len(), before, "the answers wrote nothing");
}
