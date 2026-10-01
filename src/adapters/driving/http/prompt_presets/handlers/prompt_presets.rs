//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset handlers

use axum::body::Body;
use axum::{Form, extract::State, response::Html, response::IntoResponse, response::Response};

use crate::application::errors::ApplicationError;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::NarratorMode;
use crate::domain::model::utils::settings_defaults;
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::presets::{
    preset_card_html, preset_edit_form_html, preset_view_form_html,
};
use crate::adapters::driving::http::utils::error::render_error;
use crate::adapters::driving::http::utils::handler_helpers::{
    generate_preset_id, parse_preset_type, render_template,
};
use crate::adapters::driving::http::utils::response::bad_request;

use crate::adapters::driving::http::prompt_presets::templates::prompt_presets::{
    ModeActiveIds, PromptPresetsTemplate,
};

/// Per-mode active preset ids for the panel, derived from the mode registry.
fn mode_active_ids(
    settings: &crate::domain::model::settings::AppSettings,
) -> (ModeActiveIds, ModeActiveIds, ModeActiveIds) {
    let novel = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    let interactive_fiction = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::InteractiveFiction);
    (
        ModeActiveIds {
            novel: novel.system_prompt_preset_id.clone(),
            interactive_fiction: interactive_fiction.system_prompt_preset_id.clone(),
        },
        ModeActiveIds {
            novel: novel.quantifier_prompt_preset_id.clone(),
            interactive_fiction: interactive_fiction.quantifier_prompt_preset_id.clone(),
        },
        ModeActiveIds {
            novel: novel.impersonate_prompt_preset_id.clone(),
            interactive_fiction: interactive_fiction.impersonate_prompt_preset_id.clone(),
        },
    )
}

macro_rules! require_preset {
    ($storage:expr, $id:expr) => {
        match $storage.get_preset($id) {
            Ok(Some(p)) => p,
            Ok(None) => {
                return Html("<span class='error'>Preset not found</span>".to_string())
                    .into_response();
            }
            Err(e) => {
                return Html(format!("<span class='error'>Load failed: {e}</span>"))
                    .into_response();
            }
        }
    };
}

pub async fn preset_card_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => {
            return Html(format!("<span class='error'>Load failed: {e}</span>")).into_response();
        }
    };
    let novel_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    let if_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::InteractiveFiction);
    Html(preset_card_html(&preset, &novel_bundle, &if_bundle)).into_response()
}

pub async fn view_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    Html(preset_view_form_html(&preset)).into_response()
}

pub async fn panel_handler(State(app_state): State<AppState>) -> Html<String> {
    let system_presets = app_state
        .prompt_preset_service
        .list_presets(PresetType::System)
        .unwrap_or_default();
    let quantifier_presets = app_state
        .prompt_preset_service
        .list_presets(PresetType::Quantifier)
        .unwrap_or_default();
    let impersonate_presets = app_state
        .prompt_preset_service
        .list_presets(PresetType::Impersonate)
        .unwrap_or_default();

    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => return Html(format!("<span class='error'>Load failed: {e}</span>")),
    };
    let (active_system, active_quantifier, active_impersonate) = mode_active_ids(&settings);

    render_template(PromptPresetsTemplate {
        system_presets,
        quantifier_presets,
        impersonate_presets,
        active_system,
        active_quantifier,
        active_impersonate,
    })
}

/// Checkbox fields encode the mode flags: a checked box posts
/// `value="true"`, an unchecked box posts nothing (serde default false).
///
/// axum's `Form` (serde_urlencoded) cannot deserialize `Vec<String>` from
/// checkbox groups — a single checked box yields one scalar value — so the
/// flags are two independent booleans instead of a repeated field.
#[derive(Debug, Default, serde::Deserialize)]
pub struct PresetForm {
    pub name: String,
    pub role: Option<String>,
    pub instructions: Option<String>,
    pub writing_style: Option<String>,
    pub output_format: Option<String>,
    pub preset_type: String,
    #[serde(default)]
    pub allowed_mode_novel: bool,
    #[serde(default)]
    pub allowed_mode_if: bool,
}

fn form_allowed_modes(novel: bool, if_mode: bool) -> Vec<NarratorMode> {
    let mut modes = Vec::new();
    if novel {
        modes.push(NarratorMode::Novel);
    }
    if if_mode {
        modes.push(NarratorMode::InteractiveFiction);
    }
    modes
}

/// A refusal reaches the user as a 400, which the shell shows as a toast and
/// leaves the panel in place. Other errors keep the existing in-fragment
/// error rendering.
fn preset_save_error_response(error: ApplicationError, prefix: &str) -> Response<Body> {
    match error {
        ApplicationError::Validation(message) => bad_request(render_error(&message)),
        other => Html(format!("<span class='error'>{prefix}: {other}</span>")).into_response(),
    }
}

impl PresetForm {
    fn into_preset(
        self,
        id: String,
        preset_type: PresetType,
        allowed_modes: Vec<NarratorMode>,
    ) -> PromptPreset {
        PromptPreset {
            id,
            name: self.name,
            role: self.role,
            instructions: self.instructions,
            writing_style: self.writing_style,
            output_format: self.output_format,
            allowed_modes,
            is_default: false,
            preset_type,
        }
    }
}

