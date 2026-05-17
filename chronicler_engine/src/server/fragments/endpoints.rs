use axum::{extract::State, response::Html};

use crate::server::AppState;

use super::renderers::{
    render_action_area, render_action_hints, render_character_headshots, render_header,
    render_llm_messages, render_story_log, render_visual_sidebar,
};

fn render_fragment<F>(state: &AppState, render: F, name: &str) -> Html<String>
where
    F: FnOnce(&AppState) -> crate::error::Result<String>,
{
    match render(state) {
        Ok(html) => Html(html),
        Err(e) => {
            log::error!("{name} failed: {e}");
            Html(super::renderers::render_error(&e.to_string()))
        }
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn header_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_header, "header_fragment")
}

/// [DOC: docs/system/game_flow.md]
pub async fn story_log_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_story_log, "story_log_fragment")
}

/// [DOC: docs/system/game_flow.md]
pub async fn visual_sidebar_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_visual_sidebar, "visual_sidebar_fragment")
}

/// [DOC: docs/system/game_flow.md]
pub async fn action_area_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_action_area, "action_area_fragment")
}

/// [DOC: docs/system/game_flow.md]
pub async fn character_headshots_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(
        &state,
        render_character_headshots,
        "character_headshots_fragment",
    )
}

/// [DOC: docs/system/game_flow.md]
pub async fn hints_handler(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_action_hints, "hints_handler")
}

pub async fn llm_messages_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_llm_messages, "llm_messages_fragment")
}

pub async fn status_ready_handler(State(_state): State<AppState>) -> Html<String> {
    Html("<span class=\"status ready\">Ready</span>".to_string())
}

/// [DOC: docs/system/game_flow.md]
pub async fn generating_status_handler(State(state): State<AppState>) -> Html<String> {
    let (status, phase) = match state.load_state() {
        Ok(guard) => (
            guard.narrative.input_buffer.status.clone(),
            guard.narrative.input_buffer.phase.clone(),
        ),
        _ => Default::default(),
    };

    if let Some(err) = status.error_message() {
        Html(format!("<span class=\"status error\">Error: {err}</span>"))
    } else if status.is_generating() {
        Html(phase.as_endpoint_str().to_string())
    } else {
        Html("idle".to_string())
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn reset_generating_handler(State(state): State<AppState>) -> Html<String> {
    let result = match state.load_state() {
        Ok(mut guard) => {
            guard.narrative.input_buffer.status = crate::model::state::GenerationStatus::Idle;
            let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&guard);
            state.snapshot_storage.save(&snapshot).is_ok()
        }
        Err(_) => false,
    };

    if result {
        Html("reset".to_string())
    } else {
        Html("failed".to_string())
    }
}
