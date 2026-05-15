use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};

use crate::model::checkpoint::Checkpoint;
use crate::server::AppState;

use super::renderers::render_error;

/// [DOC: docs/system/game_flow.md]
pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    Path((turn_id, swipe_index)): Path<(String, u32)>,
) -> (StatusCode, String) {
    let result = match state.load_state() {
        Ok(mut guard) => {
            let msg = guard
                .narrative
                .messages
                .iter_mut()
                .rev()
                .find(|m| m.turn_id == turn_id);
            match msg {
                Some(m) if (swipe_index as usize) < m.swipes.len() => {
                    m.active_swipe_index = swipe_index;
                    m.text = m.active_text().to_string();
                    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                        &guard,
                        turn_id,
                        swipe_index,
                    );
                    state.snapshot_storage.save(&snapshot)
                }
                _ => Err(crate::error::EngineError::Internal(
                    crate::error::internal_error("Invalid swipe index".to_string()),
                )),
            }
        }
        Err(e) => Err(e),
    };

    match result {
        Ok(()) => (
            StatusCode::OK,
            "<span class=\"status ready\">Switched</span>".to_string(),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn create_checkpoint_handler(State(state): State<AppState>) -> (StatusCode, String) {
    let result = (|| {
        let latest = state.snapshot_storage.load_latest(None)?.ok_or_else(|| {
            crate::error::EngineError::Internal(crate::error::internal_error(
                "No state to checkpoint".to_string(),
            ))
        })?;
        let checkpoint = Checkpoint {
            id: uuid::Uuid::new_v4().to_string(),
            turn_id: latest.turn_id,
            swipe_index: latest.swipe_index,
            name: format!("Checkpoint {}", chrono::Utc::now().format("%H:%M:%S")),
            created_at: chrono::Utc::now(),
        };
        state.snapshot_storage.save_checkpoint(&checkpoint)
    })();

    match result {
        Ok(()) => (
            StatusCode::OK,
            "<span class=\"status ready\">Checkpoint saved</span>".to_string(),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn restore_checkpoint_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, String) {
    let result = (|| {
        let checkpoint = state
            .snapshot_storage
            .load_checkpoint(&id)?
            .ok_or_else(|| {
                crate::error::EngineError::Internal(crate::error::internal_error(
                    "Checkpoint not found".to_string(),
                ))
            })?;
        let snapshot = state
            .snapshot_storage
            .load_by_turn(&checkpoint.turn_id, checkpoint.swipe_index)?
            .ok_or_else(|| {
                crate::error::EngineError::Internal(crate::error::internal_error(
                    "Checkpoint snapshot not found".to_string(),
                ))
            })?;
        let game_state = crate::model::state::GameState::from_snapshot(
            &snapshot,
            state.world.clone(),
            state.map.clone(),
            state.player.clone(),
            (*state.npcs).clone(),
        );
        // Snapshot already contains correct active_swipe_index for all messages.
        // No manual adjustment needed in the Message model.
        let new_snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
            &game_state,
            checkpoint.turn_id,
            checkpoint.swipe_index,
        );
        state.snapshot_storage.save(&new_snapshot)
    })();

    match result {
        Ok(()) => (
            StatusCode::OK,
            "<span class=\"status ready\">Restored</span>".to_string(),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn delete_checkpoint_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, String) {
    match state.snapshot_storage.delete_checkpoint(&id) {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn list_checkpoints_fragment(
    State(state): State<AppState>,
) -> Response<axum::body::Body> {
    let checkpoints = match state.snapshot_storage.list_checkpoints() {
        Ok(cps) => cps,
        Err(e) => {
            return match Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from(render_error(&e.to_string())))
            {
                Ok(r) => r,
                Err(_) => Response::new(axum::body::Body::from("Internal error")),
            };
        }
    };

    let html = checkpoints
        .iter()
        .map(|cp| {
            format!(
                r#"<div class="checkpoint-item" data-id="{}">
                    <span class="checkpoint-name">{}</span>
                    <span class="checkpoint-meta">Turn {} | Swipe {}</span>
                    <button class="checkpoint-restore" hx-post="/checkpoint/{}/restore" hx-swap="none">Restore</button>
                    <button class="checkpoint-delete" hx-post="/checkpoint/{}/delete" hx-target="closest .checkpoint-item" hx-swap="outerHTML">×</button>
                </div>"#,
                cp.id, cp.name, &cp.turn_id[..8.min(cp.turn_id.len())], cp.swipe_index, cp.id, cp.id
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    match Response::builder()
        .status(StatusCode::OK)
        .body(axum::body::Body::from(html))
    {
        Ok(r) => r,
        Err(_) => Response::new(axum::body::Body::from("Internal error")),
    }
}
