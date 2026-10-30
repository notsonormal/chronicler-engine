//! HTTP E2E tests for the change numbers (`GET /changes`): which numbers each save path moves, and which numbers a read, a failed save and a rolled-back transaction leave as they were. Tagged against `docs/specs/changes.md`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use serde_json::Value;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride, TxWrite};
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::change::{ChangeKind, EngineKind, GameKind};
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::settings::NarratorMode;
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{
    active_game_of, empty_sqlite_storage, tx_token, ScriptedModel, TestAppBuilder, TestMap,
};
use chronicler_engine::domain::model::message::{Maker, Swipe, SwipeAt};

use crate::support::app_wiring::{app_with_model, app_with_script, two_game_app};
use crate::support::http_fixtures::{append_narrations, create_game_from, story_of, world_form_body};
use crate::support::http_requests::{
    post_action, post_form, post_on_last_message, response_body, wait_for_jobs, wait_settled,
};
use crate::SettingsTestGuard;

fn kinds() -> [&'static str; 8] {
    ChangeKind::ALL.map(ChangeKind::token)
}

fn engine_wide_kinds() -> [&'static str; 5] {
    EngineKind::ALL.map(EngineKind::token)
}

async fn fetch_changes(app: &axum::Router) -> Value {
    let request = Request::builder()
        .uri("/changes")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /changes answers 200"
    );
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.starts_with("application/json"),
        "GET /changes answers JSON, got: {content_type:?}"
    );
    serde_json::from_str(&response_body(response).await).expect("the answer is a JSON object")
}

fn moved_among(kinds: &[&'static str], before: &Value, after: &Value) -> Vec<&'static str> {
    let mut moved: Vec<&'static str> = kinds
        .iter()
        .copied()
        .filter(|kind| before.get(*kind) != after.get(*kind))
        .collect();
    moved.sort_unstable();
    moved
}

fn assert_moved_among(
    kinds: &[&'static str],
    before: &Value,
    after: &Value,
    expected: &[&str],
    label: &str,
) {
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(
        moved_among(kinds, before, after),
        expected,
        "{label}: the numbers that moved, from {before} to {after}"
    );
}

fn assert_moved(before: &Value, after: &Value, expected: &[&str], label: &str) {
    assert_moved_among(&kinds(), before, after, expected, label);
    assert_revision_follows(before, after, !expected.is_empty(), label);
}

/// The revision goes up when a number moved and stays when none did.
fn assert_revision_follows(before: &Value, after: &Value, a_number_moved: bool, label: &str) {
    let (before_revision, after_revision) = (revision_of(before), revision_of(after));
    if a_number_moved {
        assert!(
            after_revision > before_revision,
            "{label}: a number moved, so the revision went up: {before} to {after}"
        );
    } else {
        assert_eq!(
            before_revision, after_revision,
            "{label}: no number moved, so the revision stayed: {before} to {after}"
        );
    }
}

fn revision_of(answer: &Value) -> u64 {
    answer["revision"]
        .as_u64()
        .unwrap_or_else(|| panic!("the answer holds a whole-number revision: {answer}"))
}

async fn assert_save_moves(
    app: &axum::Router,
    uri: &str,
    body: &str,
    expected: &[&str],
    label: &str,
) {
    let before = fetch_changes(app).await;
    let response = post_form(app, uri, body).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "{label}: the save answers 200"
    );
    let after = fetch_changes(app).await;
    assert_moved(&before, &after, expected, label);
}

// [docs/specs/changes.md] SCENARIO: 43.1
#[tokio::test]
async fn test_changes_answer_holds_the_server_id_the_game_and_a_number_for_each_kind() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let answer = fetch_changes(&app).await;
    assert!(
        answer["server"]
            .as_str()
            .is_some_and(|server| !server.is_empty()),
        "the server id is a non-empty string: {answer}"
    );
    assert_eq!(
        answer["game"],
        active_game_of(&state).0,
        "the answer names the active Game"
    );
    for kind in kinds() {
        assert!(
            answer[kind].is_u64(),
            "the number of {kind} is a whole number: {answer}"
        );
    }
    assert!(
        answer["revision"].is_u64(),
        "the revision is a whole number: {answer}"
    );

    let app = TestAppBuilder::default_test()
        .storage(Arc::new(empty_sqlite_storage()))
        .skip_seeding(true)
        .build();
    let answer = fetch_changes(&app).await;
    assert_eq!(
        answer["game"],
        Value::Null,
        "with no active Game the answer names none"
    );
    for kind in GameKind::ALL.map(GameKind::token) {
        assert!(
            answer.get(kind).is_none(),
            "with no active Game the answer holds no {kind}: {answer}"
        );
    }
    for kind in engine_wide_kinds() {
        assert!(
            answer[kind].is_u64(),
            "the number of {kind} is a whole number: {answer}"
        );
    }
    assert!(
        answer["revision"].is_u64(),
        "the revision is a whole number with no active Game: {answer}"
    );
}

// [docs/specs/changes.md] SCENARIO: 43.2
#[tokio::test]
async fn test_the_server_id_is_the_same_in_one_server_and_differs_between_two() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let first = fetch_changes(&app).await;
    let rename = post_form(
        &app,
        &format!("/games/{}/rename", active_game_of(&state).0),
        "display_name=Renamed",
    )
    .await;
    assert_eq!(rename.status(), StatusCode::OK);
    let second = fetch_changes(&app).await;
    assert_eq!(
        first["server"], second["server"],
        "one server keeps its id through a save"
    );

    let other = TestAppBuilder::default_test().build();
    assert_ne!(
        first["server"],
        fetch_changes(&other).await["server"],
        "two servers hold two ids"
    );
}

