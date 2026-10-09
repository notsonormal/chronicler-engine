//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset handlers

use axum::body::Body;
use axum::{Form, extract::State, response::Html, response::IntoResponse, response::Response};

use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{AppSettings, NarratorMode};
use crate::domain::model::utils::settings_defaults;
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::presets::{
    preset_card_html, preset_edit_form_html, preset_view_form_html,
};
use crate::adapters::driving::http::utils::error::{
    action_error_response, action_failure_response, action_refusal_response,
    error_fragment_response,
};
use crate::adapters::driving::http::utils::handler_helpers::{generate_storage_id, render_template};

use crate::adapters::driving::http::prompt_presets::templates::prompt_presets::PromptPresetsTemplate;

/// The one response a single-card endpoint returns, so an edit-refreshed card is
/// byte-identical to the same card in the panel.
fn card_response(preset: &PromptPreset, settings: &AppSettings) -> Response<Body> {
    Html(preset_card_html(preset, settings)).into_response()
}

fn presets_template(app_state: &AppState, settings: &AppSettings) -> PromptPresetsTemplate {
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
    let options_presets = app_state
        .prompt_preset_service
        .list_presets(PresetType::Options)
        .unwrap_or_default();

    PromptPresetsTemplate {
        system_presets,
        quantifier_presets,
        impersonate_presets,
        options_presets,
        settings: settings.clone(),
    }
}

/// The two load macros differ only in their status: a 200 body lands inside
/// the region a GET fragment loads, while a non-2xx leaves a POST handler's
/// region in place for the client's inline error slot.
macro_rules! require_preset {
    ($storage:expr, $id:expr) => {
        match $storage.get_preset($id) {
            Ok(Some(p)) => p,
            Ok(None) => return error_fragment_response("Preset not found"),
            Err(e) => return error_fragment_response(format!("Load failed: {e}")),
        }
    };
}

macro_rules! require_preset_action {
    ($storage:expr, $id:expr) => {
        match $storage.get_preset($id) {
            Ok(Some(p)) => p,
            Ok(None) => return action_refusal_response("Preset not found"),
            Err(e) => return action_failure_response(format!("Load failed: {e}")),
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
        Err(e) => return error_fragment_response(format!("Load failed: {e}")),
    };
    card_response(&preset, &settings)
}

pub async fn view_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    Html(preset_view_form_html(&preset)).into_response()
}

pub async fn panel_handler(State(app_state): State<AppState>) -> Response<Body> {
    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => return error_fragment_response(format!("Load failed: {e}")),
    };

    render_template(presets_template(&app_state, &settings)).into_response()
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
    let preset_type = match PresetType::try_from(form.preset_type.as_str()).ok() {
        Some(pt) => pt,
        None => return action_refusal_response("Invalid preset type"),
    };

    // Create forms render no checkboxes; both-false means "flags not
    // offered" → default to both modes.
    let allowed_modes = if form.allowed_mode_novel || form.allowed_mode_if {
        form_allowed_modes(form.allowed_mode_novel, form.allowed_mode_if)
    } else {
        settings_defaults::default_allowed_modes()
    };

    let preset = form.into_preset(generate_storage_id("preset"), preset_type, allowed_modes);

    if let Err(e) = app_state.prompt_preset_service.save_preset(&preset) {
        return action_error_response(e, "Save failed");
    }

    panel_handler(State(app_state)).await.into_response()
}

pub async fn edit_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    if preset.is_default {
        return error_fragment_response("Cannot edit default presets");
    }

    Html(preset_edit_form_html(&preset, preset.preset_type.as_str())).into_response()
}

pub async fn update_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<PresetForm>,
) -> Response<Body> {
    let mut preset = match app_state.prompt_preset_service.get_preset(&id) {
        Ok(Some(preset)) => preset,
        Ok(None) => return action_refusal_response("Preset not found"),
        Err(e) => return action_failure_response(format!("Load failed: {e}")),
    };

    if preset.is_default {
        return action_refusal_response("Cannot edit default presets");
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
        return action_error_response(e, "Update failed");
    }

    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => return action_failure_response(format!("Load failed: {e}")),
    };
    card_response(&preset, &settings)
}

pub async fn delete_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = match app_state.prompt_preset_service.get_preset(&id) {
        Ok(Some(preset)) => preset,
        Ok(None) => return action_refusal_response("Preset not found"),
        Err(e) => return action_failure_response(format!("Load failed: {e}")),
    };

    if preset.is_default {
        return action_refusal_response("Cannot delete default presets");
    }

    // Refuse presets referenced as any mode's default — deleting one would
    // leave that mode's bundle pointing at a missing preset.
    {
        let settings = match app_state.settings() {
            Ok(s) => s,
            Err(e) => return action_failure_response(format!("Load failed: {e}")),
        };
        if settings.mode_preset_registry.references(&id) {
            return action_refusal_response(
                "Preset is a mode default; change the default before deleting",
            );
        }
        if settings.active_preset_id(PresetType::Options, NarratorMode::Novel) == id {
            return action_refusal_response(
                "Preset is the default Options preset; change the default before deleting",
            );
        }
    }

    if let Err(e) = app_state.prompt_preset_service.delete_preset(&id) {
        return action_failure_response(format!("Delete failed: {e}"));
    }

    Html(String::new()).into_response()
}

pub async fn duplicate_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset_action!(app_state.prompt_preset_service, &id);

    let mut copy = preset.clone();
    copy.id = generate_storage_id("preset");
    copy.name = match app_state.prompt_preset_service.next_copy_name(&preset) {
        Ok(name) => name,
        Err(e) => return action_error_response(e, "Duplicate failed"),
    };
    copy.is_default = false;

    if let Err(e) = app_state.prompt_preset_service.save_preset(&copy) {
        return action_error_response(e, "Duplicate failed");
    }

    panel_handler(State(app_state)).await.into_response()
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct ActivateQuery {
    pub mode: Option<String>,
}

pub async fn activate_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<ActivateQuery>,
) -> Response<Body> {
    let preset = require_preset_action!(app_state.prompt_preset_service, &id);

    // Absent or invalid mode falls back to Novel (the single panel button).
    // An Options preset is mode-agnostic, so its flags do not gate activation.
    let mode = NarratorMode::parse_or_default(query.mode.as_deref().unwrap_or("novel"));

    // Refuse before any write so a rejected activation leaves settings
    // untouched.
    if preset.preset_type.is_mode_tagged() && !preset.allows(mode) {
        return action_refusal_response(format!("Preset not allowed for {} mode", mode.as_str()));
    }

    let outcome = app_state.settings_service.update_settings(|settings| {
        settings.set_active_preset(preset.preset_type, mode, id.clone());
        Ok(settings.clone())
    });

    let settings = match outcome {
        Ok(s) => s,
        Err(e) => return action_failure_response(format!("Save failed: {e}")),
    };

    render_template(presets_template(&app_state, &settings)).into_response()
}
