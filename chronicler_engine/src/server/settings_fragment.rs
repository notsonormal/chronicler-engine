//! [DOC: docs/architecture/system.md]

use askama::Template;
use axum::{Form, extract::State, response::Html};

use crate::model::settings::AppSettings;
use crate::narrative::llm::LlmBackendType;
use crate::server::AppState;

/// [DOC: docs/architecture/system.md]
pub fn parse_backend(s: &str) -> LlmBackendType {
    match s {
        "deepseek" => LlmBackendType::DeepSeek,
        "mock" => LlmBackendType::Mock,
        _ => LlmBackendType::OpenRouter,
    }
}

/// [DOC: docs/architecture/system.md]
pub fn parse_api_key(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

#[derive(Template)]
#[template(
    source = r##"
<div class="settings-panel">
    <h2>LLM Settings</h2>
    <form hx-post="/settings" hx-target="#settings-status">
        <div class="form-group">
            <label for="llm_backend">Backend</label>
            <select name="llm_backend" id="llm_backend">
                <option value="openrouter" {% if backend_openrouter %}selected{% endif %}>OpenRouter</option>
                <option value="deepseek" {% if backend_deepseek %}selected{% endif %}>DeepSeek</option>
                <option value="mock" {% if backend_mock %}selected{% endif %}>Mock (Testing)</option>
            </select>
        </div>
        <div class="form-group">
            <label for="llm_model">LLM Model</label>
            <input type="text" id="llm_model" name="llm_model" value="{{ llm_model }}" placeholder="openai/gpt-4o-mini" />
        </div>
        <div class="form-group">
            <label for="quantifier_model">Quantifier Model</label>
            <input type="text" id="quantifier_model" name="quantifier_model" value="{{ quantifier_model }}" placeholder="openai/gpt-4o-mini" />
        </div>
        <div class="form-group">
            <label for="api_key">OpenRouter API Key</label>
            <input type="password" id="api_key" name="api_key" value="" placeholder="{{ api_key_placeholder }}" />
        </div>
        <button type="submit">Save Settings</button>
        <span id="settings-status"></span>
    </form>
</div>
"##,
    ext = "html"
)]
pub struct SettingsTemplate {
    pub backend_openrouter: bool,
    pub backend_deepseek: bool,
    pub backend_mock: bool,
    pub llm_model: String,
    pub quantifier_model: String,
    pub api_key_placeholder: String,
}

impl SettingsTemplate {
    fn from_settings(settings: &AppSettings) -> Self {
        Self {
            backend_openrouter: settings.llm_backend == LlmBackendType::OpenRouter,
            backend_deepseek: settings.llm_backend == LlmBackendType::DeepSeek,
            backend_mock: settings.llm_backend == LlmBackendType::Mock,
            llm_model: settings.llm_model.clone(),
            quantifier_model: settings.quantifier_model.clone(),
            api_key_placeholder: if settings.openrouter_api_key.is_some() {
                "(current key set)".to_string()
            } else {
                "(not set - use env var)".to_string()
            },
        }
    }
}

pub async fn settings_panel(State(app_state): State<AppState>) -> Html<String> {
    let settings = app_state.settings.read().unwrap();
    let template = SettingsTemplate::from_settings(&settings);
    Html(template.render().unwrap())
}

#[derive(Debug, serde::Deserialize)]
pub struct SettingsForm {
    pub llm_backend: String,
    pub llm_model: String,
    pub quantifier_model: String,
    pub api_key: String,
}

