//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Header fragment + status-swap header builders.

use askama::Template;

use axum::{body::Body, http::HeaderValue, response::Response};

use crate::adapters::driving::http::templates::HeaderTemplate;
use crate::adapters::driving::http::utils::error::{error_disclosure, raw_error_detail};
use crate::adapters::driving::http::utils::response::html_escape;
use crate::application::games::view_query::RoleHealth;
use crate::domain::model::agent::Role;
use crate::error::{EngineError, Result};

pub fn role_label(role: Role) -> &'static str {
    match role {
        Role::Narrator => "Narrator",
        Role::Quantifier => "Quantifier",
        Role::Options => "Options",
        Role::Trigger => "Trigger",
    }
}

pub fn role_failure_effect(role: Role) -> &'static str {
    match role {
        Role::Narrator => "the turn was left unnarrated",
        Role::Quantifier => "using fallback NPC IDs",
        Role::Options => "showing no options",
        Role::Trigger => "skipping the trigger",
    }
}

/// The sentence one failed role contributes to the failure banner and to its
/// own Settings row. Role health covers every game, so a failure can outlive
/// the game it came from until the role's next call succeeds.
pub fn role_failure_sentence(role: Role) -> String {
    format!(
        "{} failed — {}",
        role_label(role),
        role_failure_effect(role)
    )
}

fn banner_message(degraded: &[&RoleHealth]) -> String {
    degraded
        .iter()
        .map(|r| role_failure_sentence(r.role))
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
                name = html_escape(role_label(r.role)),
            )
        })
        .collect()
}

/// The header fragment plus the out-of-band failure banner it swaps in. The
/// banner names each degraded role and states that role health is engine-wide.
pub fn header_fragment_html(game_name: String, roles: &[RoleHealth]) -> Result<String> {
    let template = HeaderTemplate { game_name };
    let header = template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))?;

    let degraded: Vec<&RoleHealth> = roles.iter().filter(|r| r.last_error.is_some()).collect();
    let banner = if degraded.is_empty() {
        String::new()
    } else {
        let summary = format!("{} (engine-wide role health)", banner_message(&degraded));
        error_disclosure("failure-banner-popover", &summary, &banner_detail(roles))
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
