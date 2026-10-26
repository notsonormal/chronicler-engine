//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Header and failure-banner fragment builders, and the Settings role-health cell builders.

use askama::Template;

use crate::adapters::driving::http::templates::HeaderTemplate;
use crate::adapters::driving::http::utils::error::{error_disclosure, raw_error_detail};
use crate::adapters::driving::http::utils::response::html_escape;
use crate::adapters::driving::http::view_models::SafeHtml;
use crate::application::games::view_query::RoleHealth;
use crate::domain::model::agent::Role;
use crate::error::{EngineError, Result};

/// Options and Trigger have no Settings row, so their health appears only in the
/// failure banner.
const SETTINGS_ROLE_ROWS: [Role; 2] = [Role::Narrator, Role::Quantifier];

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
        Role::Narrator => "no narration was written",
        Role::Quantifier => "keeping the Characters present",
        Role::Options => "showing no options",
        Role::Trigger => "skipping the trigger",
    }
}

/// Role health is engine-wide, so a failure can outlive the game it came from until
/// the role's next call succeeds.
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

#[derive(Clone, Copy)]
enum RoleHealthState<'a> {
    /// The newest attempt failed, carrying its raw failure text.
    Failed(&'a str),
    /// The newest attempt succeeded.
    Succeeded,
    /// No attempt has been recorded, which is not a success — the role has
    /// never run.
    NoCalls,
}

impl<'a> RoleHealthState<'a> {
    fn classify(health: Option<&'a RoleHealth>) -> Self {
        match health.map(|health| (&health.last_error, &health.backend_model)) {
            Some((Some(error), _)) => RoleHealthState::Failed(error),
            Some((None, Some(_))) => RoleHealthState::Succeeded,
            _ => RoleHealthState::NoCalls,
        }
    }

    fn is_failed(self) -> bool {
        matches!(self, RoleHealthState::Failed(_))
    }
}

fn role_health_line(state: RoleHealthState<'_>) -> SafeHtml {
    match state {
        RoleHealthState::Failed(error) => SafeHtml::new(raw_error_detail(error)),
        RoleHealthState::Succeeded => {
            SafeHtml::new(r#"<span class="error-role-ok">last call succeeded</span>"#.to_string())
        }
        RoleHealthState::NoCalls => {
            SafeHtml::new(r#"<span class="error-role-nocalls">No calls yet</span>"#.to_string())
        }
    }
}

fn banner_detail(roles: &[RoleHealth]) -> String {
    roles
        .iter()
        .map(|r| {
            let health = RoleHealthState::classify(Some(r));
            let class = if health.is_failed() {
                "error-role degraded"
            } else {
                "error-role"
            };
            let backend = html_escape(r.backend_model.as_deref().unwrap_or("no recorded call"));
            format!(
                "<div class=\"{class}\"><div class=\"error-role-header\"><span class=\"error-role-name\">{name}</span><span class=\"error-role-backend\">{backend}</span></div>{body}</div>",
                name = html_escape(role_label(r.role)),
                body = role_health_line(health),
            )
        })
        .collect()
}

pub fn role_health_cell(role: Role, health: Option<&RoleHealth>) -> SafeHtml {
    match RoleHealthState::classify(health) {
        RoleHealthState::Failed(error) => SafeHtml::new(format!(
            "<span class=\"role-health degraded\">Degraded</span>{}",
            error_disclosure(
                &format!("role-health-{}-popover", role.agent_name()),
                &role_failure_sentence(role),
                &raw_error_detail(error),
            )
        )),
        RoleHealthState::Succeeded => {
            SafeHtml::new(r##"<span class="role-health healthy">Healthy</span>"##.to_string())
        }
        RoleHealthState::NoCalls => no_calls_cell(),
    }
}

fn no_calls_cell() -> SafeHtml {
    SafeHtml::new(r##"<span class="role-health unknown">No calls yet</span>"##.to_string())
}

pub fn role_health_cell_id(role: Role) -> String {
    format!("role-health-{}", role.agent_name())
}

pub fn connections_degraded(roles: &[RoleHealth]) -> bool {
    roles
        .iter()
        .any(|health| SETTINGS_ROLE_ROWS.contains(&health.role) && health.last_error.is_some())
}

pub fn connections_degraded_marker(degraded: bool) -> String {
    if !degraded {
        return String::new();
    }
    r##"<span class="subtab-degraded-marker" aria-hidden="true" title="A role is degraded (engine-wide role health)"><svg class="icon" aria-hidden="true"><use href="#i-triangle-alert"/></svg></span>"##
        .to_string()
}

/// The header part holds the game name only. The engine-wide failure banner and
/// the Settings role-health cells are parts of their own.
pub fn header_fragment_html(game_name: String) -> Result<String> {
    HeaderTemplate { game_name }
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

/// The engine-wide failure banner: empty while every role is healthy.
pub fn failure_banner_html(roles: &[RoleHealth]) -> String {
    let degraded: Vec<&RoleHealth> = roles.iter().filter(|r| r.last_error.is_some()).collect();
    if degraded.is_empty() {
        return String::new();
    }
    let summary = format!("{} (engine-wide role health)", banner_message(&degraded));
    error_disclosure("failure-banner-popover", &summary, &banner_detail(roles))
}
