//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Header fragment + status-swap header builders.

use askama::Template;

use axum::{body::Body, http::HeaderValue, response::Response};

use crate::adapters::driving::http::templates::HeaderTemplate;
use crate::adapters::driving::http::utils::error::{error_disclosure, raw_error_detail};
use crate::adapters::driving::http::utils::response::html_escape;
use crate::application::games::view_query::RoleHealth;
use crate::application::ports::llm_provider::{
    AGENT_NARRATOR, AGENT_OPTIONS, AGENT_QUANTIFIER, AGENT_TRIGGER,
};
use crate::error::{EngineError, Result};

fn role_effect(role: &str) -> Option<&'static str> {
    match role {
        AGENT_NARRATOR => Some("the turn was left unnarrated"),
        AGENT_QUANTIFIER => Some("using fallback NPC IDs"),
        AGENT_OPTIONS => Some("showing no options"),
        AGENT_TRIGGER => Some("skipping the trigger"),
        _ => None,
    }
}

fn banner_message(degraded: &[&RoleHealth]) -> String {
    degraded
        .iter()
        .map(|r| match role_effect(&r.role) {
            Some(effect) => format!("{} failed — {effect}", r.label),
            None => format!("{} failed", r.label),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn banner_detail(roles: &[RoleHealth]) -> String {
    roles
        .iter()
        .map(|r| {
            let class = if r.last_error.is_some() {
                "error-role degraded"
            } else {
                "error-role"
            };
            let backend = html_escape(r.backend_model.as_deref().unwrap_or("no recorded call"));
            let body = match &r.last_error {
                Some(error) => raw_error_detail(error),
                None => {
                    "<span class=\"error-role-ok\">last call succeeded</span>".to_string()
                }
            };
            format!(
                "<div class=\"{class}\"><div class=\"error-role-header\"><span class=\"error-role-name\">{name}</span><span class=\"error-role-backend\">{backend}</span></div>{body}</div>",
                name = html_escape(&r.label),
            )
        })
        .collect()
}

pub(crate) fn render_header_unlocked(game_name: String, roles: &[RoleHealth]) -> Result<String> {
    let template = HeaderTemplate { game_name };
    let header = template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))?;

    let degraded: Vec<&RoleHealth> = roles.iter().filter(|r| r.last_error.is_some()).collect();
    let banner = if degraded.is_empty() {
        String::new()
    } else {
        error_disclosure(
            "failure-banner-popover",
            &banner_message(&degraded),
            &banner_detail(roles),
        )
    };

    Ok(format!(
        "{header}<div id=\"failure-banner-degraded\" hx-swap-oob=\"true\">{banner}</div>"
    ))
}

pub fn add_status_swap_headers(response: &mut Response<Body>) {
    response
        .headers_mut()
        .insert("HX-Retarget", HeaderValue::from_static("#status-display"));
    response
        .headers_mut()
        .insert("HX-Reswap", HeaderValue::from_static("innerHTML"));
}