// [docs/specs/changes.md] SCENARIO: 43.3
#[tokio::test]
async fn test_a_read_moves_no_number() {
    let _guard = SettingsTestGuard::new();
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let (app, state, _storage) = app_with_script(model);
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Job ends");

    let baseline = fetch_changes(&app).await;
    for uri in [
        "/changes",
        "/screen?parts=header",
        "/screen?parts=failure-banner",
        "/screen?parts=story-log",
        "/screen?parts=visual-sidebar",
        "/screen?parts=options-dock",
        "/screen?parts=send-control,autoplay-controls",
        "/screen?parts=games",
        "/screen?parts=worlds",
        "/screen?parts=settings",
        "/screen?parts=prompt-presets",
        "/screen?parts=llm-messages",
        "/screen?parts=status-display",
        "/debug/state",
    ] {
        let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "GET {uri} answers 200");
        assert_moved(&baseline, &fetch_changes(&app).await, &[], uri);
    }
}

// [docs/specs/changes.md] SCENARIO: 43.4
#[tokio::test]
async fn test_a_save_of_a_game_moves_the_games_list_only() {
    for row in ["rename", "posture", "mode", "presets", "delete"] {
        let (app, state, _storage, second) = two_game_app();
        let active = active_game_of(&state).0;
        let mode = state
            .game_catalogue
            .active_game()
            .expect("the active Game reads")
            .expect("a Game is active")
            .narrator_mode;
        let (uri, body) = match row {
            "rename" => (format!("/games/{active}/rename"), "display_name=Renamed".to_string()),
            "posture" => (
                format!("/games/{active}/posture"),
                "narrative_perspective=second&narrative_tense=past".to_string(),
            ),
            "mode" => (
                format!("/games/{active}/mode"),
                match mode {
                    NarratorMode::Novel => "narrator_mode=interactive_fiction",
                    NarratorMode::InteractiveFiction => "narrator_mode=novel",
                }
                .to_string(),
            ),
            "presets" => (
                format!("/games/{active}/presets"),
                "system_preset_id=system_default&quantifier_preset_id=quantifier_default&impersonate_preset_id=impersonate_default&options_preset_id=options_default".to_string(),
            ),
            _ => (format!("/games/{second}/delete"), String::new()),
        };
        assert_save_moves(&app, &uri, &body, &["games"], row).await;
    }
}

