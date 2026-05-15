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
        let mut guard = state.load_state()?;
        guard.delete_last_log()?;
        let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
            &guard,
            guard.narrative.current_turn_id.clone(),
            0,
        );
        state.snapshot_storage.save(&snapshot)
    })();

    match result {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}
