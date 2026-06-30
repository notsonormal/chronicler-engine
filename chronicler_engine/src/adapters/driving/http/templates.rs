//! [DOC: docs/system/dashboard.md]
//! Template rendering utilities

use askama::Template;

use crate::application::ports::llm_message_repository::LlmMessage;
use crate::domain::model::state::message_types::MessageEntry;
use crate::adapters::driven::text_check::CheckResult;
use crate::adapters::driving::http::view_models::{
    ActionAreaViewModel, LlmMessageView, MessageEntryView, NpcPortraitView, PreviewIssueView,
    VisualSidebarViewModel,
};

#[derive(Template)]
#[template(
    source = r##"<div class="header"><span class="game-title">Chronicler Engine</span><span class="game-name">{{ game_name }}</span><span class="connection-status connected" id="connection-status">Connected</span></div>"##,
    ext = "html"
)]
pub struct HeaderTemplate {
    pub game_name: String,
}

#[derive(Template)]
#[template(
    source = r#"<div class="story-log" id="story-log">{% for entry in entries %}<div class="log-entry {{ entry.log_type }}{% if entry.location_header.is_some() %} location{% endif %}" data-id="{{ entry.id }}" data-raw-text="{{ entry.raw_text | escape }}"><div class="message-header"><div class="message-info">{% if entry.location_header.is_some() %}<span class="location-header">{{ entry.location_header.as_ref().unwrap() }}</span><span class="location-timestamp">- {{ entry.timestamp }}</span>{% elif entry.event_header.is_some() %}<span class="event-header">{{ entry.event_header.as_ref().unwrap() }}</span><span class="event-timestamp">- {{ entry.timestamp }}</span>{% else %}<span class="timestamp">{{ entry.timestamp }}</span>{% if entry.sender != "" %}<span class="sender">{{ entry.sender }}:</span>{% endif %}{% endif %}</div><div class="message-actions"><button class="action-btn edit-btn" onclick="showEditForm({{ entry.id }})" title="Edit">&#9998;</button>{% if loop.last && entries.len() > 1 %}<button class="action-btn delete-btn" onclick="deleteMessage()" title="Delete">&#128465;</button>{% endif %}{% if entry.log_type == "input" %}<button class="action-btn check-btn" onclick="checkLogText(this.closest('.log-entry').dataset.rawText)" title="Check spelling & grammar">&#x2713;</button>{% endif %}{% if loop.last && entry.show_retrigger %}<button class="action-btn retrigger-btn" onclick="submitRetrigger()" title="Retrigger Event">&#9851;</button>{% endif %}</div></div><span class="text">{{ entry.text }}</span>{% if loop.last && (entry.log_type == "narration" || entry.log_type == "dialogue") %}<div class="swipe-controls"><button class="action-btn swipe-btn" {% if entry.prev_swipe_index.is_none() %}disabled{% else %}onclick="switchSwipe({{ entry.id }}, {{ entry.prev_swipe_index.unwrap() }})"{% endif %} title="Previous swipe">&#9664;</button><span class="swipe-counter">{{ entry.active_swipe_index + 1 }} / {{ entry.swipe_count }}</span><button class="action-btn swipe-btn" {% if entry.next_swipe_index.is_some() %}onclick="switchSwipe({{ entry.id }}, {{ entry.next_swipe_index.unwrap() }})"{% else %}onclick="submitNewSwipe()"{% endif %} title="{% if entry.next_swipe_index.is_some() %}Next swipe{% else %}Retry{% endif %}">&#9654;</button></div>{% endif %}</div>{% endfor %}</div>"#,
    ext = "html"
)]
pub struct StoryLogTemplate {
    pub entries: Vec<MessageEntryView>,
}