// [docs/specs/changes.md] SCENARIO: 43.5
#[tokio::test]
async fn test_a_new_a_switched_and_a_reset_game_change_the_active_game() {
    for row in ["create", "switch", "reset", "switch to the active Game"] {
        let (app, state, storage, second) = two_game_app();
        let first = active_game_of(&state);
        let stored = storage.require_game(first.0).expect("the Game reads back");
        let (uri, body, moves): (String, String, &[&str]) = match row {
            "create" => (
                "/games".to_string(),
                format!(
                    "world_key={}&persona_key={}",
                    stored.world_key, stored.persona_key
                ),
                &["games"],
            ),
            "switch" => (format!("/games/{second}/switch"), String::new(), &["games"]),
            "reset" => ("/reset".to_string(), String::new(), &["games"]),
            _ => (format!("/games/{}/switch", first.0), String::new(), &[]),
        };

        let before = fetch_changes(&app).await;
        let response = post_form(&app, &uri, &body).await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{row}: the save answers 200"
        );
        let after = fetch_changes(&app).await;

        let expected_game = match row {
            "switch" => GameId(second),
            "switch to the active Game" => first,
            _ => {
                let active = active_game_of(&state);
                assert_ne!(active, first, "{row}: a new Game becomes active");
                active
            }
        };
        assert_eq!(after["game"], expected_game.0, "{row}: the active Game");
        assert_moved_among(&engine_wide_kinds(), &before, &after, moves, row);
    }
}

fn connection_form(name: &str, model: &str) -> String {
    format!("conn_name={name}&conn_provider=mock&conn_model={model}&conn_api_key=&conn_base_url=")
}

fn connection_id_of(state: &AppState, name: &str) -> String {
    state
        .settings_service
        .get_settings()
        .expect("the settings read")
        .connections
        .into_iter()
        .find(|connection| connection.name == name)
        .unwrap_or_else(|| panic!("no Connection named {name:?}"))
        .id
}

// [docs/specs/changes.md] SCENARIO: 43.6
#[tokio::test]
async fn test_a_save_of_the_settings_moves_the_settings_only() {
    let _guard = SettingsTestGuard::new();
    for row in [
        "add",
        "edit",
        "delete",
        "narrator",
        "quantifier",
        "text check",
    ] {
        let (app, state) = TestAppBuilder::default_test().build_with_state();
        let arranged = row != "add" && row != "text check";
        if arranged {
            let added = post_form(
                &app,
                "/connections/add",
                &connection_form("Alpha", "alpha-model"),
            )
            .await;
            assert_eq!(
                added.status(),
                StatusCode::OK,
                "{row}: the arranged Connection saves"
            );
        }
        let (uri, body) = match row {
            "add" => (
                "/connections/add".to_string(),
                connection_form("Alpha", "alpha-model"),
            ),
            "edit" => (
                format!("/connections/{}/edit", connection_id_of(&state, "Alpha")),
                connection_form("Alpha", "alpha-model-2"),
            ),
            "delete" => (
                format!("/connections/{}/delete", connection_id_of(&state, "Alpha")),
                String::new(),
            ),
            "narrator" => (
                "/connections/set-narrator".to_string(),
                format!("connection_id={}", connection_id_of(&state, "Alpha")),
            ),
            "quantifier" => (
                "/connections/set-quantifier".to_string(),
                format!("connection_id={}", connection_id_of(&state, "Alpha")),
            ),
            _ => (
                "/settings/text-check".to_string(),
                "check_mode=spell&enable_auto_check=true".to_string(),
            ),
        };
        assert_save_moves(&app, &uri, &body, &["settings"], row).await;
    }
}

fn preset_id_named(state: &AppState, name: &str) -> String {
    use chronicler_engine::domain::model::prompt_preset::PresetType;
    state
        .prompt_preset_service
        .list_presets(PresetType::System)
        .expect("the presets read")
        .into_iter()
        .find(|preset| preset.name == name)
        .unwrap_or_else(|| panic!("no system preset named {name:?}"))
        .id
}

