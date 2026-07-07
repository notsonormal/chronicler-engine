//! [DOC: docs/system/dashboard.md]
//! Action fragment handlers

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::application::application_service::ProcessActionResult;
use crate::application::context::OpContext;
use crate::domain::model::settings::TextCheckMode;
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::templates::TextCheckPreviewTemplate;

use super::renderers::{internal_error, ok, render_action_area, render_error, service_unavailable};

#[derive(Deserialize, Serialize)]
pub struct ActionForm {
    pub command: String,
}

async fn dispatch_action(state: &AppState, ctx: OpContext, command: String) -> Response<Body> {
    let action_result = if command.is_empty() {
        state.application_service.continue_narration(ctx)
    } else {
        state.application_service.process_action(ctx, command)
    };

    match action_result {
        Ok(ProcessActionResult::Started) => {
            ok("<span class=\"status thinking\">Thinking...</span>")
        }
        Ok(ProcessActionResult::ConcurrentGeneration) => {
            ok("<span class=\"status wait\">Still thinking...</span>")
        }
        Ok(ProcessActionResult::ShuttingDown) => {
            service_unavailable(render_error("Server is shutting down"))
        }
        Err(e) => internal_error(render_error(&format!("Failed to process action: {e}"))),
    }
}

pub async fn action_handler(
    State(state): State<AppState>,
    ctx: OpContext,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();
    dispatch_action(&state, ctx, command).await
}

#[allow(clippy::expect_used)]
pub async fn action_confirm_handler(
    State(state): State<AppState>,
    ctx: OpContext,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();

    let action_response = dispatch_action(&state, ctx, command).await;
    let status = action_response.status();

    let action_area_html = match render_action_area(&state) {
        Ok(html) => html,
        Err(e) => {
            tracing::error!("Failed to render action area: {e}");
            render_error(&e.to_string())
        }
    };

    Response::builder()
        .status(status)
        .body(Body::from(action_area_html))
        .expect("static response body is valid")
}

pub async fn action_check_handler(
    State(state): State<AppState>,
    ctx: OpContext,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();

    let settings = state.settings();

    if settings.text_check.mode == TextCheckMode::Disabled || !settings.text_check.enable_auto_check
    {
        let mut response = dispatch_action(&state, ctx, command).await;
        add_status_swap_headers(&mut response);
        return response;
    }

    let result = match state.text_check_service().check_player_input(
        &command,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Text check failed: {e}");
            let mut response = dispatch_action(&state, ctx, command).await;
            add_status_swap_headers(&mut response);
            return response;
        }
    };

    match result {
        Some(check_result) => {
            let template = TextCheckPreviewTemplate::from_check_result(&check_result);
            match template.render() {
                Ok(html) => ok(html),
                Err(e) => internal_error(render_error(&format!("Template error: {e}"))),
            }
        }
        None => {
            let mut response = dispatch_action(&state, ctx, command).await;
            add_status_swap_headers(&mut response);
            response
        }
    }
}

#[allow(clippy::expect_used)]
fn add_status_swap_headers(response: &mut Response<Body>) {
    response.headers_mut().insert(
        "HX-Retarget",
        "#status-display"
            .parse()
            .expect("static header value is valid"),
    );
    response.headers_mut().insert(
        "HX-Reswap",
        "innerHTML".parse().expect("static header value is valid"),
    );
}
