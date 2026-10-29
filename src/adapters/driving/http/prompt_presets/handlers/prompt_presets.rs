//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset handlers

use askama::Template;
use axum::body::Body;
use axum::{Form, extract::State, response::Html, response::IntoResponse, response::Response};

use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{AppSettings, NarratorMode};
use crate::domain::model::utils::settings_defaults;
use crate::error::EngineError;
use crate::application::prompt_preset_service::{PresetEdit, PromptPresetService};
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::presets::{preset_edit_form_html, preset_view_form_html};
use crate::adapters::driving::http::utils::answer_words::{item_named, naming_item};
use crate::adapters::driving::http::utils::error::{
    AnswerPolicy, action_failure_response, action_not_found_response, action_invalid_response,
    answer_for_error,
};
use crate::adapters::driving::http::utils::fragment::{render_fragment, render_response};
use crate::adapters::driving::http::utils::handler_helpers::generate_storage_id;
use crate::adapters::driving::http::utils::response::ok;

use crate::adapters::driving::http::prompt_presets::templates::prompt_presets::{
    PresetAddFormTemplate, PromptPresetsTemplate,
};

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

/// The dialog forms open only on a 2xx answer: a missing preset answers a 404 and a
/// failed read a 500, and the control that asked reports either in its own slot.
macro_rules! require_preset {
    ($storage:expr, $id:expr) => {
        match $storage.get_preset($id) {
            Ok(Some(p)) => p,
            Ok(None) => return action_not_found_response("Preset not found"),
            Err(e) => return action_failure_response(format!("Load failed: {e}")),
        }
    };
}

macro_rules! require_preset_action {
    ($storage:expr, $id:expr) => {
        match $storage.get_preset($id) {
            Ok(Some(p)) => p,
            Ok(None) => return action_invalid_response("Preset not found"),
            Err(e) => return action_failure_response(format!("Load failed: {e}")),
        }
    };
}

pub async fn view_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    Html(preset_view_form_html(&preset)).into_response()
}

#[derive(Debug, serde::Deserialize)]
pub struct NewPresetQuery {
    #[serde(rename = "type")]
    pub preset_type: String,
}

pub async fn new_preset_form_handler(
    axum::extract::Query(query): axum::extract::Query<NewPresetQuery>,
) -> Response<Body> {
    match PresetType::try_from(query.preset_type.as_str()) {
        Ok(preset_type) => render_response(&PresetAddFormTemplate::for_type(preset_type)),
        Err(_) => action_invalid_response("Invalid preset type"),
    }
}

pub async fn panel_handler(State(app_state): State<AppState>) -> Response<Body> {
    render_fragment(
        &app_state,
        render_prompt_presets_panel,
        "prompt_presets_panel",
    )
}

pub fn render_prompt_presets_panel(app_state: &AppState) -> Result<String, EngineError> {
    let settings = app_state.settings()?;
    presets_template(app_state, &settings)
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
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
        None => return action_invalid_response("Invalid preset type"),
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
        return naming_item(
            answer_for_error(e, AnswerPolicy::EngineText),
            item_named(&preset.name),
        );
    }

    ok("")
}

pub async fn edit_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);
    if let Err(e) = PromptPresetService::ensure_editable(&preset) {
        return answer_for_error(e, AnswerPolicy::EngineText);
    }
    Html(preset_edit_form_html(&preset, preset.preset_type.as_str())).into_response()
}

pub async fn update_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<PresetForm>,
) -> Response<Body> {
    // preset_type is fixed at creation, so the form's hidden input is
    // ignored. Both-false = flags absent — urlencoded checkboxes cannot
    // distinguish "unchecked all" from "field omitted", so preserve.
    let allowed_modes = (form.allowed_mode_novel || form.allowed_mode_if)
        .then(|| form_allowed_modes(form.allowed_mode_novel, form.allowed_mode_if));
    let name = item_named(&form.name);
    let edit = PresetEdit {
        name: form.name,
        role: form.role,
        instructions: form.instructions,
        writing_style: form.writing_style,
        output_format: form.output_format,
        allowed_modes,
    };

    match app_state.prompt_preset_service.edit_preset(&id, edit) {
        Ok(_) => ok(""),
        Err(e) => naming_item(answer_for_error(e, AnswerPolicy::EngineText), name),
    }
}

pub async fn delete_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    if let Err(e) = app_state.prompt_preset_service.delete_preset(&id) {
        let name = app_state
            .prompt_preset_service
            .get_preset(&id)
            .ok()
            .flatten()
            .and_then(|preset| item_named(&preset.name));
        return naming_item(answer_for_error(e, AnswerPolicy::EngineText), name);
    }

    ok("")
}

pub async fn duplicate_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let preset = require_preset_action!(app_state.prompt_preset_service, &id);

    let name = item_named(&preset.name);
    let mut copy = preset.clone();
    copy.id = generate_storage_id("preset");
    copy.name = match app_state.prompt_preset_service.next_copy_name(&preset) {
        Ok(copy_name) => copy_name,
        Err(e) => return naming_item(answer_for_error(e, AnswerPolicy::EngineText), name),
    };
    copy.is_default = false;

    if let Err(e) = app_state.prompt_preset_service.save_preset(&copy) {
        return naming_item(answer_for_error(e, AnswerPolicy::EngineText), name);
    }

    ok("")
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
    let name = item_named(&preset.name);

    // Refuse before any write so a rejected activation leaves settings
    // untouched.
    if preset.preset_type.is_mode_tagged() && !preset.allows(mode) {
        return naming_item(
            action_invalid_response(format!("Preset not allowed for {} mode", mode.as_str())),
            name,
        );
    }

    let outcome = app_state.settings_service.update_settings(|settings| {
        settings.set_active_preset(preset.preset_type, mode, id.clone());
        Ok(settings.clone())
    });

    match outcome {
        Ok(_) => ok(""),
        Err(e) => naming_item(action_failure_response(format!("Save failed: {e}")), name),
    }
}
