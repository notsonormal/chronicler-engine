use axum::{extract::Form, extract::State, http::StatusCode};

use crate::server::AppState;

use super::renderers::render_error;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    pub text: String,
}

/// [DOC: docs/system/game_flow.md]
pub async fn edit_history_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    Form(form): Form<EditHistoryForm>,
) -> (StatusCode, String) {
    let result = match state.load_state() {
        Ok(mut guard) => {
            let result = guard.edit_log(id, form.text);
            if result.is_ok() {
                let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                    &guard,
                    uuid::Uuid::new_v4().to_string(),
                    0,
                );
                let _ = state.snapshot_storage.save(&snapshot);
            }
            result
        }
        Err(e) => Err(e),
    };

    match result {
        Ok(()) => (
            StatusCode::OK,
            "<span class=\"status ready\">Edited</span>".to_string(),
        ),
        Err(e) => (StatusCode::NOT_FOUND, render_error(&e.to_string())),
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn delete_history_handler(State(state): State<AppState>) -> (StatusCode, String) {
    let result = match state.load_state() {
        Ok(mut guard) => {
            let result = guard.delete_last_log();
            if result.is_ok() {
                let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                    &guard,
                    uuid::Uuid::new_v4().to_string(),
                    0,
                );
                let _ = state.snapshot_storage.save(&snapshot);
            }
            result
        }
        Err(e) => Err(e),
    };

    match result {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}
