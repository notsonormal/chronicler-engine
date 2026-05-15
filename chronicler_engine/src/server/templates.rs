//! [DOC: docs/architecture/system.md]

use std::fmt;

use askama::Template;
use pulldown_cmark::{Options, Parser, html};

use crate::model::llm_message::LlmMessage;
use crate::model::state::{LogEntry, LogType};
use crate::narrative::text_check::CheckResult;

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
    source = r#"<div class="header"><span class="game-title">Chronicler Engine</span><button class="reset-btn" hx-post="/reset" hx-confirm="Are you sure you want to reset the game? All progress will be lost." hx-swap="none">Reset Game</button><span class="connection-status connected" id="connection-status">Connected</span></div>"#,
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
    pub location_header: Option<String>,
    pub event_header: Option<String>,
}

impl From<&LogEntry> for LogEntryView {
    fn from(entry: &LogEntry) -> Self {
        let parsed_text = markdown_to_html(&entry.text);
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
            },
            location_header: entry.location_header.clone(),
            event_header: entry.event_header.clone(),
        }
    }
}

#[derive(Template)]
#[template(
    source = r#"<div class="story-log" id="story-log">{% for entry in entries %}<div class="log-entry {{ entry.log_type }}{% if entry.location_header.is_some() %} location{% endif %}" data-id="{{ entry.id }}" data-raw-text="{{ entry.raw_text | escape }}"><div class="message-header"><div class="message-info">{% if entry.location_header.is_some() %}<span class="location-header">{{ entry.location_header.as_ref().unwrap() }}</span><span class="location-timestamp">- {{ entry.timestamp }}</span>{% elif entry.event_header.is_some() %}<span class="event-header">{{ entry.event_header.as_ref().unwrap() }}</span><span class="event-timestamp">- {{ entry.timestamp }}</span>{% else %}<span class="timestamp">{{ entry.timestamp }}</span>{% if entry.sender != "" %}<span class="sender">{{ entry.sender }}:</span>{% endif %}{% endif %}</div><div class="message-actions"><button class="action-btn edit-btn" onclick="showEditForm({{ entry.id }})" title="Edit">&#9998;</button>{% if loop.last && entries.len() > 1 %}<button class="action-btn delete-btn" onclick="deleteMessage()" title="Delete">&#128465;</button>{% endif %}{% if entry.log_type == "input" %}<button class="action-btn check-btn" onclick="checkLogText(this.closest('.log-entry').dataset.rawText)" title="Check spelling & grammar">&#x2713;</button>{% endif %}{% if loop.last && entries.len() > 1 %}{% if entry.log_type == "narration" || entry.log_type == "dialogue" %}<button class="action-btn retry-btn" onclick="submitRetry()" title="Retry">&#8635;</button>{% endif %}{% endif %}</div></div><span class="text">{{ entry.text }}</span></div>{% endfor %}</div>"#,
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
    source = r##"<div class="action-area" id="action-area"><form id="command-form" hx-post="/action/check" hx-target="#action-area" hx-swap="innerHTML" hx-sync="this:drop" hx-on::before-request="saveActionArea()" hx-on::after-request="onActionFormAfterRequest()"><input type="text" name="command" placeholder="Enter command..." required minlength="1" autocomplete="off" {% if is_disabled %}disabled{% endif %} /><button type="submit" id="submit-btn" {% if is_disabled %}disabled{% endif %}><span class="btn-icon">&#9654;</span> Send</button></form><div class="action-hints" id="action-hints" hx-get="/hints" hx-trigger="load, every 5s"></div><div class="{{ status_class }}" id="status-display" hx-get="/status/generating" hx-trigger="load, every 5s" hx-swap="innerHTML" hx-on::after-swap="onStatusPoll(this)"><span class="{{ status_class }}">{{ status_text }}</span></div>{% if show_swipe_nav %}<div class="swipe-nav"><button class="swipe-btn" hx-post="/turn/{{ turn_id }}/swipe/{{ prev_swipe }}" hx-swap="none" {% if !has_prev %}disabled{% endif %}>&#9664;</button><span class="swipe-counter">{{ current_swipe + 1 }} / {{ total_swipes }}</span><button class="swipe-btn" hx-post="/turn/{{ turn_id }}/swipe/{{ next_swipe }}" hx-swap="none" {% if !has_next %}disabled{% endif %}>&#9654;</button></div>{% endif %}<button class="checkpoint-btn" hx-post="/checkpoint" hx-swap="none">&#128204; Bookmark</button></div>"##,
    ext = "html"
)]
pub struct ActionAreaTemplate {
    pub is_disabled: bool,
    pub error_message: String,
    pub status_class: String,
    pub status_text: String,
    pub available_actions: Vec<String>,
    pub show_swipe_nav: bool,
    pub current_swipe: u32,
    pub total_swipes: u32,
    pub turn_id: String,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_swipe: u32,
    pub next_swipe: u32,
}