// [docs/specs/changes.md] SCENARIO: 43.7
#[tokio::test]
async fn test_a_save_of_a_prompt_preset_moves_the_presets_and_an_activate_moves_the_settings() {
    let _guard = SettingsTestGuard::new();
    for (row, moves) in [
        ("create", "presets"),
        ("update", "presets"),
        ("duplicate", "presets"),
        ("delete", "presets"),
        ("activate", "settings"),
    ] {
        let (app, state) = TestAppBuilder::default_test().build_with_state();
        let arranged = post_form(
            &app,
            "/prompt-presets",
            "name=Probe&instructions=Probe.&preset_type=system",
        )
        .await;
        assert_eq!(
            arranged.status(),
            StatusCode::OK,
            "{row}: the arranged preset saves"
        );
        let id = preset_id_named(&state, "Probe");
        let (uri, body) = match row {
            "create" => (
                "/prompt-presets".to_string(),
                "name=Created&instructions=Created.&preset_type=system".to_string(),
            ),
            "update" => (
                format!("/prompt-presets/{id}"),
                "name=Probe&instructions=Updated.&preset_type=system".to_string(),
            ),
            "duplicate" => (format!("/prompt-presets/{id}/duplicate"), String::new()),
            "delete" => (format!("/prompt-presets/{id}/delete"), String::new()),
            _ => (format!("/prompt-presets/{id}/activate"), String::new()),
        };
        assert_save_moves(&app, &uri, &body, &[moves], row).await;
    }
}

fn probe_world() -> WorldCard {
    WorldCard {
        key: "probe_world".to_string(),
        name: "Probe World".to_string(),
        description: "A World that no Game uses.".to_string(),
        ..Default::default()
    }
}

// [docs/specs/changes.md] SCENARIO: 43.8
#[tokio::test]
async fn test_a_save_of_a_world_moves_the_worlds_only() {
    for row in ["create", "update", "posture", "delete"] {
        let storage = Arc::new(empty_sqlite_storage());
        let (app, _state) = TestAppBuilder::default_test()
            .storage(Arc::clone(&storage))
            .build_with_state();
        storage
            .seed_world(&probe_world(), &TestMap::single_room("start"))
            .expect("seed the probe World");
        let map = TestMap::single_room("start");
        let (uri, body) = match row {
            "create" => {
                let created = WorldCard {
                    key: "created_world".to_string(),
                    name: "Created World".to_string(),
                    ..probe_world()
                };
                ("/worlds".to_string(), world_form_body(&created, &map))
            }
            "update" => (
                "/worlds/probe_world".to_string(),
                world_form_body(&probe_world(), &map),
            ),
            "posture" => (
                "/worlds/probe_world/posture".to_string(),
                "narrative_tense=present".to_string(),
            ),
            _ => ("/worlds/probe_world/delete".to_string(), String::new()),
        };
        assert_save_moves(&app, &uri, &body, &["worlds"], row).await;
    }
}

// [docs/specs/changes.md] SCENARIO: 43.9
#[tokio::test]
async fn test_a_save_that_fails_moves_no_number() {
    let _guard = SettingsTestGuard::new();
    for row in [
        "rename",
        "text check",
        "preset create",
        "preset activate",
        "world create",
        "world delete",
    ] {
        let (storage, failures) = empty_sqlite_storage().with_test_failures();
        let storage = Arc::new(storage);
        let (app, state) = TestAppBuilder::default_test()
            .storage(Arc::clone(&storage))
            .build_with_state();
        let active = active_game_of(&state).0;
        let created_world = WorldCard {
            key: "created_world".to_string(),
            name: "Created World".to_string(),
            ..probe_world()
        };
        let (call, uri, body) = match row {
            "rename" => (
                StorageCall::UpdateGameDisplayName,
                format!("/games/{active}/rename"),
                "display_name=Renamed".to_string(),
            ),
            "text check" => (
                StorageCall::UpdateSettings,
                "/settings/text-check".to_string(),
                "check_mode=spell&enable_auto_check=true".to_string(),
            ),
            "preset create" => (
                StorageCall::SavePreset,
                "/prompt-presets".to_string(),
                "name=Probe&instructions=Probe.&preset_type=system".to_string(),
            ),
            "preset activate" => (
                StorageCall::UpdateSettings,
                "/prompt-presets/system_default/activate".to_string(),
                String::new(),
            ),
            "world create" => (
                StorageCall::CreateWorld,
                "/worlds".to_string(),
                world_form_body(&created_world, &TestMap::single_room("start")),
            ),
            _ => (
                StorageCall::DeleteWorld,
                "/worlds/test/delete".to_string(),
                String::new(),
            ),
        };
        failures.set(call, TestOverride::internal("the write fails"));

        let before = fetch_changes(&app).await;
        let response = post_form(&app, &uri, &body).await;
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{row}: the save fails with 500"
        );
        assert_moved(&before, &fetch_changes(&app).await, &[], row);
    }
}