pub async fn save_preset_handler(
    State(app_state): State<AppState>,
    Form(form): Form<PresetForm>,
) -> Response<Body> {
    let preset_type = match parse_preset_type(&form.preset_type) {
        Some(pt) => pt,
        None => {
            return Html("<span class='error'>Invalid preset type</span>".to_string())
                .into_response();
        }
    };

    // Create forms render no checkboxes; both-false means "flags not
    // offered" → default to both modes.
    let allowed_modes = if form.allowed_mode_novel || form.allowed_mode_if {
        form_allowed_modes(form.allowed_mode_novel, form.allowed_mode_if)
    } else {
        settings_defaults::default_allowed_modes()
    };

    let preset = form.into_preset(generate_preset_id(), preset_type, allowed_modes);

    if let Err(e) = app_state.prompt_preset_service.save_preset(&preset) {
        return preset_save_error_response(e, "Save failed");
    }

    panel_handler(State(app_state)).await.into_response()
}

pub async fn edit_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    if preset.is_default {
        return Html("<span class='error'>Cannot edit default presets</span>".to_string())
            .into_response();
    }

    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => {
            return Html(format!("<span class='error'>Load failed: {e}</span>")).into_response();
        }
    };
    let novel_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    let is_active = preset.preset_type.bundle_slot(&novel_bundle) == id;

    Html(preset_edit_form_html(
        &preset,
        preset.preset_type.as_str(),
        is_active,
    ))
    .into_response()
}

pub async fn update_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<PresetForm>,
) -> Response<Body> {
    let mut preset = require_preset!(app_state.prompt_preset_service, &id);

    if preset.is_default {
        return Html("<span class='error'>Cannot edit default presets</span>".to_string())
            .into_response();
    }

    // preset_type is fixed at creation, so the form's hidden input is
    // ignored. Both-false = flags absent — urlencoded checkboxes cannot
    // distinguish "unchecked all" from "field omitted", so preserve.
    let allowed_modes = if !form.allowed_mode_novel && !form.allowed_mode_if {
        preset.allowed_modes.clone()
    } else {
        form_allowed_modes(form.allowed_mode_novel, form.allowed_mode_if)
    };

    preset.name = form.name;
    preset.role = form.role;
    preset.instructions = form.instructions;
    preset.writing_style = form.writing_style;
    preset.output_format = form.output_format;
    preset.allowed_modes = allowed_modes;

    if let Err(e) = app_state.prompt_preset_service.save_preset(&preset) {
        return preset_save_error_response(e, "Update failed");
    }

    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => {
            return Html(format!("<span class='error'>Load failed: {e}</span>")).into_response();
        }
    };
    let novel_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    let if_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::InteractiveFiction);
    Html(preset_card_html(&preset, &novel_bundle, &if_bundle)).into_response()
}

pub async fn delete_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    if preset.is_default {
        return Html("<span class='error'>Cannot delete default presets</span>".to_string())
            .into_response();
    }

    // Refuse presets referenced as any mode's default — deleting one would
    // leave that mode's bundle pointing at a missing preset.
    {
        let settings = match app_state.settings() {
            Ok(s) => s,
            Err(e) => {
                return Html(format!("<span class='error'>Load failed: {e}</span>"))
                    .into_response();
            }
        };
        if settings.mode_preset_registry.references(&id) {
            return Html(
                "<span class='error'>Preset is a mode default; change the default before deleting</span>"
                    .to_string(),
            )
            .into_response();
        }
    }

    if let Err(e) = app_state.prompt_preset_service.delete_preset(&id) {
        return Html(format!("<span class='error'>Delete failed: {e}</span>")).into_response();
    }

    Html(String::new()).into_response()
}

pub async fn duplicate_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    let mut copy = preset.clone();
    copy.id = generate_preset_id();
    copy.name = match app_state.prompt_preset_service.next_copy_name(&preset) {
        Ok(name) => name,
        Err(e) => return preset_save_error_response(e, "Duplicate failed"),
    };
    copy.is_default = false;

    if let Err(e) = app_state.prompt_preset_service.save_preset(&copy) {
        return preset_save_error_response(e, "Duplicate failed");
    }

    panel_handler(State(app_state)).await.into_response()
}

/// Query for the activate endpoint. The panel's single activate button sends
/// no mode; the handler falls back to Novel.
#[derive(Debug, Default, serde::Deserialize)]
pub struct ActivateQuery {
    pub mode: Option<String>,
}

pub async fn activate_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<ActivateQuery>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    // Absent or invalid mode falls back to Novel (the single panel button);
    // the flags check still refuses a disallowed preset into the Novel slot.
    let mode = NarratorMode::parse_or_default(query.mode.as_deref().unwrap_or("novel"));

    // Refuse before any write so a rejected activation leaves settings
    // untouched.
    if !preset.allows(mode) {
        return Html(format!(
            "<span class='error'>Preset not allowed for {} mode</span>",
            mode.as_str()
        ))
        .into_response();
    }

    let outcome = app_state.settings_service.update_settings(|settings| {
        let mut bundle = settings.mode_preset_registry.bundle_for(mode);
        preset.preset_type.set_bundle_slot(&mut bundle, id);
        settings.mode_preset_registry.set_bundle(bundle);
        Ok(settings.clone())
    });

    let settings = match outcome {
        Ok(s) => s,
        Err(e) => {
            return Html(format!("<span class='error'>Save failed: {e}</span>")).into_response();
        }
    };

    let system_presets = app_state
        .prompt_preset_service
        .list_presets(PresetType::System)
        .unwrap_or_default();
    let quantifier_presets = app_state
        .prompt_preset_service
        .list_presets(PresetType::Quantifier)
        .unwrap_or_default();
    let impersonate_presets = app_state
        .prompt_preset_service
        .list_presets(PresetType::Impersonate)
        .unwrap_or_default();

    let (active_system, active_quantifier, active_impersonate) = mode_active_ids(&settings);
    render_template(PromptPresetsTemplate {
        system_presets,
        quantifier_presets,
        impersonate_presets,
        active_system,
        active_quantifier,
        active_impersonate,
    })
    .into_response()
}
