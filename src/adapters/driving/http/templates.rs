//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Template rendering utilities

use askama::Template;

use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::story_window::StoryWindowEntry;
use crate::application::ports::text_checker::CheckResult;
use crate::adapters::driving::http::view_models::{
    LlmMessageRecordView, LlmMessageRowView, StoryRowView, CharacterPortraitView,
    OptionsDockViewModel, PreviewIssueView, VisualSidebarViewModel,
};

#[derive(Template)]
#[template(
    source = r##"<span class="game-title">Chronicler Engine</span><span class="game-name">{{ game_name }}</span>"##,
    ext = "html"
)]
pub struct HeaderTemplate {
    pub game_name: String,
}

#[derive(Template)]
#[template(
    source = r##"{% macro disabled(on) %}{% if on %} disabled{% endif %}{% endmacro %}{% for entry in entries %}<div class="log-entry {{ entry.log_type }}{% if entry.room_header.is_some() %} location{% endif %}" id="entry-{{ entry.id }}" data-id="{{ entry.id }}" data-raw-text="{{ entry.raw_text | escape }}"><div class="message-header"><div class="message-info">{% if entry.room_header.is_some() %}<span class="location-header">{{ entry.room_header.as_ref().unwrap() }}</span><span class="location-timestamp">- {{ entry.timestamp }}</span>{% elif entry.event_header.is_some() %}<span class="event-header">{{ entry.event_header.as_ref().unwrap() }}</span><span class="event-timestamp">- {{ entry.timestamp }}</span>{% else %}<span class="timestamp">{{ entry.timestamp }}</span>{% endif %}</div><div class="message-actions"><button class="action-btn edit-btn" id="edit-entry-{{ entry.id }}" onclick="showEditForm({{ entry.id }})" title="Edit" aria-label="Edit"{{ disabled(view_only) }}><svg class="icon" aria-hidden="true"><use href="#i-pencil"/></svg></button>{% if loop.last && entries.len() > 1 %}<button class="action-btn delete-btn" onclick="deleteMessage()" title="Delete" aria-label="Delete"{{ disabled(view_only) }}><svg class="icon" aria-hidden="true"><use href="#i-trash"/></svg></button>{% endif %}{% if loop.last && entry.show_retrigger %}<button class="action-btn retrigger-btn" onclick="submitRetrigger()" title="Retrigger Event" aria-label="Retrigger Event"{{ disabled(view_only) }}><svg class="icon" aria-hidden="true"><use href="#i-zap"/></svg></button>{% endif %}</div></div><span class="text">{{ entry.text }}</span>{% if loop.last && (entry.log_type == "narration" || entry.log_type == "input") %}<div class="swipe-controls"><button class="action-btn swipe-btn" {% if entry.prev_swipe_index.is_none() || view_only %}disabled{% else %}onclick="switchSwipe({{ entry.id }}, {{ entry.prev_swipe_index.unwrap() }})"{% endif %} title="Previous swipe" aria-label="Previous swipe"><svg class="icon" aria-hidden="true"><use href="#i-chevron-left"/></svg></button><span class="swipe-counter">{{ entry.active_swipe_index + 1 }} / {{ entry.swipe_count }}</span>{% if entry.next_swipe_index.is_some() %}<button class="action-btn swipe-btn" onclick="switchSwipe({{ entry.id }}, {{ entry.next_swipe_index.unwrap() }})" title="Next swipe" aria-label="Next swipe"{{ disabled(view_only) }}><svg class="icon" aria-hidden="true"><use href="#i-chevron-right"/></svg></button>{% else %}<button class="action-btn swipe-btn" onclick="submitNewSwipe()" title="Retry" aria-label="Retry"{{ disabled(view_only) }}><svg class="icon" aria-hidden="true"><use href="#i-refresh-cw"/></svg></button>{% endif %}</div>{% endif %}</div>{% endfor %}"##,
    ext = "html"
)]
pub struct NarrationLogTemplate {
    pub entries: Vec<StoryRowView>,
    pub view_only: bool,
}

impl NarrationLogTemplate {
    pub fn new(entries: &[StoryWindowEntry], retrigger_allowed: bool, view_only: bool) -> Self {
        let mut views: Vec<StoryRowView> = entries.iter().map(StoryRowView::from).collect();
        if let Some(last) = views.last_mut() {
            last.show_retrigger = retrigger_allowed;
        }
        Self {
            entries: views,
            view_only,
        }
    }
}

