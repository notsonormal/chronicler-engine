//! [DOC: docs/system/dashboard.md]
//! Game control fragment handlers

use axum::{body::Body, extract::State};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::fragments::renderers::{
    internal_error, ok_refresh, service_unavailable_generating,
};
use crate::adapters::driving::http::op_context_loader::load_op_context_for_active_game;

pub async fn reset_handler(State(state): State<AppState>) -> axum::response::Response<Body> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return service_unavailable_generating();
    }

    state.current_cancel_token().cancel();

    let ctx = match load_op_context_for_active_game(&state) {
        Ok(ctx) => ctx,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };
    match state.application_service.reset(ctx) {
        Ok(()) => {
            state
                .is_generating
                .store(false, std::sync::atomic::Ordering::SeqCst);
            state.replace_cancel_token();
            ok_refresh()
        }
        Err(e) => internal_error(e.to_string()),
    }
}