impl ActionAreaTemplate {
    pub fn new(
        status: &crate::model::state::GenerationStatus,
        phase: &crate::model::state::GenerationPhase,
        exits: &[String],
        turn_data: Option<(String, u32, u32)>,
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

        let (
            show_swipe_nav,
            current_swipe,
            total_swipes,
            turn_id,
            has_prev,
            has_next,
            prev_swipe,
            next_swipe,
        ) = if let Some((tid, csw, total)) = turn_data {
            let has_prev = csw > 0;
            let has_next = csw + 1 < total;
            let prev_swipe = csw.saturating_sub(1);
            let next_swipe = if has_next { csw + 1 } else { csw };
            (
                total > 1,
                csw,
                total,
                tid,
                has_prev,
                has_next,
                prev_swipe,
                next_swipe,
            )
        } else {
            (false, 0, 0, String::new(), false, false, 0, 0)
        };

        Self {
            is_disabled,
            error_message: error_msg,
            status_class,
            status_text,
            available_actions,
            show_swipe_nav,
            current_swipe,
            total_swipes,
            turn_id,
            has_prev,
            has_next,
            prev_swipe,
            next_swipe,
        }
    }
}

/// [DOC: docs/system/text_check.md]
#[derive(Template)]
#[template(
    source = r##"<div class=text-check-preview>
    <div class=preview-header>
        <span class=preview-icon>&#x270D;</span>
        <span>Did you mean?</span>
    </div>
    <div class=preview-original>
        <label>Original</label>
        <span>{{ original }}</span>
    </div>
    <div class=preview-corrected>
        <label>Corrected (edit if needed)</label>
        <textarea name=command class=preview-edit-textarea id=corrected-textarea>{{ corrected }}</textarea>
    </div>
    <div class=preview-issues>
        {% for issue in issues %}<span class="issue-tag {{ issue.kind }}">{{ issue.message }}</span>{% endfor %}
    </div>
    <div class=preview-actions>
        <form method=post hx-post=/action/confirm hx-target="#action-area" hx-swap="outerHTML" hx-include="#corrected-textarea">
            <button type=submit class=btn-corrected>Send</button>
        </form>
        <form method=post hx-post=/action/confirm hx-target="#action-area" hx-swap="outerHTML">
            <input type=hidden name=command value="{{ original }}" />
            <button type=submit class=btn-original>Send Original</button>
        </form>
        <button type=button class=btn-cancel onclick="restoreActionArea()">Cancel</button>
    </div>
</div>"##,
    ext = "html"
)]
pub struct TextCheckPreviewTemplate {
    pub original: String,
    pub corrected: String,
    pub issues: Vec<PreviewIssueView>,
}

#[derive(Debug, Clone)]
pub struct PreviewIssueView {
    pub message: String,
    pub kind: String,
}

impl TextCheckPreviewTemplate {
    pub fn from_check_result(result: &CheckResult) -> Self {
        let issues = result
            .issues
            .iter()
            .map(|issue| PreviewIssueView {
                message: issue.message.clone(),
                kind: match issue.kind {
                    crate::narrative::text_check::IssueKind::Spelling => "spell",
                    crate::narrative::text_check::IssueKind::Grammar => "grammar",
                    crate::narrative::text_check::IssueKind::Capitalization => "capitalization",
                    crate::narrative::text_check::IssueKind::Style => "style",
                    crate::narrative::text_check::IssueKind::Formatting => "formatting",
                    crate::narrative::text_check::IssueKind::Other => "other",
                }
                .to_string(),
            })
            .collect();

        Self {
            original: result.original.clone(),
            corrected: result.corrected.clone(),
            issues,
        }
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

#[derive(Template)]
#[template(
    source = r#"<div class="llm-message-list" id="llm-message-list">
{% for msg in messages %}
<div class="llm-message-card" id="llm-msg-{{ msg.id }}">
    <div class="llm-message-header" onclick="toggleLlmMessage(this)">
        <span class="llm-message-agent">{{ msg.agent_name }}</span>
        <span class="llm-message-model">{{ msg.backend_name }} / {{ msg.model_name }}</span>
        <span class="llm-message-time">{{ msg.timestamp }}</span>
        {% if msg.has_error %}<span class="llm-message-error">ERROR</span>{% endif %}
    </div>
    <div class="llm-message-body">
        <div class="llm-message-prompts">
            <details class="llm-message-prompt-details">
                <summary>System</summary>
                <pre class="llm-message-prompt-pre">{{ msg.system_prompt_preview }}</pre>
            </details>
            <details class="llm-message-prompt-details">
                <summary>User</summary>
                <pre class="llm-message-prompt-pre">{{ msg.user_prompt_preview }}</pre>
            </details>
            <details class="llm-message-prompt-details" open>
                <summary>Response</summary>
                <pre class="llm-message-prompt-pre">{{ msg.parsed_response_preview }}</pre>
            </details>
        </div>
        <div class="llm-message-raw">
            <details>
                <summary>Raw Request JSON</summary>
                <pre>{{ msg.raw_request_json }}</pre>
            </details>
            <details>
                <summary>Raw Response JSON</summary>
                <pre>{{ msg.raw_response_json }}</pre>
            </details>
        </div>
    </div>
</div>
{% endfor %}
{% if messages.is_empty() %}
<div class="llm-message-empty">No LLM messages yet.</div>
{% endif %}
</div>"#,
    ext = "html"
)]
pub struct LlmMessagesTemplate {
    pub messages: Vec<LlmMessageView>,
}

impl LlmMessagesTemplate {
    pub fn new(messages: &[LlmMessage]) -> Self {
        Self {
            messages: messages.iter().map(LlmMessageView::from).collect(),
        }
    }
}
