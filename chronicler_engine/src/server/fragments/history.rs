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
    let result: Result<(), crate::error::EngineError> = (|| {
        let latest = state.snapshot_storage.load_latest()?;
        let mut guard = state.load_state()?;
        guard.narrative.history.edit(id, form.text.clone())?;
        if latest.is_some() {
            let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&guard);
            state.snapshot_storage.save(&snapshot)?;
            state.message_storage.update_message(id, &form.text)?;
        }
        Ok(())
    })();

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
    let result: Result<(), crate::error::EngineError> = (|| {
        let mut guard = state.load_state()?;
        let last_id = guard
            .narrative
            .history
            .last()
            .map(|m| m.id)
            .ok_or_else(|| {
                crate::error::EngineError::Internal(crate::error::internal_error(
                    "History is empty".to_string(),
                ))
            })?;
        guard.narrative.history.delete_last()?;
        let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&guard);
        state.snapshot_storage.save(&snapshot)?;
        state.message_storage.delete_message(last_id)?;
        Ok(())
    })();

    match result {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}
