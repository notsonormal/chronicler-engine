//! [DOC: docs/system/dashboard.md]
//! View models decouple templates from domain types.

use std::fmt;

use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::adapters::driven::text_check::CheckResult;

#[allow(private_interfaces)]
#[derive(Debug, Clone)]
pub struct SafeHtml(String);

impl askama::filters::HtmlSafe for SafeHtml {}

impl fmt::Display for SafeHtml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) fn markdown_to_html(text: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let escaped = text.replace('&', "&amp;").replace('<', "&lt;");

    let parser = Parser::new_ext(&escaped, Options::ENABLE_SMART_PUNCTUATION);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
        .replace('\u{201C}', "<q>")
        .replace('\u{201D}', "</q>")
}

#[derive(Debug, Clone)]
pub struct MessageEntryView {
    pub id: u64,
    pub timestamp: String,
    pub sender: String,
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
            sender: entry.sender.clone().unwrap_or_default(),
            text: SafeHtml(parsed_text),
            raw_text: entry.text.clone(),
            log_type: match entry.message_type {
                MessageType::Narration => "narration".to_string(),
                MessageType::Dialogue => "dialogue".to_string(),
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
    pub error_message: String,
    pub status_class: String,
    pub status_text: String,
    pub available_actions: Vec<String>,
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
            error_message: error_msg,
            status_class,
            status_text,
            available_actions: vec![],
        }
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
