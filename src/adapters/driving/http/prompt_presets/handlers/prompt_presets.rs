//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset handlers

use axum::{Form, extract::State, response::Html};

use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::NarratorMode;
use crate::domain::model::utils::settings_defaults;
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::presets::{
    preset_card_html, preset_edit_form_html, preset_view_form_html,
};
use crate::adapters::driving::http::utils::handler_helpers::{
    generate_preset_id, parse_preset_type, render_template,
};

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

macro_rules! try_lock {
    ($lock:expr) => {
        $lock.unwrap_or_else(|p| {
            tracing::warn!("Poisoned settings lock recovered in handler");
            p.into_inner()
        })
    };
}

macro_rules! require_preset {
    ($storage:expr, $id:expr) => {
        match $storage.get_preset($id) {
            Ok(Some(p)) => p,
            Ok(None) => return Html("<span class='error'>Preset not found</span>".to_string()),
            Err(e) => return Html(format!("<span class='error'>Load failed: {e}</span>")),
        }
    };
}

pub async fn preset_card_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    let settings = try_lock!(app_state.settings.read());
    let novel_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    let is_active = preset.preset_type.bundle_slot(&novel_bundle) == id;

    Html(preset_card_html(&preset, is_active))
}

pub async fn view_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    Html(preset_view_form_html(&preset))
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

    let settings = try_lock!(app_state.settings.read());
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

#[derive(Debug, Default, serde::Deserialize)]
pub struct PresetForm {
    pub name: String,
    pub role: Option<String>,
    pub instructions: Option<String>,
    pub writing_style: Option<String>,
    pub output_format: Option<String>,
    pub preset_type: String,
}

impl PresetForm {
    fn into_preset(self, id: String, preset_type: PresetType) -> PromptPreset {
        PromptPreset {
            id,
            name: self.name,
            role: self.role,
            instructions: self.instructions,
            writing_style: self.writing_style,
            output_format: self.output_format,
            // Panel-created presets are selectable for every mode.
            allowed_modes: settings_defaults::default_allowed_modes(),
            is_default: false,
            preset_type,
        }
    }
}

pub async fn save_preset_handler(
    State(app_state): State<AppState>,
    Form(form): Form<PresetForm>,
) -> Html<String> {
    let preset_type = match parse_preset_type(&form.preset_type) {
        Some(pt) => pt,
        None => {
            return Html("<span class='error'>Invalid preset type</span>".to_string());
        }
    };

    let preset = form.into_preset(generate_preset_id(), preset_type);

    if let Err(e) = app_state.prompt_preset_service.save_preset(&preset) {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    panel_handler(State(app_state)).await
}

pub async fn edit_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    if preset.is_default {
        return Html("<span class='error'>Cannot edit default presets</span>".to_string());
    }

    let settings = try_lock!(app_state.settings.read());
    let novel_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    let is_active = preset.preset_type.bundle_slot(&novel_bundle) == id;

    Html(preset_edit_form_html(
        &preset,
        preset.preset_type.as_str(),
        is_active,
    ))
}

pub async fn update_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<PresetForm>,
) -> Html<String> {
    let existing = require_preset!(app_state.prompt_preset_service, &id);

    if existing.is_default {
        return Html("<span class='error'>Cannot edit default presets</span>".to_string());
    }

    let preset_type = match parse_preset_type(&form.preset_type) {
        Some(pt) => pt,
        None => {
            return Html("<span class='error'>Invalid preset type</span>".to_string());
        }
    };

    let updated = form.into_preset(id, preset_type);
    // The form carries no mode-flag field; keep the user-owned flags.
    let mut updated = updated;
    updated.allowed_modes = existing.allowed_modes;

    if let Err(e) = app_state.prompt_preset_service.save_preset(&updated) {
        return Html(format!("<span class='error'>Update failed: {e}</span>"));
    }

    let settings = try_lock!(app_state.settings.write());
    let novel_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    let is_active = preset_type.bundle_slot(&novel_bundle) == updated.id;
    Html(preset_card_html(&updated, is_active))
}

pub async fn delete_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    if preset.is_default {
        return Html("<span class='error'>Cannot delete default presets</span>".to_string());
    }

    // Refuse presets referenced as any mode's default — deleting one would
    // leave that mode's bundle pointing at a missing preset.
    {
        let settings = try_lock!(app_state.settings.read());
        if settings.mode_preset_registry.references(&id) {
            return Html(
                "<span class='error'>Preset is a mode default; change the default before deleting</span>"
                    .to_string(),
            );
        }
    }

    if let Err(e) = app_state.prompt_preset_service.delete_preset(&id) {
        return Html(format!("<span class='error'>Delete failed: {e}</span>"));
    }

    Html(String::new())
}

pub async fn duplicate_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    let mut copy = preset.clone();
    copy.id = generate_preset_id();
    copy.name = format!("{} (Copy)", copy.name);
    copy.is_default = false;

    if let Err(e) = app_state.prompt_preset_service.save_preset(&copy) {
        return Html(format!("<span class='error'>Duplicate failed: {e}</span>"));
    }

    panel_handler(State(app_state)).await
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
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_service, &id);

    // Absent or invalid mode falls back to Novel (the single panel button);
    // the flags check still refuses a disallowed preset into the Novel slot.
    let mode = NarratorMode::parse_or_default(query.mode.as_deref().unwrap_or("novel"));

    let mut settings = try_lock!(app_state.settings.write());

    // Refuse before any save or in-memory commit so a rejected activation
    // leaves settings untouched.
    if !preset.allows(mode) {
        return Html(format!(
            "<span class='error'>Preset not allowed for {} mode</span>",
            mode.as_str()
        ));
    }

    let mut candidate = settings.clone();
    let mut bundle = candidate.mode_preset_registry.bundle_for(mode);
    preset.preset_type.set_bundle_slot(&mut bundle, id);
    candidate.mode_preset_registry.set_bundle(bundle);

    if let Err(e) = app_state.settings_service.save_settings(&candidate) {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    *settings = candidate;

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
}