#[derive(Template)]
#[template(
    source = r##"<div id="visual-sidebar" class="location-header-bar">{% if vm.room_has_image %}<div class="image-container location-image"><img src="{{ vm.room_src }}" alt="{{ vm.room_alt }}" /></div>{% else %}<div class="image-container no-image"><div class="placeholder">No Room Image</div></div>{% endif %}</div><div class="character-portrait-divider"></div><div class="character-portraits">{% for character in vm.characters %}<div class="image-container character-portrait"><img src="{{ character.image_path }}" alt="{{ character.name }}" /><div class="image-label" aria-hidden="true" title="{{ character.name }}">{{ character.name }}</div></div>{% endfor %}</div>"##,
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
    source = r##"{% for character in characters %}<div class="headshot" onclick="toggleVisualSidebar()"><img src="{{ character.image_path }}" alt="{{ character.name }}" /><div class="name">{{ character.name }}</div></div>{% endfor %}"##,
    ext = "html"
)]
pub struct CharacterHeadshotsTemplate {
    pub characters: Vec<CharacterPortraitView>,
}

impl CharacterHeadshotsTemplate {
    pub fn new(character_data: Vec<CharacterPortraitView>) -> Self {
        Self {
            characters: character_data,
        }
    }
}

/// An empty set of Options and no failure render an empty body.
#[derive(Template)]
#[template(
    source = r##"{% if vm.failure.is_some() %}<div class="options-notice" role="status"><span class="options-notice-line">{{ vm.failure.as_ref().unwrap().reason }}</span><details class="options-notice-details" data-keep="open"><summary>Details</summary><pre>{{ vm.failure.as_ref().unwrap().raw }}</pre></details></div>{% else %}{% if !vm.options.is_empty() %}<div class="options-strip"><div class="options-label"><span>options — pick one, or type your own</span><form class="options-regen-form" hx-post="/action/check" hx-swap="none" hx-sync="this:drop" hx-on::before-request="updateToThinking()" hx-on::after-request="endPendingAfterRequest(event)"><input type="hidden" name="command" value="/options" /><button type="submit" class="mini-btn" title="New options" aria-label="New options" {% if vm.is_busy %}disabled{% endif %}><svg class="icon" aria-hidden="true"><use href="#i-refresh-cw"/></svg></button></form></div>{% for option in vm.options %}<div class="option-item"><button type="button" class="option-btn" onclick="useOption(this)" {% if vm.is_busy %}disabled{% endif %}>{{ option }}</button><button type="button" class="mini-btn" onclick="editOption(this)" title="Edit before send" aria-label="Edit before send" {% if vm.is_busy %}disabled{% endif %}><svg class="icon" aria-hidden="true"><use href="#i-pencil"/></svg></button></div>{% endfor %}</div>{% endif %}{% endif %}"##,
    ext = "html"
)]
pub struct OptionsDockTemplate {
    pub vm: OptionsDockViewModel,
}

impl OptionsDockTemplate {
    pub fn new(vm: OptionsDockViewModel) -> Self {
        Self { vm }
    }
}

