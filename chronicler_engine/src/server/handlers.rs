//! [DOC: docs/system/dashboard.md]

use axum::response::Html;

/// Serves the static index.html file.
pub async fn index_handler() -> Html<String> {
    Html(include_str!("../../assets/index.html").to_string())
}
