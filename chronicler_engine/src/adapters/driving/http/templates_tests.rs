use askama::Template;
use chrono::Utc;

use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::adapters::driving::http::templates::{
    ActionAreaTemplate, HeaderTemplate, NarrativeLogTemplate, VisualSidebarTemplate,
};
use crate::adapters::driving::http::view_models::{
    ActionAreaViewModel, NpcPortraitView, VisualSidebarViewModel,
};

#[test]
fn test_header_template_renders_game_name() {
    let template = HeaderTemplate {
        game_name: "Test Game".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(rendered.contains("Chronicler Engine"));
    assert!(rendered.contains(r#"class="header""#));
    assert!(rendered.contains(r#"class="game-title""#));
    assert!(rendered.contains("connection-status"));
}

#[test]
fn test_header_template_escapes_html() {
    let template = HeaderTemplate {
        game_name: "<script>alert('xss')</script>".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(rendered.contains("Chronicler Engine"));
    assert!(!rendered.contains("<script>"));
}

#[test]
fn test_header_template_connection_status() {
    let template = HeaderTemplate {
        game_name: "Any Game".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(rendered.contains(r#"id="connection-status""#));
    assert!(rendered.contains("Connected"));
}

#[test]
fn test_story_log_template_empty() {
    let template = NarrativeLogTemplate::new(&[], false);
    let rendered = template.render().unwrap();
    assert!(rendered.contains(r#"id="story-log""#));
}

#[test]
fn test_story_log_template_with_entries() {
    let entries = vec![MessageEntry {
        id: 1,
        sender: Some("Game Master".to_string()),
        text: "Welcome to the adventure!".to_string(),
        message_type: MessageType::Narration,
        timestamp: Utc::now(),
        ..Default::default()
    }];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(rendered.contains("Welcome to the adventure!"));
    assert!(rendered.contains("Game Master"));
    assert!(rendered.contains("narration"));
}

#[test]
fn test_story_log_template_escapes_html() {
    let entries = vec![MessageEntry {
        id: 1,
        sender: None,
        text: "<script>alert('xss')</script>".to_string(),
        message_type: MessageType::Narration,
        timestamp: Utc::now(),
        ..Default::default()
    }];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(!rendered.contains("<script>"));
}

#[test]
fn test_story_log_template_has_message_actions() {
    let entries = vec![
        MessageEntry {
            id: 1,
            sender: Some("Game Master".to_string()),
            text: "Welcome!".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
        MessageEntry {
            id: 2,
            sender: Some("Player".to_string()),
            text: "Hello".to_string(),
            message_type: MessageType::Input,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(rendered.contains("message-header"));
    assert!(rendered.contains("message-actions"));
    assert!(rendered.contains("edit-btn"));
    assert!(rendered.contains("delete-btn"));
}

#[test]
fn test_story_log_template_input_has_check_button() {
    let entries = vec![
        MessageEntry {
            id: 1,
            sender: Some("Player".to_string()),
            text: "look around".to_string(),
            message_type: MessageType::Input,
            timestamp: Utc::now(),
            ..Default::default()
        },
        MessageEntry {
            id: 2,
            sender: None,
            text: "You look around.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(rendered.contains("check-btn"));
    assert!(rendered.contains("delete-btn"));
}

#[test]
fn test_story_log_template_renders_event_header() {
    let entries = vec![
        MessageEntry {
            id: 1,
            sender: None,
            text: "Gabriella steps forward.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            event_header: Some("Gabriella Introduction".to_string()),
            ..Default::default()
        },
        MessageEntry {
            id: 2,
            sender: None,
            text: "She smiles.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(rendered.contains("event-header"));
    assert!(rendered.contains("Gabriella Introduction"));
    assert!(rendered.contains("event-timestamp"));
    assert!(rendered.contains("edit-btn"));
    assert!(rendered.contains("delete-btn"));
    assert!(!rendered.contains("check-btn"));
    // Retry button removed — swipe right arrow handles new swipe generation
    assert!(!rendered.contains("retry-btn"));
}

#[test]
fn test_story_log_template_renders_location_header() {
    let entries = vec![
        MessageEntry {
            id: 1,
            sender: None,
            text: "You walk into the hall.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            location_header: Some("Entrance Hall".to_string()),
            ..Default::default()
        },
        MessageEntry {
            id: 2,
            sender: None,
            text: "The hall is grand.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(rendered.contains("location-header"));
    assert!(rendered.contains("Entrance Hall"));
    assert!(rendered.contains("location-timestamp"));
    assert!(rendered.contains("edit-btn"));
    assert!(rendered.contains("delete-btn"));
}

#[test]
fn test_story_log_template_retrigger_button_shown_when_last_trigger_present() {
    let entries = vec![
        MessageEntry {
            id: 1,
            sender: Some("Player".to_string()),
            text: "look around".to_string(),
            message_type: MessageType::Input,
            timestamp: Utc::now(),
            ..Default::default()
        },
        MessageEntry {
            id: 2,
            sender: None,
            text: "You look around.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ];
    let template = NarrativeLogTemplate::new(&entries, true);
    let rendered = template.render().unwrap();
    assert!(rendered.contains("retrigger-btn"));
    assert!(rendered.contains("submitRetrigger"));
}

#[test]
fn test_story_log_template_retrigger_button_hidden_on_event_continuation() {
    let entries = vec![
        MessageEntry {
            id: 1,
            sender: None,
            text: "You look around.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
        MessageEntry {
            id: 2,
            sender: None,
            text: "An NPC appears.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            event_header: Some("NPC Event".to_string()),
            ..Default::default()
        },
    ];
    let template = NarrativeLogTemplate::new(&entries, true);
    let rendered = template.render().unwrap();
    // Last entry is an event continuation, so retrigger should NOT appear
    assert!(!rendered.contains("retrigger-btn"));
}

#[test]
fn test_story_log_template_retrigger_button_hidden_without_last_trigger() {
    let entries = vec![MessageEntry {
        id: 1,
        sender: None,
        text: "You look around.".to_string(),
        message_type: MessageType::Narration,
        timestamp: Utc::now(),
        ..Default::default()
    }];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(!rendered.contains("retrigger-btn"));
}

#[test]
fn test_story_log_template_swipe_controls_on_last_narration_with_one_swipe() {
    let entries = vec![MessageEntry {
        id: 1,
        sender: None,
        text: "You look around.".to_string(),
        message_type: MessageType::Narration,
        timestamp: Utc::now(),
        ..Default::default()
    }];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains("swipe-controls"),
        "Swipe controls should appear on last narration"
    );
    assert!(
        rendered.contains("submitNewSwipe()"),
        "Right arrow should call submitNewSwipe on latest swipe"
    );
    assert!(rendered.contains("Previous swipe"));
    assert!(rendered.contains("Retry"));
    assert!(rendered.contains("1 / 1"));
}

#[test]
fn test_story_log_template_swipe_controls_on_last_dialogue_with_one_swipe() {
    let entries = vec![MessageEntry {
        id: 1,
        sender: Some("NPC".to_string()),
        text: "Hello there.".to_string(),
        message_type: MessageType::Dialogue,
        timestamp: Utc::now(),
        ..Default::default()
    }];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains("swipe-controls"),
        "Swipe controls should appear on last dialogue"
    );
    assert!(rendered.contains("submitNewSwipe()"));
}

#[test]
fn test_story_log_template_no_swipe_controls_on_input() {
    let entries = vec![MessageEntry {
        id: 1,
        sender: Some("Player".to_string()),
        text: "look around".to_string(),
        message_type: MessageType::Input,
        timestamp: Utc::now(),
        ..Default::default()
    }];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(
        !rendered.contains("swipe-controls"),
        "Swipe controls should not appear on input"
    );
}

#[test]
fn test_story_log_template_no_swipe_controls_on_system() {
    let entries = vec![MessageEntry {
        id: 1,
        sender: None,
        text: "System message.".to_string(),
        message_type: MessageType::System,
        timestamp: Utc::now(),
        ..Default::default()
    }];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(
        !rendered.contains("swipe-controls"),
        "Swipe controls should not appear on system"
    );
}

#[test]
fn test_story_log_template_no_swipe_controls_on_non_last_narration() {
    let entries = vec![
        MessageEntry {
            id: 1,
            sender: None,
            text: "First narration.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
        MessageEntry {
            id: 2,
            sender: Some("Player".to_string()),
            text: "hello".to_string(),
            message_type: MessageType::Input,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(
        !rendered.contains("swipe-controls"),
        "Swipe controls should not appear on non-last entry"
    );
}

#[test]
fn test_story_log_template_swipe_navigation_between_existing_swipes() {
    let mut entry = MessageEntry {
        id: 1,
        sender: None,
        text: "First swipe.".to_string(),
        message_type: MessageType::Narration,
        timestamp: Utc::now(),
        ..Default::default()
    };
    entry.swipe_count = 3;
    entry.active_swipe_index = 1;
    let entries = vec![entry];
    let template = NarrativeLogTemplate::new(&entries, false);
    let rendered = template.render().unwrap();
    assert!(rendered.contains("swipe-controls"));
    assert!(rendered.contains("2 / 3"));
    assert!(
        rendered.contains("switchSwipe(1, 0)"),
        "Left arrow should navigate to previous swipe"
    );
    assert!(
        rendered.contains("switchSwipe(1, 2)"),
        "Right arrow should navigate to next swipe"
    );
    assert!(
        !rendered.contains("submitNewSwipe()"),
        "Right arrow should not call submitNewSwipe when next swipe exists"
    );
}

#[test]
fn test_visual_sidebar_with_image() {
    let template = VisualSidebarTemplate::new(VisualSidebarViewModel::new(
        Some("/images/room.png".to_string()),
        "Test Room".to_string(),
        vec![],
    ));
    let rendered = template.render().unwrap();
    assert!(rendered.contains(r#"id="visual-sidebar""#));
    assert!(rendered.contains("/images/room.png"));
    assert!(rendered.contains("Test Room"));
}

#[test]
fn test_visual_sidebar_no_image() {
    let template = VisualSidebarTemplate::new(VisualSidebarViewModel::new(
        None,
        "Test Room".to_string(),
        vec![],
    ));
    let rendered = template.render().unwrap();
    assert!(rendered.contains("no-image"));
    assert!(rendered.contains("No Location Image"));
}

#[test]
fn test_visual_sidebar_with_npcs() {
    let template = VisualSidebarTemplate::new(VisualSidebarViewModel::new(
        Some("/images/room.png".to_string()),
        "Test Room".to_string(),
        vec![
            NpcPortraitView {
                image_path: "/images/npc1.png".to_string(),
                name: "Alice".to_string(),
            },
            NpcPortraitView {
                image_path: "/images/npc2.png".to_string(),
                name: "Bob".to_string(),
            },
        ],
    ));
    let rendered = template.render().unwrap();
    assert!(rendered.contains("npc-portrait"));
    assert!(rendered.contains("Alice"));
    assert!(rendered.contains("Bob"));
}

#[test]
fn test_action_area_ready() {
    let template = ActionAreaTemplate::new(ActionAreaViewModel::new(
        &GenerationStatus::Idle,
        &GenerationPhase::default(),
    ));
    let rendered = template.render().unwrap();
    assert!(rendered.contains("id=\"action-area\""));
    assert!(rendered.contains("Ready"));
}

#[test]
fn test_action_area_thinking() {
    let template = ActionAreaTemplate::new(ActionAreaViewModel::new(
        &GenerationStatus::Generating,
        &GenerationPhase::Narrating,
    ));
    let rendered = template.render().unwrap();
    assert!(rendered.contains("Generating narration..."));
    assert!(rendered.contains("disabled"));
}

#[test]
fn test_action_area_quantifying() {
    let template = ActionAreaTemplate::new(ActionAreaViewModel::new(
        &GenerationStatus::Generating,
        &GenerationPhase::Quantifying,
    ));
    let rendered = template.render().unwrap();
    assert!(rendered.contains("Quantifying scene..."));
    assert!(rendered.contains("disabled"));
}

#[test]
fn test_action_area_generating_event() {
    let template = ActionAreaTemplate::new(ActionAreaViewModel::new(
        &GenerationStatus::Generating,
        &GenerationPhase::GeneratingEvent,
    ));
    let rendered = template.render().unwrap();
    assert!(rendered.contains("Generating event..."));
    assert!(rendered.contains("disabled"));
}

#[test]
fn test_action_area_no_exits() {
    let template = ActionAreaTemplate::new(ActionAreaViewModel::new(
        &GenerationStatus::Idle,
        &GenerationPhase::default(),
    ));
    let rendered = template.render().unwrap();
    assert!(rendered.contains("command-form"));
}

#[test]
fn test_markdown_to_html_basic_quote() {
    let input = "\"Hello\"";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert!(output.contains("<q>Hello</q>"));
}

#[test]
fn test_markdown_to_html_multiple_quotes() {
    let input = "\"Well, well,\" Gabriella remarks, \"Welcome back\"";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert!(output.contains("<q>Well, well,</q>"));
    assert!(output.contains("<q>Welcome back</q>"));
}

#[test]
fn test_markdown_to_html_mixed_content() {
    let input = "She said \"Hello there\" and walked away.";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert!(output.contains("<q>Hello there</q>"));
    assert!(output.contains("She said"));
    assert!(output.contains("and walked away"));
}

#[test]
fn test_markdown_to_html_italic() {
    let input = "This is *italic* text.";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert!(output.contains("<em>italic</em>"));
}

#[test]
fn test_markdown_to_html_bold() {
    let input = "This is **bold** text.";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert!(output.contains("<strong>bold</strong>"));
}

#[test]
fn test_markdown_to_html_blockquote() {
    let input = "> This is a quote";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert!(output.contains("<blockquote>"));
}

#[test]
fn test_markdown_to_html_mixed_markdown() {
    let input = "**Bold** and *italic* and \"quoted\" text.";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert!(output.contains("<strong>Bold</strong>"));
    assert!(output.contains("<em>italic</em>"));
    assert!(output.contains("<q>quoted</q>"));
}

#[test]
fn test_markdown_to_html_no_quotes() {
    let input = "Plain text without quotes.";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert_eq!(output, "<p>Plain text without quotes.</p>\n");
}

#[test]
fn test_markdown_to_html_ampersand_escaped_once() {
    let output =
        crate::adapters::driving::http::utils::view_models::markdown_to_html("salt & pepper");
    assert_eq!(output, "<p>salt &amp; pepper</p>\n");
}

#[test]
fn test_markdown_to_html_xss_prevention() {
    let input = "<script>alert('xss')</script>";
    let output = crate::adapters::driving::http::utils::view_models::markdown_to_html(input);
    assert!(output.contains("&lt;script&gt;"));
    assert!(!output.contains("<script>"));
}
