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
    let result = (|| {
        let latest = state.snapshot_storage.load_latest(None)?;
        let (turn_id, swipe_index) = match latest {
            Some(s) => (s.turn_id, s.swipe_index),
            None => (String::new(), 0),
        };
        let mut guard = state.load_state()?;
        let result = guard.edit_log(id, form.text);
        if result.is_ok() && !turn_id.is_empty() {
            let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                &guard,
                turn_id,
                swipe_index,
            );
            let _ = state.snapshot_storage.save(&snapshot);
        }
        result
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
    let result = (|| {
        let _latest = state.snapshot_storage.load_latest(None)?;
        let mut guard = state.load_state()?;
        match guard.delete_last_turn() {
            Some(removed_turn_id) => {
                let _ = state
                    .snapshot_storage
                    .delete_turn_snapshots(&removed_turn_id);
                let new_turn_id = guard
                    .narrative
                    .turns
                    .last()
                    .map(|t| t.id.clone())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                    &guard,
                    new_turn_id,
                    0,
                );
                state.snapshot_storage.save(&snapshot)
            }
            None => Err(crate::error::EngineError::Internal(
                crate::error::internal_error("History is empty".to_string()),
            )),
        }
    })();

    match result {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}
