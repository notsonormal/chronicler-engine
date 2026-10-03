//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! View models decouple templates from domain types.

use std::fmt;

use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::NarratorMode;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::application::ports::text_checker::CheckResult;
use crate::adapters::driving::http::utils::view_models::markdown_to_html;

#[allow(private_interfaces)]
#[derive(Debug, Clone)]
pub struct SafeHtml(String);

impl SafeHtml {
    /// Wrap already-escaped markup so an Askama template renders it verbatim.
    pub(crate) fn new(html: String) -> Self {
        Self(html)
    }
}

impl askama::filters::HtmlSafe for SafeHtml {}

/// One `<option>` in a `<select>`: the value and label a template renders,
/// and whether this is the current selection. `SelectOptionsTemplate` renders
/// it so Askama escapes every value; the constructors here are the single
/// source for which options each select offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

impl SelectOptionView {
    /// The narrator-mode options, marking `selected`.
    pub fn narrator_modes(selected: NarratorMode) -> Vec<Self> {
        [NarratorMode::Novel, NarratorMode::InteractiveFiction]
            .into_iter()
            .map(|mode| Self {
                value: mode.as_str().to_string(),
                label: mode.display_label().to_string(),
                selected: mode == selected,
            })
            .collect()
    }

    /// The LLM-connection provider options, marking the provider whose wire
    /// id is `selected` (an unknown id selects none).
    pub fn providers(selected: &str) -> Vec<Self> {
        [
            ("openrouter", "OpenRouter"),
            ("deepseek", "DeepSeek"),
            ("ollama", "Ollama"),
        ]
        .into_iter()
        .map(|(value, label)| Self {
            value: value.to_string(),
            label: label.to_string(),
            selected: value == selected,
        })
        .collect()
    }

    /// Picker options for one preset slot: presets allowing the game's mode,
    /// plus the stored selection (even when disallowed or absent from the
    /// library) so the browser never silently substitutes another preset.
    pub fn presets(presets: &[PromptPreset], mode: NarratorMode, selected_id: &str) -> Vec<Self> {
        let mut options: Vec<Self> = presets
            .iter()
            .filter(|p| p.allows(mode) || p.id == selected_id)
            .map(|p| Self {
                value: p.id.clone(),
                label: p.name.clone(),
                selected: p.id == selected_id,
            })
            .collect();
        if !options.iter().any(|o| o.value == selected_id) {
            options.insert(
                0,
                Self {
                    value: selected_id.to_string(),
                    label: format!("(missing) {selected_id}"),
                    selected: true,
                },
            );
        }
        options
    }
}

impl fmt::Display for SafeHtml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MessageEntryView {
    pub id: u64,
    pub timestamp: String,
    pub text: SafeHtml,
    pub raw_text: String,
    pub log_type: String,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
    pub swipe_count: usize,
    pub active_swipe_index: usize,
    pub prev_swipe_index: Option<usize>,
    pub next_swipe_index: Option<usize>,
    pub show_retrigger: bool,
}

