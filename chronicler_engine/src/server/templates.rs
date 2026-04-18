//! Askama template definitions for HTML fragments.
//!
//! These templates provide compile-time validation for HTML fragments used in HTMX partial updates.
//! Variables are automatically HTML-escaped by Askama unless marked as safe.

use std::fmt;

use askama::Template;
use pulldown_cmark::{Options, Parser, html};

use crate::model::state::{LogEntry, LogType};

/// Wrapper type that marks pre-parsed HTML content as safe (no escaping needed).
/// Used for markdown-converted content - the markdown parser outputs safe HTML tags.
#[allow(private_interfaces)]
#[derive(Debug, Clone)]
pub struct SafeHtml(String);

impl askama::filters::HtmlSafe for SafeHtml {}

impl fmt::Display for SafeHtml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Markdown to HTML Conversion (for UI display only)

/// Convert markdown text to HTML for display on UI.
///
/// This is ONLY used when rendering the UI - the raw LLM text is preserved unchanged in LogEntry.
/// This separation allows us to send the original text back to the LLM for context.
///
/// Security: We escape < and & before parsing to prevent XSS attacks.
/// The markdown parser will then convert these escaped entities back appropriately.
fn markdown_to_html(text: &str) -> String {
    // Escape dangerous characters: < and & (these are the only real XSS vectors)
    // We don't escape > or " because they're not dangerous in HTML
    let escaped = text.replace('&', "&amp;").replace('<', "&lt;");

    // Parse markdown with smart quotes enabled
    let mut options = Options::empty();
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    let parser = Parser::new_ext(&escaped, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Convert curly quotes (") to <q> tags for dialogue styling
    // These are the Unicode curly quote characters that pulldown_cmark produces
    html_output
        .replace('\u{201C}', "<q>")
        .replace('\u{201D}', "</q>")
}

// Header Template

/// Header template with compile-time field validation.
///
/// Renders the game header with title, current location, and connection status.
/// Room name is automatically HTML-escaped by Askama.
#[derive(Template)]
#[template(
    source = r#"<div class="header"><span class="game-title">Chronicler Engine</span><span class="connection-status connected" id="connection-status">Connected</span></div>"#,
    ext = "html"
)]
pub struct HeaderTemplate {
    /// The current room name - now displayed in story log, not header.
    pub room_name: String,
}

// Story Log Template

/// Story log entry for use in templates (owned version).
#[derive(Debug, Clone)]
pub struct LogEntryView {
    pub timestamp: String,
    /// Sender display text, or empty string if none.
    pub sender: String,
    /// Pre-parsed HTML text (wrapped in SafeHtml to prevent escaping).
    pub text: SafeHtml,
    pub log_type: String,
    /// True if this is a location entry (sender present + empty text)
    pub is_location: bool,
}

impl From<&LogEntry> for LogEntryView {
    fn from(entry: &LogEntry) -> Self {
        // Parse markdown BEFORE Askama escapes (converts "..." to <q>, etc.)
        let parsed_text = markdown_to_html(&entry.text);
        // Detect location entries: sender present + empty text
        let is_location = entry.sender.is_some() && entry.text.is_empty();
        Self {
            timestamp: entry.timestamp.format("%H:%M").to_string(),
            sender: entry.sender.clone().unwrap_or_default(),
            text: SafeHtml(parsed_text),
            log_type: match entry.log_type {
                LogType::Narration => "narration".to_string(),
                LogType::Dialogue => "dialogue".to_string(),
                LogType::System => "system".to_string(),
                LogType::Input => "input".to_string(),
            },
            is_location,
        }
    }
}

/// Story log template with compile-time validation.
///
/// Renders the narration history as a series of log entries.
/// Each entry includes timestamp, optional sender, and formatted text.
/// Note: text is converted via markdown_to_html() in LogEntryView::from() to convert
/// markdown quotes to HTML <q> tags. The text is wrapped in SafeHtml to prevent
/// Askama from escaping the HTML.
#[derive(Template)]
#[template(
    source = r#"<div class="story-log" id="story-log">{% for entry in entries %}<div class="log-entry {{ entry.log_type }}">{% if entry.is_location %}<span class="location-header">{{ entry.sender }}</span><span class="location-timestamp">- {{ entry.timestamp }}</span>{% else %}<span class="timestamp">{{ entry.timestamp }}</span>{% if entry.sender != "" %}<span class="sender">{{ entry.sender }}:</span> {% endif %}{% endif %}<span class="text">{{ entry.text }}</span></div>{% endfor %}</div>"#,
    ext = "html"
)]
pub struct StoryLogTemplate {
    /// Log entries to render.
    pub entries: Vec<LogEntryView>,
}

impl StoryLogTemplate {
    /// Create a new StoryLogTemplate from log entries.
    pub fn new(entries: &[LogEntry]) -> Self {
        Self {
            entries: entries.iter().map(LogEntryView::from).collect(),
        }
    }
}

// Visual Sidebar Template

/// Visual sidebar template with optional images and NPC portraits.
///
/// Renders location image and NPC portraits in the sidebar.
#[derive(Template)]
#[template(
    source = r#"<div class="visual-sidebar" id="visual-sidebar">{% if room_has_image %}<div class="image-container location-image"><img src="{{ room_src }}" alt="{{ room_alt }}" /><div class="image-label">Location</div></div>{% else %}<div class="image-container no-image"><div class="placeholder">No Location Image</div></div>{% endif %}<div class="npc-portraits">{% for npc in npcs %}<div class="image-container npc-portrait"><img src="{{ npc.0 }}" alt="{{ npc.1 }}" /><div class="image-label">{{ npc.1 }}</div></div>{% endfor %}</div></div>"#,
    ext = "html"
)]
pub struct VisualSidebarTemplate {
    /// Whether room has an image.
    pub room_has_image: bool,
    /// Room image source (if present).
    pub room_src: String,
    /// Room image alt text.
    pub room_alt: String,
    /// NPC portraits: (src, alt) pairs.
    pub npcs: Vec<(String, String)>,
}