#[derive(Template)]
#[template(
    source = r##"<div class="text-check-preview" role="group" aria-label="Text check suggestions" data-shell-form>
    <div class="preview-header">
        <svg class="icon preview-icon" aria-hidden="true"><use href="#i-spell-check"/></svg>
        <span>Text check suggestions</span>
    </div>
    <p class="preview-note">Suggestions only — nothing changes unless you choose “Send with edits”.</p>
    <div class="preview-original">
        <label>Original</label>
        <span>{{ original }}</span>
    </div>
    <div class="preview-corrected">
        <label>Suggested text (edit freely)</label>
        <textarea name="command" class="preview-edit-textarea" id="corrected-textarea">{{ corrected }}</textarea>
    </div>
    <div class="preview-issues">
        {% for issue in issues %}<span class="issue-tag {{ issue.kind }}">{{ issue.message }}</span>{% endfor %}
    </div>
    {# The corrections are suggestions the player opts into, so Send Original keeps the primary style. #}
    <div class="form-actions">
        <form method="post" hx-post="/action/confirm" hx-swap="none" hx-on::before-request="onPreviewBeforeRequest()" hx-on::after-request="onPreviewSendAfterRequest(event)">
            <input type="hidden" name="command" value="{{ original }}" />
            <button type="submit" class="btn-primary btn-original">Send Original</button>
        </form>
        <form method="post" hx-post="/action/confirm" hx-swap="none" hx-include="#corrected-textarea" hx-on::before-request="onPreviewBeforeRequest()" hx-on::after-request="onPreviewSendAfterRequest(event)">
            <button type="submit" class="btn-cyan">Send with edits</button>
        </form>
        <button type="button" class="btn-cyan preview-cancel" onclick="closeActionPreview()">Cancel</button>
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
    source = r##"<div class="llm-message-list" id="llm-message-list">
{% for msg in messages %}
<div class="llm-message-card" id="llm-msg-{{ msg.id }}">
    <div class="llm-message-header">
        <span class="llm-message-agent">{{ msg.agent_name }}</span>
        <span class="llm-message-model">{{ msg.backend_name }} / {{ msg.model_name }}</span>
        <span class="llm-message-time">{{ msg.timestamp }}</span>
        {% if msg.error_summary.is_some() %}<span class="llm-message-error">ERROR</span>{% endif %}
        <button type="button" class="btn-cyan llm-message-open" id="llm-msg-open-{{ msg.id }}" hx-get="/fragment/llm-messages/{{ msg.id }}" hx-target="#edit-dialog-body" hx-swap="innerHTML" aria-label="Open the record of {{ msg.agent_name }} at {{ msg.timestamp }}">Open</button>
    </div>
    {% if let Some(summary) = msg.error_summary %}
    <div class="llm-message-error-summary">{{ summary }}</div>
    {% endif %}
</div>
{% endfor %}
{% if messages.is_empty() %}
<div class="llm-message-empty">No LLM messages yet.</div>
{% endif %}
</div>"##,
    ext = "html"
)]
pub struct LlmMessagesTemplate {
    pub messages: Vec<LlmMessageRowView>,
}

impl LlmMessagesTemplate {
    pub fn new(messages: &[LlmMessage]) -> Self {
        Self {
            messages: messages.iter().map(LlmMessageRowView::from).collect(),
        }
    }
}

#[derive(Template)]
#[template(
    source = r##"<div class="llm-message-record" data-dialog-form>
    <h2 id="edit-dialog-title">{{ record.agent_name }} · {{ record.timestamp }}</h2>
    <p class="llm-message-record-model">{{ record.backend_name }} / {{ record.model_name }}</p>
    {% if record.error_message.is_some() %}
    <div class="llm-message-failure">
        <span class="llm-message-failure-label">Failure</span>
        <pre class="llm-message-failure-text">{{ record.error_message.as_deref().unwrap_or_default() }}</pre>
    </div>
    {% endif %}
    {% if !record.system_prompt.is_empty() || !record.user_prompt.is_empty() || !record.parsed_response.is_empty() %}
    <div class="llm-message-prompts">
        {% if !record.system_prompt.is_empty() %}
        <details class="llm-message-prompt-details">
            <summary>System</summary>
            <pre class="llm-message-prompt-pre">{{ record.system_prompt }}</pre>
        </details>
        {% endif %}
        {% if !record.user_prompt.is_empty() %}
        <details class="llm-message-prompt-details">
            <summary>User</summary>
            <pre class="llm-message-prompt-pre">{{ record.user_prompt }}</pre>
        </details>
        {% endif %}
        {% if !record.parsed_response.is_empty() %}
        <details class="llm-message-prompt-details" open>
            <summary>Response</summary>
            <pre class="llm-message-prompt-pre">{{ record.parsed_response }}</pre>
        </details>
        {% endif %}
    </div>
    {% endif %}
    {% if !record.raw_request_json.is_empty() || !record.raw_response_json.is_empty() %}
    <div class="llm-message-raw">
        {% if !record.raw_request_json.is_empty() %}
        <details>
            <summary>Raw Request JSON</summary>
            <pre>{{ record.raw_request_json }}</pre>
        </details>
        {% endif %}
        {% if !record.raw_response_json.is_empty() %}
        <details>
            <summary>Raw Response JSON</summary>
            <pre>{{ record.raw_response_json }}</pre>
        </details>
        {% endif %}
    </div>
    {% endif %}
    <div class="form-actions">
        <button type="button" class="btn-cyan" data-close-dialog>Close</button>
    </div>
</div>"##,
    ext = "html"
)]
pub struct LlmMessageRecordTemplate {
    pub record: LlmMessageRecordView,
}

impl LlmMessageRecordTemplate {
    pub fn new(message: &LlmMessage) -> Self {
        Self {
            record: LlmMessageRecordView::from(message),
        }
    }
}
