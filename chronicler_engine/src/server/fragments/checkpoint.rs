use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};

use crate::model::checkpoint::Checkpoint;
use crate::server::AppState;

use super::renderers::render_error;

/// [DOC: docs/system/game_flow.md]
pub async fn create_checkpoint_handler(State(state): State<AppState>) -> (StatusCode, String) {
    let result: Result<(), crate::error::EngineError> = (|| {
        let latest = state.snapshot_storage.load_latest()?.ok_or_else(|| {
            crate::error::EngineError::Internal(crate::error::internal_error(
                "No state to checkpoint".to_string(),
            ))
        })?;
        let snapshot_id = latest.db_id.ok_or_else(|| {
            crate::error::EngineError::Internal(crate::error::internal_error(
                "Snapshot has no database ID".to_string(),
            ))
        })?;
        let checkpoint = Checkpoint {
            id: uuid::Uuid::new_v4().to_string(),
            snapshot_id,
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
    let result: Result<(), crate::error::EngineError> = (|| {
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
            .load_by_id(checkpoint.snapshot_id)?
            .ok_or_else(|| {
                crate::error::EngineError::Internal(crate::error::internal_error(
                    "Checkpoint snapshot not found".to_string(),
                ))
            })?;
        let mut game_state = crate::model::state::GameState::from_snapshot(
            &snapshot,
            state.world.clone(),
            state.map.clone(),
            state.player.clone(),
            (*state.npcs).clone(),
        );
        if let Ok(messages) = state.message_storage.load_messages() {
            game_state.narrative.messages = messages;
        }
        let new_snapshot =
            crate::model::state_snapshot::GameStateSnapshot::from_game_state(&game_state);
        state.snapshot_storage.save(&new_snapshot)?;
        Ok(())
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
                    <span class="checkpoint-meta">Snapshot {}</span>
                    <button class="checkpoint-restore" hx-post="/checkpoint/{}/restore" hx-swap="none">Restore</button>
                    <button class="checkpoint-delete" hx-post="/checkpoint/{}/delete" hx-target="closest .checkpoint-item" hx-swap="outerHTML">×</button>
                </div>"#,
                cp.id, cp.name, cp.snapshot_id, cp.id, cp.id
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