impl From<&MessageEntry> for MessageEntryView {
    fn from(entry: &MessageEntry) -> Self {
        let parsed_text = markdown_to_html(&entry.text);
        let active = entry.active_swipe_index;
        let count = entry.swipe_count;
        Self {
            id: entry.id,
            timestamp: entry.timestamp.format("%H:%M").to_string(),
            text: SafeHtml(parsed_text),
            raw_text: entry.text.clone(),
            log_type: match entry.message_type {
                MessageType::Narration => "narration".to_string(),
                MessageType::System => "system".to_string(),
                MessageType::Input => "input".to_string(),
            },
            location_header: entry.location_header.clone(),
            event_header: entry.event_header.clone(),
            swipe_count: count,
            active_swipe_index: active,
            prev_swipe_index: if active > 0 { Some(active - 1) } else { None },
            next_swipe_index: if active + 1 < count {
                Some(active + 1)
            } else {
                None
            },
            show_retrigger: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreviewIssueView {
    pub message: String,
    pub kind: String,
}

impl PreviewIssueView {
    pub fn from_check_result(result: &CheckResult) -> Vec<Self> {
        result
            .issues
            .iter()
            .map(|issue| PreviewIssueView {
                message: issue.message.clone(),
                kind: issue.kind.to_string(),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LlmMessageView {
    pub id: i64,
    pub agent_name: String,
    pub backend_name: String,
    pub model_name: String,
    pub timestamp: String,
    pub system_prompt_preview: String,
    pub user_prompt_preview: String,
    pub parsed_response_preview: String,
    pub has_error: bool,
    pub raw_request_json: String,
    pub raw_response_json: String,
}

impl From<&LlmMessage> for LlmMessageView {
    fn from(msg: &LlmMessage) -> Self {
        let pretty_json = |s: &str| -> String {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return trimmed.to_string();
            }
            serde_json::from_str::<serde_json::Value>(trimmed)
                .ok()
                .and_then(|v| serde_json::to_string_pretty(&v).ok())
                .unwrap_or_else(|| trimmed.to_string())
        };
        Self {
            id: msg.id,
            agent_name: msg.agent_name.clone(),
            backend_name: msg.backend_name.clone(),
            model_name: msg.model_name.clone(),
            timestamp: msg.created_at.format("%H:%M:%S").to_string(),
            system_prompt_preview: msg.system_prompt.clone(),
            user_prompt_preview: msg.user_prompt.clone(),
            parsed_response_preview: msg.parsed_response.clone(),
            has_error: msg.error_message.is_some(),
            raw_request_json: pretty_json(&msg.raw_request_json),
            raw_response_json: pretty_json(&msg.raw_response_json),
        }
    }
}

/// View model for the action area template.
#[derive(Debug, Clone)]
pub struct ActionAreaViewModel {
    pub is_disabled: bool,
    pub status_class: String,
    pub status_text: String,
}

impl ActionAreaViewModel {
    pub fn new(status: &GenerationStatus, phase: &GenerationPhase) -> Self {
        let is_disabled = status.is_generating();
        let error_msg = status.error_message().unwrap_or_default().to_string();
        let status_class = if is_disabled {
            "status thinking".to_string()
        } else if status.error_message().is_some() {
            "status error".to_string()
        } else {
            "status ready".to_string()
        };
        let status_text = if is_disabled {
            phase.display_text().to_string()
        } else if !error_msg.is_empty() {
            error_msg.clone()
        } else {
            "Ready".to_string()
        };

        Self {
            is_disabled,
            status_class,
            status_text,
        }
    }
}

/// View model for the options dock — the current pickable option set,
/// rendered above the command input as part of the input surface.
#[derive(Debug, Clone)]
pub struct OptionsDockViewModel {
    pub options: Vec<String>,
    pub is_busy: bool,
}

impl OptionsDockViewModel {
    pub fn new(options: Vec<String>, is_busy: bool) -> Self {
        Self { options, is_busy }
    }
}

/// View model for a single NPC portrait in the visual sidebar.
#[derive(Debug, Clone)]
pub struct NpcPortraitView {
    pub image_path: String,
    pub name: String,
}

/// View model for the visual sidebar template.
#[derive(Debug, Clone)]
pub struct VisualSidebarViewModel {
    pub room_has_image: bool,
    pub room_src: String,
    pub room_alt: String,
    pub npcs: Vec<NpcPortraitView>,
}

impl VisualSidebarViewModel {
    pub fn new(
        room_image_path: Option<String>,
        room_name: String,
        npc_data: Vec<NpcPortraitView>,
    ) -> Self {
        let room_has_image = room_image_path.is_some();
        let room_src = room_image_path.unwrap_or_default();

        Self {
            room_has_image,
            room_src,
            room_alt: room_name,
            npcs: npc_data,
        }
    }
}
