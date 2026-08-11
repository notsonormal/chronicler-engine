//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment-rendering glue: uniform try-render / log-error wrapper for AppState renderers.

use axum::response::Html;

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::utils::error::render_error;

pub fn render_fragment<F>(state: &AppState, render: F, name: &str) -> Html<String>
where
    F: FnOnce(&AppState) -> crate::error::Result<String>,
{
    match render(state) {
        Ok(html) => Html(html),
        Err(e) => {
            tracing::error!("{name} failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}
