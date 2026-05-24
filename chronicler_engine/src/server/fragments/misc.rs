use std::sync::Arc;

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    http::StatusCode,
};

use crate::model::game::generate_game_name;
use crate::model::settings::TextCheckMode;
use crate::model::state::GameState;
use crate::narrative::text_check::check_player_input;
use crate::server::AppState;
use crate::server::templates::TextCheckPreviewTemplate;

use super::renderers::render_error;

#[allow(clippy::expect_used)]
fn internal_error_response(message: impl Into<String>) -> axum::response::Response<Body> {
    let msg = message.into();
    axum::response::Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(render_error(&msg)))
        .expect("static response body is valid")
}

/// [DOC: docs/system/text_check.md]
#[allow(clippy::expect_used)]
pub async fn check_text_handler(
    State(state): State<AppState>,
    Form(form): Form<super::actions::ActionForm>,
) -> axum::response::Response<Body> {
    let text = form.command.trim().to_string();
    if text.is_empty() {
        return axum::response::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "<span class=\"status error\">Enter text to check</span>",
            ))
            .expect("static response body is valid");
    }

    let settings = state.settings();

    if settings.text_check.mode == TextCheckMode::Disabled {
        return axum::response::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                "<span class=\"status ready\">Text check is disabled</span>",
            ))
            .expect("static response body is valid");
    }

    match check_player_input(
        &text,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(Some(result)) => {
            let template = TextCheckPreviewTemplate::from_check_result(&result);
            match template.render() {
                Ok(html) => axum::response::Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(html))
                    .expect("static response body is valid"),
                Err(e) => axum::response::Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(render_error(&format!("Template error: {e}"))))
                    .expect("static response body is valid"),
            }
        }
        Ok(None) => axum::response::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                "<span class=\"status ready\">No issues found</span>",
            ))
            .expect("static response body is valid"),
        Err(e) => axum::response::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(render_error(&format!("Check failed: {e}"))))
            .expect("static response body is valid"),
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn retry_handler(State(state): State<AppState>) -> (StatusCode, String) {
    let mut game_state = match state.load_state() {
        Ok(gs) => gs,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                render_error("Failed to load state"),
            );
        }
    };

    if game_state.narrative.history.last_input_text().is_none() {
        return (StatusCode::BAD_REQUEST, render_error("No input to retry"));
    }

    game_state.narrative.input_buffer.status = crate::model::state::GenerationStatus::Generating;
    game_state.narrative.input_buffer.phase = crate::model::state::GenerationPhase::Narrating;
    let generating_snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&game_state);
    if let Err(e) = state.snapshot_storage.save(&generating_snapshot) {
        log::error!("Failed to save retry snapshot: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            render_error(&format!("Failed to save state: {e}")),
        );
    }

    let ctx = state.as_game_service_context();
    let game_service = state.game_service.clone();
    let token = state.current_cancel_token();

    if token.is_cancelled() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            render_error("Server is shutting down"),
        );
    }

    // [DOC: docs/architecture/invariants.md#INV-004]
    // Retry runs off the async thread so the HTTP handler returns immediately.
    tokio::task::spawn_blocking(move || {
        if token.is_cancelled() {
            return;
        }
        game_service.retry_last_response(ctx);
    });

    (
        StatusCode::OK,
        "<span class=\"status ready\">Retrying...</span>".to_string(),
    )
}

/// Requires `last_trigger` to be present and the last message to be a narration.
/// [DOC: docs/architecture/system.md]
pub async fn retrigger_handler(State(state): State<AppState>) -> (StatusCode, String) {
    let mut game_state = match state.load_state() {
        Ok(gs) => gs,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                render_error("Failed to load state"),
            );
        }
    };

    if game_state.narrative.last_trigger.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            render_error("No trigger context available"),
        );
    }

    let messages = match state.message_storage.load_messages() {
        Ok(msgs) => msgs,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                render_error("Failed to load messages"),
            );
        }
    };

    let Some(last_msg) = messages.last() else {
        return (
            StatusCode::BAD_REQUEST,
            render_error("No messages to retrigger"),
        );
    };
    let is_narration = last_msg.log_type == crate::model::state::LogType::Narration
        || last_msg.log_type == crate::model::state::LogType::Dialogue;
    if !is_narration || last_msg.event_header.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            render_error("Last message must be a narration to retrigger"),
        );
    }

    game_state.narrative.input_buffer.status = crate::model::state::GenerationStatus::Generating;
    game_state.narrative.input_buffer.phase = crate::model::state::GenerationPhase::Narrating;
    let generating_snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&game_state);
    if let Err(e) = state.snapshot_storage.save(&generating_snapshot) {
        log::error!("Failed to save retrigger snapshot: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            render_error(&format!("Failed to save state: {e}")),
        );
    }

    let ctx = state.as_game_service_context();
    let game_service = state.game_service.clone();
    let token = state.current_cancel_token();

    if token.is_cancelled() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            render_error("Server is shutting down"),
        );
    }

    // [DOC: docs/architecture/invariants.md#INV-004]
    tokio::task::spawn_blocking(move || {
        if token.is_cancelled() {
            return;
        }
        game_service.retrigger_event(ctx);
    });

    (
        StatusCode::OK,
        "<span class=\"status ready\">Retriggering...</span>".to_string(),
    )
}

