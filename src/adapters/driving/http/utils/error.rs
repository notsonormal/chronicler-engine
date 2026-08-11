//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Error rendering helpers for HTTP fragments.

use crate::adapters::driving::http::utils::response::html_escape;

pub fn render_error(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}
