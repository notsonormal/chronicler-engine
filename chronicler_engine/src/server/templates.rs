//! [DOC: docs/architecture/system.md]

use std::fmt;

use askama::Template;
use pulldown_cmark::{Options, Parser, html};

use crate::model::state::{LogEntry, LogType};

/// [DOC: docs/architecture/system.md]
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
    // [DOC: docs/architecture/system.md]
    let escaped = text.replace('&', "&amp;").replace('<', "&lt;");

    let parser = Parser::new_ext(&escaped, Options::ENABLE_SMART_PUNCTUATION);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
        .replace('\u{201C}', "<q>")
        .replace('\u{201D}', "</q>")
}

#[derive(Template)]
#[template(
    source = r#"<div class="header"><span class="game-title">Chronicler Engine</span><span class="connection-status connected" id="connection-status">Connected</span></div>"#,
    ext = "html"
)]
pub struct HeaderTemplate {
    pub room_name: String,
}

#[derive(Debug, Clone)]
pub struct LogEntryView {
    pub id: u64,
    pub timestamp: String,
    pub sender: String,
    pub text: SafeHtml,
    pub raw_text: String,
    pub log_type: String,
    pub is_location: bool,
    pub is_event: bool,
}

impl From<&LogEntry> for LogEntryView {
    fn from(entry: &LogEntry) -> Self {
        let parsed_text = markdown_to_html(&entry.text);
        let is_event = entry.log_type == LogType::Event;
        let is_location = entry.sender.is_some() && entry.text.is_empty() && !is_event;
        Self {
            id: entry.id,
            timestamp: entry.timestamp.format("%H:%M").to_string(),
            sender: entry.sender.clone().unwrap_or_default(),
            text: SafeHtml(parsed_text),
            raw_text: entry.text.clone(),
            log_type: match entry.log_type {
                LogType::Narration => "narration".to_string(),
                LogType::Dialogue => "dialogue".to_string(),
                LogType::System => "system".to_string(),
                LogType::Input => "input".to_string(),
                LogType::Event => "event".to_string(),
            },
            is_location,
            is_event,
        }
    }
}

#[derive(Template)]
#[template(
    source = r#"<div class="story-log" id="story-log">{% for entry in entries %}<div class="log-entry {{ entry.log_type }}" data-id="{{ entry.id }}" data-raw-text="{{ entry.raw_text | escape }}">{% if entry.is_location %}<span class="location-header">{{ entry.sender }}</span><span class="location-timestamp">- {{ entry.timestamp }}</span>{% elif entry.is_event %}<span class="event-header">{{ entry.sender }}</span><span class="event-timestamp">- {{ entry.timestamp }}</span>{% else %}<span class="timestamp">{{ entry.timestamp }}</span>{% if entry.sender != "" %}<span class="sender">{{ entry.sender }}:</span> {% endif %}{% endif %}<span class="text">{{ entry.text }}</span>{% if !entry.is_location && !entry.is_event %}<button class="edit-btn" onclick="showEditForm({{ entry.id }})" title="Edit">&#9998;</button>{% if loop.last %}{% if entry.log_type == "narration" || entry.log_type == "dialogue" %}<button class="retry-btn" onclick="submitRetry()" title="Retry">&#8635;</button>{% endif %}{% endif %}{% endif %}</div>{% endfor %}</div>"#,
    ext = "html"
)]
pub struct StoryLogTemplate {
    pub entries: Vec<LogEntryView>,
}

impl StoryLogTemplate {
    pub fn new(entries: &[LogEntry]) -> Self {
        Self {
            entries: entries.iter().map(LogEntryView::from).collect(),
        }
    }
}

#[derive(Template)]
#[template(
    source = r#"<div id="visual-sidebar" class="location-header-bar">{% if room_has_image %}<div class="image-container location-image"><img src="{{ room_src }}" alt="{{ room_alt }}" /></div>{% else %}<div class="image-container no-image"><div class="placeholder">No Location Image</div></div>{% endif %}</div><div class="npc-portrait-divider"></div><div class="npc-portraits">{% for npc in npcs %}<div class="image-container npc-portrait"><img src="{{ npc.0 }}" alt="{{ npc.1 }}" /></div>{% endfor %}</div>"#,
    ext = "html"
)]
pub struct VisualSidebarTemplate {
    pub room_has_image: bool,
    pub room_src: String,
    pub room_alt: String,
    pub npcs: Vec<(String, String)>,
}

impl VisualSidebarTemplate {
    pub fn new(
        room_image_path: Option<String>,
        room_name: String,
        npc_data: Vec<(String, String)>,
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

#[derive(Template)]
#[template(
    source = r#"{% for npc in npcs %}<div class="headshot" onclick="toggleVisualSidebar()"><img src="{{ npc.0 }}" alt="{{ npc.1 }}" /><div class="name">{{ npc.1 }}</div></div>{% endfor %}"#,
    ext = "html"
)]
pub struct CharacterHeadshotsTemplate {
    pub npcs: Vec<(String, String)>,
}

impl CharacterHeadshotsTemplate {
    pub fn new(npc_data: Vec<(String, String)>) -> Self {
        Self { npcs: npc_data }
    }
}

#[derive(Template)]
#[template(
    source = r#"<div cmd-area id=cmd-area><form method=post hx-post=/action hx-target=#cmd-area hx-swap=outerHTML class=command-wrapper><input type=text name=command placeholder="Enter command..." value="" {% if is_disabled %}disabled{% endif %} autocomplete=off /><button type=submit {% if is_disabled %}disabled{% endif %}>Send</button></form><div class=command-hints>{% for action in available_actions %}<span class=action-hint>{{ action }}</span>{% endfor %}</div><div id="error-message" class="error-message">{{ error_message }}</div><div class="{{ status_class }}" id="status-display">{{ status_text }}</div></div>"#,
    ext = "html"
)]
pub struct ActionAreaTemplate {
    pub is_disabled: bool,
    pub error_message: String,
    pub status_class: String,
    pub status_text: String,
    pub available_actions: Vec<String>,
}

impl ActionAreaTemplate {
    pub fn new(
        status: &crate::model::state::GenerationStatus,
        phase: &crate::model::state::GenerationPhase,
        exits: &[String],
    ) -> Self {
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

        let mut available_actions = vec!["Look".to_string(), "Inventory".to_string()];
        available_actions.extend(exits.iter().cloned());

        Self {
            is_disabled,
            error_message: error_msg,
            status_class,
            status_text,
            available_actions,
        }
    }
}