/// [DOC: docs/architecture/system.md]
#[allow(clippy::expect_used)]
pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    axum::extract::Path((message_id, swipe_index)): axum::extract::Path<(u64, usize)>,
) -> axum::response::Response<Body> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return axum::response::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from(
                "<span class=\"status wait\">Generation in progress, please wait...</span>",
            ))
            .expect("static response body is valid");
    }

    let messages = match state.message_storage.load_messages() {
        Ok(msgs) => msgs,
        Err(e) => return internal_error_response(format!("Failed to load messages: {e}")),
    };

    let is_last = messages.last().map(|m| m.id == message_id).unwrap_or(false);
    if !is_last {
        return axum::response::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "<span class=\"status error\">Only the last message can be swiped</span>",
            ))
            .expect("static response body is valid");
    }

    if let Err(e) = state
        .message_storage
        .update_active_swipe(message_id, swipe_index)
    {
        return internal_error_response(format!("Failed to update active swipe: {e}"));
    }

    let target_msg = match messages.iter().find(|m| m.id == message_id) {
        Some(m) => m,
        None => return internal_error_response("Message not found"),
    };

    let target_swipe = match target_msg.swipes.get(swipe_index) {
        Some(s) => s,
        None => return internal_error_response("Swipe index out of bounds"),
    };

    let Some(snapshot_id) = target_swipe.snapshot_id else {
        return internal_error_response("Swipe has no associated snapshot");
    };

    let mut snapshot = match state.snapshot_storage.load_by_id(snapshot_id) {
        Ok(Some(s)) => s,
        Ok(None) => return internal_error_response("Snapshot not found"),
        Err(e) => return internal_error_response(format!("Failed to load snapshot: {e}")),
    };

    snapshot.created_at = chrono::Utc::now();
    if let Err(e) = state.snapshot_storage.save(&snapshot) {
        return internal_error_response(format!("Failed to save state: {e}"));
    }

    match super::renderers::render_story_log(&state) {
        Ok(html) => axum::response::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(html))
            .expect("static response body is valid"),
        Err(e) => internal_error_response(format!("Failed to render story log: {e}")),
    }
}

pub(crate) fn build_fresh_initial_state(state: &AppState) -> GameState {
    let mut initial_state = GameState::new(
        Arc::clone(&state.world),
        Arc::clone(&state.map),
        Arc::clone(&state.player),
        (*state.npcs).values().cloned().collect(),
        state.world.starting_room_id.clone(),
    );

    // Re-inject scenario text so reset produces the same initial state as startup.
    if let Some(scenario) = state.world.default_scenario() {
        let room_name = crate::engine::logic::find_room_in_world_map(
            &initial_state,
            &state.world.starting_room_id,
        )
        .map(|r| r.name.clone())
        .unwrap_or_else(|| state.world.starting_room_id.clone());

        initial_state.narrative.pending_location = Some(room_name);
        let text = scenario.text.replace("{{user}}", &state.player.sheet.name);
        if !text.is_empty() {
            initial_state.add_log(text, None, crate::model::state::LogType::Narration);
        }

        // Re-populate npc_encounter_log and npcs_in_area from scenario NPCs.
        initial_state.init_scenario_npcs(scenario);
    }

    initial_state
}

/// [DOC: docs/system/game_flow.md]
#[allow(clippy::expect_used)]
pub async fn reset_handler(State(state): State<AppState>) -> axum::response::Response<Body> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return axum::response::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from(
                "<span class=\"status wait\">Generation in progress, please wait...</span>",
            ))
            .expect("static response body is valid");
    }

    state.current_cancel_token().cancel();

    let current_id = state.snapshot_storage.current_game_id();
    let world_name = state.world.name.clone();

    if let Err(e) = state.snapshot_storage.delete_game(current_id) {
        log::error!("Reset failed: could not delete current game: {e}");
        return internal_error_response(e.to_string());
    }

    let existing_names: Vec<String> = match state.snapshot_storage.list_games() {
        Ok(games) => games
            .into_iter()
            .filter(|g| g.world_name == world_name)
            .map(|g| g.name)
            .collect(),
        Err(e) => {
            log::error!("Reset failed: could not list games: {e}");
            return internal_error_response(e.to_string());
        }
    };

    let new_name = generate_game_name(&world_name, &existing_names);
    let new_id = match state.snapshot_storage.create_game(&world_name, &new_name) {
        Ok(id) => id,
        Err(e) => {
            log::error!("Reset failed: could not create new game: {e}");
            return internal_error_response(e.to_string());
        }
    };

    state.snapshot_storage.set_game_id(new_id);
    state.message_storage.set_game_id(new_id);

    let mut initial_state = build_fresh_initial_state(&state);

    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&initial_state);
    let snapshot_id = match state.snapshot_storage.save(&snapshot) {
        Ok(id) => id,
        Err(e) => {
            log::error!("Reset failed: failed to save initial snapshot: {e}");
            return internal_error_response(e.to_string());
        }
    };

    if let Some(msg) = initial_state.narrative.history.last_mut() {
        if msg.id == 0 {
            msg.snapshot_id = Some(snapshot_id);
            if let Err(e) = state.message_storage.insert_message(msg) {
                log::error!("Reset failed: failed to persist message: {e}");
                return internal_error_response(e.to_string());
            }
        }
    }

    // Reset generation flags so subsequent actions work after reset.
    state
        .is_generating
        .store(false, std::sync::atomic::Ordering::SeqCst);
    state.replace_cancel_token();

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("HX-Refresh", "true")
        .body(Body::empty())
        .expect("static response body is valid")
}