// [docs/specs/changes.md] SCENARIO: 43.10
#[tokio::test]
async fn test_a_transaction_whose_later_write_fails_moves_none_of_its_own_numbers() {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let storage = Arc::new(storage);
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    append_narrations(&storage, active_game_of(&state), 1, |_| {
        "A second Narration.".to_string()
    });
    failures.set_write(
        TxWrite::PutDock,
        TestOverride::internal("the dock write fails"),
    );

    let before = fetch_changes(&app).await;
    let response = post_on_last_message(&app, "/history/delete").await;
    assert_ne!(
        response.status(),
        StatusCode::OK,
        "a delete whose transaction fails does not answer 200"
    );
    let body = response_body(response).await;
    assert!(
        body.to_lowercase().contains("save"),
        "the answer says the save failed, got: {body}"
    );
    assert_moved(
        &before,
        &fetch_changes(&app).await,
        &["busy"],
        "the rolled-back writes raise nothing, and the failure line raises busy",
    );
}

fn held_action() -> (
    Arc<ScriptedModel>,
    chronicler_engine::test_support::ModelHold,
) {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    (model, hold)
}

// [docs/specs/changes.md] SCENARIO: 43.11
#[tokio::test]
async fn test_a_job_moves_the_busy_state_when_it_starts_and_when_it_ends() {
    let (model, hold) = held_action();
    let (app, state) = app_with_model(model);
    let baseline = fetch_changes(&app).await;

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(hold.wait_started(1000), "the Job reaches its narrator call");
    let running = fetch_changes(&app).await;
    assert_moved(
        &baseline,
        &running,
        &["busy", "story", "games"],
        "a Job at its narrator call",
    );

    hold.release();
    assert!(wait_settled(&state, 2000).await, "the Job ends");
    let ended = fetch_changes(&app).await;
    assert_ne!(
        running["busy"], ended["busy"],
        "the end of the Job moves the busy state again"
    );
    assert_moved(
        &baseline,
        &ended,
        &["story", "options", "busy", "games", "llm"],
        "a Job that ended",
    );
}

async fn pause_before_first_job(
    app: &axum::Router,
    state: &AppState,
    game: GameId,
) -> chronicler_engine::application::autoplay::pause::PauseHandle {
    let pause = state.job_deps.pause.hold(game);
    assert!(
        post_form(app, "/autoplay", "cycles=2")
            .await
            .status()
            .is_success()
    );
    assert!(
        pause.wait_paused(5000),
        "the run reaches its pause before its first Job"
    );
    pause
}

// [docs/specs/changes.md] SCENARIO: 43.12
#[tokio::test]
async fn test_the_start_and_the_end_of_an_autoplay_run_move_the_busy_state_and_the_games_list() {
    let (app, state, _storage) = app_with_script(Arc::new(ScriptedModel::new()));
    let game = active_game_of(&state);
    let baseline = fetch_changes(&app).await;

    let pause = pause_before_first_job(&app, &state, game).await;
    assert_moved(
        &baseline,
        &fetch_changes(&app).await,
        &["busy", "games"],
        "the start of a run",
    );

    let started = fetch_changes(&app).await;

    let stopped = post_form(&app, "/status/reset-busy", "").await;
    assert_eq!(response_body(stopped).await, "reset");
    pause.release();
    assert!(wait_for_jobs(&state, 5000).await, "the run ends");
    assert_moved(
        &started,
        &fetch_changes(&app).await,
        &["busy", "games", "options"],
        "the stop of a run, since its start",
    );
}

