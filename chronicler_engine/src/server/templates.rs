//! [DOC: docs/architecture/system.md]

use std::fmt;

use askama::Template;
use pulldown_cmark::{Options, Parser, html};

use crate::model::state::{LogEntry, LogType};

// [DOC: docs/architecture/system.md]
#[allow(private_interfaces)]
#[derive(Debug, Clone)]
pub struct SafeHtml(String);

impl askama::filters::HtmlSafe for SafeHtml {}

impl fmt::Display for SafeHtml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn markdown_to_html(text: &str) -> String {
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
    pub fn new(status: &crate::model::state::GenerationStatus, exits: &[String]) -> Self {
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
            "Thinking...".to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_template_renders_room_name() {
        let template = HeaderTemplate {
            room_name: "Test Room".to_string(),
        };
        let rendered = template.render().unwrap();
        assert!(rendered.contains("Chronicler Engine"));
        assert!(rendered.contains(r#"class="header""#));
    }

    #[test]
    fn test_header_template_escapes_html() {
        let template = HeaderTemplate {
            room_name: "<script>alert('xss')</script>".to_string(),
        };
        let rendered = template.render().unwrap();
        assert!(rendered.contains("Chronicler Engine"));
    }

    #[test]
    fn test_header_template_connection_status() {
        let template = HeaderTemplate {
            room_name: "Any Room".to_string(),
        };
        let rendered = template.render().unwrap();
        assert!(rendered.contains(r#"id="connection-status""#));
        assert!(rendered.contains("Connected"));
    }

    #[test]
    fn test_story_log_template_empty() {
        let template = StoryLogTemplate::new(&[]);
        let rendered = template.render().unwrap();
        assert!(rendered.contains(r#"id="story-log""#));
    }

    #[test]
    fn test_story_log_template_with_entries() {
        use crate::model::state::LogType;
        use chrono::Utc;

        let entries = vec![LogEntry {
            id: 1,
            sender: Some("Game Master".to_string()),
            text: "Welcome to the adventure!".to_string(),
            log_type: LogType::Narration,
            timestamp: Utc::now(),
        }];
        let template = StoryLogTemplate::new(&entries);
        let rendered = template.render().unwrap();
        assert!(rendered.contains("Welcome to the adventure!"));
        assert!(rendered.contains("Game Master"));
        assert!(rendered.contains("narration"));
    }

    #[test]
    fn test_story_log_template_escapes_html() {
        use chrono::Utc;

        let entries = vec![LogEntry {
            id: 1,
            sender: None,
            text: "<script>alert('xss')</script>".to_string(),
            log_type: LogType::Narration,
            timestamp: Utc::now(),
        }];
        let template = StoryLogTemplate::new(&entries);
        let rendered = template.render().unwrap();
        assert!(!rendered.contains("<script>"));
    }

    #[test]
    fn test_story_log_template_renders_event_entry() {
        use chrono::Utc;

        let entries = vec![LogEntry {
            id: 1,
            sender: Some("Gabriella Introduction".to_string()),
            text: "".to_string(),
            log_type: LogType::Event,
            timestamp: Utc::now(),
        }];
        let template = StoryLogTemplate::new(&entries);
        let rendered = template.render().unwrap();
        assert!(rendered.contains("event-header"));
        assert!(rendered.contains("Gabriella Introduction"));
        assert!(rendered.contains("event-timestamp"));
        // Event entries should not have edit/retry buttons
        assert!(!rendered.contains("edit-btn"));
        assert!(!rendered.contains("retry-btn"));
    }

    #[test]
    fn test_visual_sidebar_with_image() {
        let template = VisualSidebarTemplate::new(
            Some("/images/room.png".to_string()),
            "Test Room".to_string(),
            vec![],
        );
        let rendered = template.render().unwrap();
        assert!(rendered.contains(r#"id="visual-sidebar""#));
        assert!(rendered.contains("/images/room.png"));
        assert!(rendered.contains("Test Room"));
    }

    #[test]
    fn test_visual_sidebar_no_image() {
        let template = VisualSidebarTemplate::new(None, "Test Room".to_string(), vec![]);
        let rendered = template.render().unwrap();
        assert!(rendered.contains("no-image"));
        assert!(rendered.contains("No Location Image"));
    }

    #[test]
    fn test_visual_sidebar_with_npcs() {
        let template = VisualSidebarTemplate::new(
            Some("/images/room.png".to_string()),
            "Test Room".to_string(),
            vec![
                ("/images/npc1.png".to_string(), "Alice".to_string()),
                ("/images/npc2.png".to_string(), "Bob".to_string()),
            ],
        );
        let rendered = template.render().unwrap();
        assert!(rendered.contains("npc-portrait"));
        assert!(rendered.contains("Alice"));
        assert!(rendered.contains("Bob"));
    }

    #[test]
    fn test_action_area_ready() {
        let template = ActionAreaTemplate::new(
            &crate::model::state::GenerationStatus::Idle,
            &["north".to_string(), "east".to_string()],
        );
        let rendered = template.render().unwrap();
        assert!(rendered.contains("id=cmd-area"));
        assert!(rendered.contains("Ready"));
        assert!(rendered.contains("Look"));
        assert!(rendered.contains("Inventory"));
        assert!(rendered.contains("north"));
        assert!(rendered.contains("east"));
    }

    #[test]
    fn test_action_area_thinking() {
        let template =
            ActionAreaTemplate::new(&crate::model::state::GenerationStatus::Generating, &[]);
        let rendered = template.render().unwrap();
        assert!(rendered.contains("Thinking..."));
        assert!(rendered.contains("disabled"));
    }

    #[test]
    fn test_action_area_no_exits() {
        let template = ActionAreaTemplate::new(&crate::model::state::GenerationStatus::Idle, &[]);
        let rendered = template.render().unwrap();
        assert!(rendered.contains("Look"));
        assert!(rendered.contains("Inventory"));
    }

    #[test]
    fn test_markdown_to_html_basic_quote() {
        let input = "\"Hello\"";
        let output = markdown_to_html(input);
        assert!(output.contains("<q>Hello</q>"));
    }

    #[test]
    fn test_markdown_to_html_multiple_quotes() {
        let input = "\"Well, well,\" Gabriella remarks, \"Welcome back\"";
        let output = markdown_to_html(input);
        assert!(output.contains("<q>Well, well,</q>"));
        assert!(output.contains("<q>Welcome back</q>"));
    }

    #[test]
    fn test_markdown_to_html_mixed_content() {
        let input = "She said \"Hello there\" and walked away.";
        let output = markdown_to_html(input);
        assert!(output.contains("<q>Hello there</q>"));
        assert!(output.contains("She said"));
        assert!(output.contains("and walked away"));
    }

    #[test]
    fn test_markdown_to_html_italic() {
        let input = "This is *italic* text.";
        let output = markdown_to_html(input);
        assert!(output.contains("<em>italic</em>"));
    }

    #[test]
    fn test_markdown_to_html_bold() {
        let input = "This is **bold** text.";
        let output = markdown_to_html(input);
        assert!(output.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_markdown_to_html_blockquote() {
        let input = "> This is a quote";
        let output = markdown_to_html(input);
        assert!(output.contains("<blockquote>"));
    }

    #[test]
    fn test_markdown_to_html_mixed_markdown() {
        let input = "**Bold** and *italic* and \"quoted\" text.";
        let output = markdown_to_html(input);
        assert!(output.contains("<strong>Bold</strong>"));
        assert!(output.contains("<em>italic</em>"));
        assert!(output.contains("<q>quoted</q>"));
    }

    #[test]
    fn test_markdown_to_html_no_quotes() {
        let input = "Plain text without quotes.";
        let output = markdown_to_html(input);
        assert_eq!(output, "<p>Plain text without quotes.</p>\n");
    }

    #[test]
    fn test_markdown_to_html_xss_prevention() {
        let input = "<script>alert('xss')</script>";
        let output = markdown_to_html(input);
        assert!(output.contains("&lt;script&gt;"));
        assert!(!output.contains("<script>"));
    }
}
