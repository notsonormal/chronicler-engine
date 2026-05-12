use std::sync::Arc;

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    http::StatusCode,
};

use crate::model::settings::TextCheckMode;
use crate::model::state::GameState;
use crate::narrative::text_check::check_player_input;
use crate::server::AppState;
use crate::server::templates::TextCheckPreviewTemplate;

use super::renderers::render_error;

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
    let snapshot = match state.snapshot_storage.load_latest(None) {
        Ok(Some(s)) => s,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                render_error("Failed to load state"),
            );
        }
    };

    let mut game_state = GameState::from_snapshot(
        &snapshot,
        Arc::clone(&state.world),
        Arc::clone(&state.map),
        Arc::clone(&state.player),
        (*state.npcs).clone(),
    );

    if game_state.get_last_input_text().is_none() {
        return (StatusCode::BAD_REQUEST, render_error("No input to retry"));
    }

    game_state.narrative.generation.status = crate::model::state::GenerationStatus::Generating;
    game_state.narrative.generation.phase = crate::model::state::GenerationPhase::Narrating;
    let new_swipe = snapshot.swipe_index + 1;
    let generating_snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &game_state,
        snapshot.message_id.clone(),
        new_swipe,
    );
    if let Err(e) = state.snapshot_storage.save(&generating_snapshot) {
        log::error!("Failed to save retry snapshot: {e}");
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

/// [DOC: docs/system/game_flow.md]
#[allow(clippy::expect_used)]
pub async fn reset_handler(State(state): State<AppState>) -> axum::response::Response<Body> {
    state.current_cancel_token().cancel();

    if let Err(e) = state.snapshot_storage.reset() {
        log::error!("Reset failed: {e}");
        return axum::response::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(render_error(&e.to_string())))
            .expect("static response body is valid");
    }

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

        // Re-populate character_state and npcs_in_area from scenario NPCs.
        initial_state.init_scenario_npcs(scenario);
    }

    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &initial_state,
        "initial".to_string(),
        0,
    );
    let _ = state.snapshot_storage.save(&snapshot);

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
