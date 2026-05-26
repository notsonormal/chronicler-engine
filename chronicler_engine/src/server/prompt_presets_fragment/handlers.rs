use axum::{Form, extract::State, response::Html};

use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::server::AppState;

use super::fragments::{preset_card_html, preset_edit_form_html, preset_view_form_html};
use super::template::PromptPresetsTemplate;

macro_rules! try_lock {
    ($lock:expr) => {
        $lock.unwrap_or_else(|p| {
            log::warn!("Poisoned settings lock recovered in handler");
            p.into_inner()
        })
    };
}

macro_rules! require_preset {
    ($storage:expr, $id:expr) => {
        match $storage.get($id) {
            Ok(Some(p)) => p,
            Ok(None) => return Html("<span class='error'>Preset not found</span>".to_string()),
            Err(e) => return Html(format!("<span class='error'>Load failed: {e}</span>")),
        }
    };
}

fn render_template<T: askama::Template>(template: T) -> Html<String> {
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<span class='error'>Template error: {e}</span>")),
    }
}

fn parse_preset_type(value: &str) -> Option<PresetType> {
    PresetType::try_from(value).ok()
}

fn generate_preset_id() -> String {
    format!(
        "preset-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
}

/// [DOC: docs/architecture/system.md]
pub async fn preset_card_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_storage, &id);

    let settings = try_lock!(app_state.settings.read());
    let is_active = match preset.preset_type {
        PresetType::System => settings.active_system_prompt_preset_id == id,
        PresetType::Quantifier => settings.active_quantifier_prompt_preset_id == id,
    };

    Html(preset_card_html(&preset, is_active))
}

/// [DOC: docs/architecture/system.md]
pub async fn view_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_storage, &id);

    Html(preset_view_form_html(&preset))
}

/// [DOC: docs/architecture/system.md]
pub async fn panel_handler(State(app_state): State<AppState>) -> Html<String> {
    let system_presets = app_state
        .prompt_preset_storage
        .list(PresetType::System)
        .unwrap_or_default();
    let quantifier_presets = app_state
        .prompt_preset_storage
        .list(PresetType::Quantifier)
        .unwrap_or_default();

    let settings = try_lock!(app_state.settings.read());

    render_template(PromptPresetsTemplate {
        system_presets,
        quantifier_presets,
        active_system_id: settings.active_system_prompt_preset_id.clone(),
        active_quantifier_id: settings.active_quantifier_prompt_preset_id.clone(),
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
            is_default: false,
            preset_type,
        }
    }
}

/// [DOC: docs/architecture/system.md]
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

    if let Err(e) = app_state.prompt_preset_storage.save(&preset) {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    panel_handler(State(app_state)).await
}

/// [DOC: docs/architecture/system.md]
pub async fn edit_preset_form_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_storage, &id);

    if preset.is_default {
        return Html("<span class='error'>Cannot edit default presets</span>".to_string());
    }

    let settings = try_lock!(app_state.settings.read());
    let is_active = match preset.preset_type {
        PresetType::System => settings.active_system_prompt_preset_id == id,
        PresetType::Quantifier => settings.active_quantifier_prompt_preset_id == id,
    };

    Html(preset_edit_form_html(
        &preset,
        preset.preset_type.as_str(),
        is_active,
    ))
}

/// [DOC: docs/architecture/system.md]
pub async fn update_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<PresetForm>,
) -> Html<String> {
    let existing = require_preset!(app_state.prompt_preset_storage, &id);

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

    if let Err(e) = app_state.prompt_preset_storage.save(&updated) {
        return Html(format!("<span class='error'>Update failed: {e}</span>"));
    }

    let settings = try_lock!(app_state.settings.write());
    let is_active = match preset_type {
        PresetType::System => settings.active_system_prompt_preset_id == updated.id,
        PresetType::Quantifier => settings.active_quantifier_prompt_preset_id == updated.id,
    };
    Html(preset_card_html(&updated, is_active))
}

/// [DOC: docs/architecture/system.md]
pub async fn delete_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_storage, &id);

    if preset.is_default {
        return Html("<span class='error'>Cannot delete default presets</span>".to_string());
    }

    if let Err(e) = app_state.prompt_preset_storage.delete(&id) {
        return Html(format!("<span class='error'>Delete failed: {e}</span>"));
    }

    Html(String::new())
}

/// [DOC: docs/architecture/system.md]
pub async fn duplicate_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_storage, &id);

    let mut copy = preset.clone();
    copy.id = generate_preset_id();
    copy.name = format!("{} (Copy)", copy.name);
    copy.is_default = false;

    if let Err(e) = app_state.prompt_preset_storage.save(&copy) {
        return Html(format!("<span class='error'>Duplicate failed: {e}</span>"));
    }

    panel_handler(State(app_state)).await
}

/// [DOC: docs/architecture/system.md]
pub async fn activate_preset_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let preset = require_preset!(app_state.prompt_preset_storage, &id);

    let mut settings = try_lock!(app_state.settings.write());

    match preset.preset_type {
        PresetType::System => {
            settings.active_system_prompt_preset_id = id.clone();
        }
        PresetType::Quantifier => {
            settings.active_quantifier_prompt_preset_id = id.clone();
        }
    }
    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    let system_presets = app_state
        .prompt_preset_storage
        .list(PresetType::System)
        .unwrap_or_default();
    let quantifier_presets = app_state
        .prompt_preset_storage
        .list(PresetType::Quantifier)
        .unwrap_or_default();

    render_template(PromptPresetsTemplate {
        system_presets,
        quantifier_presets,
        active_system_id: settings.active_system_prompt_preset_id.clone(),
        active_quantifier_id: settings.active_quantifier_prompt_preset_id.clone(),
    })
}
