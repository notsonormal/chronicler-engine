//! [DOC: docs/system/dashboard.md]
//! Text check fragment handler

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
};

use serde::Deserialize;

use crate::model::settings::TextCheckMode;
use crate::narrative::text_check::check_player_input;
use crate::server::AppState;
use crate::server::fragments::renderers::{bad_request, internal_error, ok};
use crate::server::templates::TextCheckPreviewTemplate;

#[derive(Deserialize)]
pub struct CheckTextForm {
    pub command: String,
}

#[allow(clippy::expect_used)]
pub async fn check_text_handler(
    State(state): State<AppState>,
    Form(form): Form<CheckTextForm>,
) -> axum::response::Response<Body> {
    let text = form.command.trim().to_string();
    if text.is_empty() {
        return bad_request("<span class=\"status error\">Enter text to check</span>");
    }

    let settings = state.settings();

    if settings.text_check.mode == TextCheckMode::Disabled {
        return ok("<span class=\"status ready\">Text check is disabled</span>");
    }

    match check_player_input(
        &text,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(Some(result)) => {
            let template = TextCheckPreviewTemplate::from_check_result(&result);
            match template.render() {
                Ok(html) => ok(html),
                Err(e) => internal_error(format!("Template error: {e}")),
            }
        }
        Ok(None) => ok("<span class=\"status ready\">No issues found</span>"),
        Err(e) => internal_error(format!("Check failed: {e}")),
    }
}
