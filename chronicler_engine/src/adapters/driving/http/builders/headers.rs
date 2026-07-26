//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Header fragment + status-swap header builders.

use askama::Template;

use axum::{body::Body, http::HeaderValue, response::Response};

use crate::adapters::driving::http::templates::HeaderTemplate;
use crate::error::{EngineError, Result};

pub(crate) fn render_header_unlocked(game_name: String) -> Result<String> {
    let template = HeaderTemplate { game_name };
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub(crate) fn add_status_swap_headers(response: &mut Response<Body>) {
    response
        .headers_mut()
        .insert("HX-Retarget", HeaderValue::from_static("#status-display"));
    response
        .headers_mut()
        .insert("HX-Reswap", HeaderValue::from_static("innerHTML"));
}