// [docs/specs/changes.md] SCENARIO: 43.13
#[tokio::test]
async fn test_a_change_of_the_step_of_a_run_moves_the_busy_state_and_not_the_games_list() {
    let model = Arc::new(ScriptedModel::new().narration("The late answer."));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    let pause = pause_before_first_job(&app, &state, game).await;
    let at_the_start = fetch_changes(&app).await;

    pause.release();
    assert!(
        hold.wait_started(5000),
        "the first Job of the run reaches its narrator call"
    );
    assert_moved(
        &at_the_start,
        &fetch_changes(&app).await,
        &["busy"],
        "the first Job of a run at its narrator call",
    );

    let stopped = post_form(&app, "/status/reset-busy", "").await;
    assert_eq!(response_body(stopped).await, "reset");
    hold.release();
    assert!(hold.wait_taken(1000), "the stopped call takes its answer");
    assert!(wait_for_jobs(&state, 5000).await, "the run ends");
}

// [docs/specs/changes.md] SCENARIO: 43.14
#[tokio::test]
async fn test_a_message_saved_in_a_game_that_is_not_active_moves_the_games_list() {
    let (model, hold) = held_action();
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    let first = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(hold.wait_started(1000), "the Job reaches its narrator call");
    let second = create_game_from(&app, &state, &storage, first, "test").await;
    let after_the_create = fetch_changes(&app).await;

    hold.release();
    assert!(wait_for_jobs(&state, 2000).await, "the Job ends");
    let after_the_job = fetch_changes(&app).await;

    assert_eq!(
        after_the_job["game"], second.0,
        "the second Game stays active"
    );
    assert_moved(
        &after_the_create,
        &after_the_job,
        &["games", "llm"],
        "the end of a Job in a Game that is not active",
    );
}

// [docs/specs/changes.md] SCENARIO: 43.15
#[tokio::test]
async fn test_a_save_of_the_story_log_moves_the_story_and_a_delete_also_the_games_list() {
    for row in ["edit", "switch", "delete"] {
        let storage = Arc::new(empty_sqlite_storage());
        let (app, state) = TestAppBuilder::default_test()
            .storage(Arc::clone(&storage))
            .build_with_state();
        let game = active_game_of(&state);
        append_narrations(&storage, game, 1, |_| "A second Narration.".to_string());
        let last = story_of(&state).pop().expect("the story holds a Message");
        let second_swipe = Swipe::new(
            "A second Swipe.".to_string(),
            Maker::Narrator,
            last.swipes[0].state.clone(),
        );
        storage
            .transaction(game, tx_token(), |tx| {
                tx.insert_swipe(
                    SwipeAt {
                        message: last.id,
                        swipe: 1,
                    },
                    &second_swipe,
                )
            })
            .expect("the second Swipe saves");

        let (uri, body, moves): (String, String, &[&str]) = match row {
            "edit" => (
                format!("/history/{}", last.id),
                "text=An+edited+text".to_string(),
                &["story"],
            ),
            "switch" => (
                format!("/message/{}/swipe/1", last.id),
                String::new(),
                &["story", "options", "busy"],
            ),
            _ => (
                "/history/delete".to_string(),
                format!("message={}", last.id),
                &["story", "games", "options", "busy"],
            ),
        };
        assert_save_moves(&app, &uri, &body, moves, row).await;
    }
}

// [docs/specs/changes.md] SCENARIO: 43.16
#[tokio::test]
async fn test_the_revision_goes_up_with_every_number_that_moves_and_stays_with_none() {
    let _guard = SettingsTestGuard::new();
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let active = active_game_of(&state).0;

    let first = fetch_changes(&app).await;
    let second = fetch_changes(&app).await;
    assert_revision_follows(&first, &second, false, "two reads");

    let mut before = second;
    for (label, uri, body) in [
        (
            "a rename",
            format!("/games/{active}/rename"),
            "display_name=Renamed".to_string(),
        ),
        ("a reset", "/reset".to_string(), String::new()),
        (
            "a Connection",
            "/connections/add".to_string(),
            connection_form("Alpha", "alpha-model"),
        ),
    ] {
        let response = post_form(&app, &uri, &body).await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{label}: the save answers 200"
        );
        let after = fetch_changes(&app).await;
        assert_revision_follows(&before, &after, true, label);
        before = after;
    }
}