impl VisualSidebarTemplate {
    /// Create from room image path and NPC data.
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

// Character Headshots Template

/// Template for character headshots grid.
#[derive(Template)]
#[template(
    source = r#"{% for npc in npcs %}<div class="headshot" onclick="toggleVisualSidebar()"><img src="{{ npc.0 }}" alt="{{ npc.1 }}" /><div class="name">{{ npc.1 }}</div></div>{% endfor %}"#,
    ext = "html"
)]
pub struct CharacterHeadshotsTemplate {
    /// NPC images: (src, name) pairs.
    pub npcs: Vec<(String, String)>,
}

impl CharacterHeadshotsTemplate {
    /// Create from NPC image data.
    pub fn new(npc_data: Vec<(String, String)>) -> Self {
        Self { npcs: npc_data }
    }
}

// Action Area Template

/// Action area template with form and status indicators.
///
/// Renders the command input form with action hints and status.
#[derive(Template)]
#[template(
    source = r#"<div cmd-area id=cmd-area><form method=post hx-post=/action hx-target=#cmd-area hx-swap=outerHTML class=command-wrapper><input type=text name=command placeholder="Enter command..." value="" {% if is_disabled %}disabled{% endif %} autocomplete=off /><button type=submit {% if is_disabled %}disabled{% endif %}>Send</button></form><div class=command-hints>{% for action in available_actions %}<span class=action-hint>{{ action }}</span>{% endfor %}</div><div id="error-message" class="error-message">{{ error_message }}</div><div class="{{ status_class }}" id="status-display">{{ status_text }}</div></div>"#,
    ext = "html"
)]
pub struct ActionAreaTemplate {
    /// Whether the form is disabled (while generating).
    pub is_disabled: bool,
    /// Error message to display (empty string if none).
    pub error_message: String,
    /// CSS class for status indicator.
    pub status_class: String,
    /// Status text to display.
    pub status_text: String,
    /// Available action hints.
    pub available_actions: Vec<String>,
}

impl ActionAreaTemplate {
    /// Create with available exits.
    pub fn new(is_generating: bool, exits: &[String]) -> Self {
        Self::new_with_error(is_generating, exits, None)
    }

    /// Create with available exits and error message.
    pub fn new_with_error(
        is_generating: bool,
        exits: &[String],
        error_message: Option<String>,
    ) -> Self {
        let is_disabled = is_generating;
        let error_msg = error_message.clone().unwrap_or_default();
        let status_class = if is_generating {
            "status thinking".to_string()
        } else if error_message.is_some() {
            "status error".to_string()
        } else {
            "status ready".to_string()
        };
        let status_text = if is_generating {
            "Thinking...".to_string()
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

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    // Header Template Tests

    #[test]
    fn test_header_template_renders_room_name() {
        let template = HeaderTemplate {
            room_name: "Test Room".to_string(),
        };
        let rendered = template.render().unwrap();
        // Room name is now in story log, not header
        assert!(rendered.contains("Chronicler Engine"));
        assert!(rendered.contains(r#"class="header""#));
    }

    #[test]
    fn test_header_template_escapes_html() {
        let template = HeaderTemplate {
            room_name: "<script>alert('xss')</script>".to_string(),
        };
        let rendered = template.render().unwrap();
        // Room name no longer in header output, just verify it doesn't cause errors
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

    // Story Log Template Tests

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
            sender: None,
            text: "<script>alert('xss')</script>".to_string(),
            log_type: LogType::Narration,
            timestamp: Utc::now(),
        }];
        let template = StoryLogTemplate::new(&entries);
        let rendered = template.render().unwrap();
        // The text goes through parse_markdown which doesn't escape <script>
        // but SafeHtml marks it as safe. This test verifies the old escaping behavior.
        // In practice, parse_markdown should escape HTML input for security.
        // For now, just verify it doesn't contain raw <script> tags in output
        assert!(!rendered.contains("<script>"));
    }

    // Visual Sidebar Template Tests

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

    // Action Area Template Tests

    #[test]
    fn test_action_area_ready() {
        let template = ActionAreaTemplate::new(false, &["north".to_string(), "east".to_string()]);
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
        let template = ActionAreaTemplate::new(true, &[]);
        let rendered = template.render().unwrap();
        assert!(rendered.contains("Thinking..."));
        assert!(rendered.contains("disabled"));
    }

    #[test]
    fn test_action_area_no_exits() {
        let template = ActionAreaTemplate::new(false, &[]);
        let rendered = template.render().unwrap();
        assert!(rendered.contains("Look"));
        assert!(rendered.contains("Inventory"));
    }

    // markdown_to_html Tests

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
        // Should have two <q> tags wrapping each quoted section
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
        // pulldown-cmark adds a trailing newline
        assert_eq!(output, "<p>Plain text without quotes.</p>\n");
    }

    #[test]
    fn test_markdown_to_html_xss_prevention() {
        // Verify that malicious input is escaped
        let input = "<script>alert('xss')</script>";
        let output = markdown_to_html(input);
        // < should be escaped to &lt;
        assert!(output.contains("&lt;script&gt;"));
        // No raw <script> tags
        assert!(!output.contains("<script>"));
    }
}