pub async fn save_settings_handler(
    State(app_state): State<AppState>,
    Form(form): Form<SettingsForm>,
) -> Html<String> {
    let backend = match form.llm_backend.as_str() {
        "deepseek" => LlmBackendType::DeepSeek,
        "mock" => LlmBackendType::Mock,
        _ => LlmBackendType::OpenRouter,
    };

    let api_key = if form.api_key.is_empty() {
        None
    } else {
        Some(form.api_key)
    };

    let new_settings = AppSettings {
        llm_backend: backend,
        llm_model: form.llm_model,
        quantifier_model: form.quantifier_model,
        openrouter_api_key: api_key,
    };

    if let Err(e) = new_settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    let mut settings = app_state.settings.write().unwrap();
    *settings = new_settings;

    Html("Settings saved!".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::settings::AppSettings;

    mod parse_backend {
        use super::*;

        #[test]
        fn test_deepseek_returns_deepseek() {
            assert_eq!(parse_backend("deepseek"), LlmBackendType::DeepSeek);
        }

        #[test]
        fn test_mock_returns_mock() {
            assert_eq!(parse_backend("mock"), LlmBackendType::Mock);
        }

        #[test]
        fn test_openrouter_returns_openrouter() {
            assert_eq!(parse_backend("openrouter"), LlmBackendType::OpenRouter);
        }

        #[test]
        fn test_unknown_returns_openrouter_default() {
            assert_eq!(parse_backend("unknown_backend"), LlmBackendType::OpenRouter);
            assert_eq!(parse_backend(""), LlmBackendType::OpenRouter);
            assert_eq!(parse_backend("ollama"), LlmBackendType::OpenRouter);
        }
    }

    mod parse_api_key {
        use super::*;

        #[test]
        fn test_empty_returns_none() {
            assert_eq!(parse_api_key(""), None);
        }

        #[test]
        fn test_non_empty_returns_some() {
            assert_eq!(parse_api_key("sk-test123"), Some("sk-test123".to_string()));
            assert_eq!(parse_api_key("   "), Some("   ".to_string()));
        }
    }

    mod settings_template_from_settings {
        use super::*;

        fn make_settings(backend: LlmBackendType, api_key: Option<String>) -> AppSettings {
            AppSettings {
                llm_backend: backend,
                llm_model: "test-model".to_string(),
                quantifier_model: "test-quantifier".to_string(),
                openrouter_api_key: api_key,
            }
        }

        #[test]
        fn test_openrouter_backend_sets_openrouter_flag() {
            let settings = make_settings(LlmBackendType::OpenRouter, None);
            let template = SettingsTemplate::from_settings(&settings);

            assert!(template.backend_openrouter);
            assert!(!template.backend_deepseek);
            assert!(!template.backend_mock);
        }

        #[test]
        fn test_deepseek_backend_sets_deepseek_flag() {
            let settings = make_settings(LlmBackendType::DeepSeek, None);
            let template = SettingsTemplate::from_settings(&settings);

            assert!(!template.backend_openrouter);
            assert!(template.backend_deepseek);
            assert!(!template.backend_mock);
        }

        #[test]
        fn test_mock_backend_sets_mock_flag() {
            let settings = make_settings(LlmBackendType::Mock, None);
            let template = SettingsTemplate::from_settings(&settings);

            assert!(!template.backend_openrouter);
            assert!(!template.backend_deepseek);
            assert!(template.backend_mock);
        }

        #[test]
        fn test_api_key_set_shows_current_key_placeholder() {
            let settings = make_settings(LlmBackendType::OpenRouter, Some("sk-abc123".to_string()));
            let template = SettingsTemplate::from_settings(&settings);

            assert_eq!(template.api_key_placeholder, "(current key set)");
        }

        #[test]
        fn test_api_key_none_shows_env_var_placeholder() {
            let settings = make_settings(LlmBackendType::OpenRouter, None);
            let template = SettingsTemplate::from_settings(&settings);

            assert_eq!(template.api_key_placeholder, "(not set - use env var)");
        }

        #[test]
        fn test_models_are_copied() {
            let settings = AppSettings {
                llm_backend: LlmBackendType::OpenRouter,
                llm_model: "custom-model".to_string(),
                quantifier_model: "custom-quantifier".to_string(),
                openrouter_api_key: None,
            };

            let template = SettingsTemplate::from_settings(&settings);

            assert_eq!(template.llm_model, "custom-model");
            assert_eq!(template.quantifier_model, "custom-quantifier");
        }
    }
}