impl StoryLogTemplate {
    pub fn new(entries: &[MessageEntry], has_last_trigger: bool) -> Self {
        let mut views: Vec<MessageEntryView> = entries.iter().map(MessageEntryView::from).collect();
        if let Some(last) = views.last_mut() {
            let is_narration = last.log_type == "narration" || last.log_type == "dialogue";
            let is_event_continuation = last.event_header.is_some();
            last.show_retrigger = has_last_trigger && is_narration && !is_event_continuation;
        }
        Self { entries: views }
    }
}

#[derive(Template)]
#[template(
    source = r#"<div id="visual-sidebar" class="location-header-bar">{% if vm.room_has_image %}<div class="image-container location-image"><img src="{{ vm.room_src }}" alt="{{ vm.room_alt }}" /></div>{% else %}<div class="image-container no-image"><div class="placeholder">No Location Image</div></div>{% endif %}</div><div class="npc-portrait-divider"></div><div class="npc-portraits">{% for npc in vm.npcs %}<div class="image-container npc-portrait"><img src="{{ npc.image_path }}" alt="{{ npc.name }}" /></div>{% endfor %}</div>"#,
    ext = "html"
)]
pub struct VisualSidebarTemplate {
    pub vm: VisualSidebarViewModel,
}

impl VisualSidebarTemplate {
    pub fn new(vm: VisualSidebarViewModel) -> Self {
        Self { vm }
    }
}

#[derive(Template)]
#[template(
    source = r#"{% for npc in npcs %}<div class="headshot" onclick="toggleVisualSidebar()"><img src="{{ npc.image_path }}" alt="{{ npc.name }}" /><div class="name">{{ npc.name }}</div></div>{% endfor %}"#,
    ext = "html"
)]
pub struct CharacterHeadshotsTemplate {
    pub npcs: Vec<NpcPortraitView>,
}

impl CharacterHeadshotsTemplate {
    pub fn new(npc_data: Vec<NpcPortraitView>) -> Self {
        Self { npcs: npc_data }
    }
}

#[derive(Template)]
#[template(
    source = r##"<div class="action-area" id="action-area"><form id="command-form" hx-post="/action/check" hx-target="#action-area" hx-swap="innerHTML" hx-sync="this:drop" hx-on::before-request="saveActionArea()" hx-on::after-request="onActionFormAfterRequest()"><input type="text" name="command" placeholder="Enter command..." required minlength="1" autocomplete="off" {% if vm.is_disabled %}disabled{% endif %} /><button type="submit" id="submit-btn" {% if vm.is_disabled %}disabled{% endif %}><span class="btn-icon">&#9654;</span> Send</button></form><div class="{{ vm.status_class }}" id="status-display" hx-get="/status/generating" hx-trigger="load, every 5s" hx-swap="innerHTML" hx-on::after-swap="onStatusPoll(this)"><span class="{{ vm.status_class }}">{{ vm.status_text }}</span></div></div>"##,
    ext = "html"
)]
pub struct ActionAreaTemplate {
    pub vm: ActionAreaViewModel,
}

impl ActionAreaTemplate {
    pub fn new(vm: ActionAreaViewModel) -> Self {
        Self { vm }
    }
}

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
    <div class="form-actions">
        <form method=post hx-post=/action/confirm hx-target="#action-area" hx-swap="outerHTML" hx-include="#corrected-textarea">
            <button type=submit class="btn-primary">Send</button>
        </form>
        <form method=post hx-post=/action/confirm hx-target="#action-area" hx-swap="outerHTML">
            <input type=hidden name=command value="{{ original }}" />
            <button type=submit class="btn-cyan">Send Original</button>
        </form>
        <button type=button class="btn-cyan" onclick="restoreActionArea()">Cancel</button>
    </div>
</div>"##,
    ext = "html"
)]
pub struct TextCheckPreviewTemplate {
    pub original: String,
    pub corrected: String,
    pub issues: Vec<PreviewIssueView>,
}

impl TextCheckPreviewTemplate {
    pub fn from_check_result(result: &CheckResult) -> Self {
        Self {
            original: result.original.clone(),
            corrected: result.corrected.clone(),
            issues: PreviewIssueView::from_check_result(result),
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
