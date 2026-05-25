use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::server::prompt_presets_fragment::fragments::{preset_card_html, preset_edit_form_html};

#[test]
fn test_preset_card_html_default_preset() {
    let preset = PromptPreset {
        id: "default".into(),
        name: "Default".into(),
        instructions: Some("System prompt.".into()),
        is_default: true,
        preset_type: PresetType::System,
        ..Default::default()
    };
    let html = preset_card_html(&preset, false);
    assert!(html.contains("Default"));
    assert!(html.contains(r#"badge">Default</span>"#));
    assert!(!html.contains("Edit</button>"));
    assert!(!html.contains("Delete</button>"));
}

#[test]
fn test_preset_card_html_non_default_preset() {
    let preset = PromptPreset {
        id: "custom-1".into(),
        name: "Custom".into(),
        instructions: Some("Custom prompt.".into()),
        ..Default::default()
    };
    let html = preset_card_html(&preset, false);
    assert!(html.contains("Custom"));
    assert!(!html.contains(r#"badge">Default</span>"#));
    assert!(html.contains("Edit</button>"));
    assert!(html.contains("Delete</button>"));
}

#[test]
fn test_preset_card_html_active_preset() {
    let preset = PromptPreset {
        id: "active-1".into(),
        name: "Active".into(),
        instructions: Some("Active prompt.".into()),
        ..Default::default()
    };
    let html = preset_card_html(&preset, true);
    assert!(html.contains(r#"badge primary">Active</span>"#));
    assert!(!html.contains("Set Active</button>"));
}

#[test]
fn test_preset_card_html_inactive_preset() {
    let preset = PromptPreset {
        id: "inactive-1".into(),
        name: "Inactive".into(),
        instructions: Some("Inactive prompt.".into()),
        ..Default::default()
    };
    let html = preset_card_html(&preset, false);
    assert!(!html.contains("Active</span>"));
    assert!(html.contains("Set Active</button>"));
}

#[test]
fn test_preset_card_html_preview_truncates() {
    let long_text = "a".repeat(200);
    let preset = PromptPreset {
        id: "test".into(),
        name: "Test".into(),
        instructions: Some(long_text.clone()),
        ..Default::default()
    };
    let html = preset_card_html(&preset, false);
    assert!(html.contains(&"a".repeat(120)));
    assert!(!html.contains(&"a".repeat(121)));
}

#[test]
fn test_preset_card_html_escapes_special_chars() {
    let preset = PromptPreset {
        id: "<script>".into(),
        name: "<b>Name</b>".into(),
        instructions: Some(r#"Say "hello" & goodbye."#.into()),
        ..Default::default()
    };
    let html = preset_card_html(&preset, false);
    assert!(!html.contains("<b>Name</b>"));
    assert!(html.contains("&lt;b&gt;Name&lt;/b&gt;"));
    assert!(html.contains("&quot;hello&quot;"));
    assert!(html.contains("&amp;"));
}

#[test]
fn test_preset_edit_form_html_renders() {
    let preset = PromptPreset {
        id: "edit-1".into(),
        name: "Editable".into(),
        instructions: Some("Edit me.".into()),
        ..Default::default()
    };
    let html = preset_edit_form_html(&preset, "system", false);
    assert!(html.contains("edit-form"));
    assert!(html.contains("Editable"));
    assert!(html.contains("edit-1"));
    assert!(html.contains(r#"hx-post="/prompt-presets/edit-1""#));
    assert!(html.contains(r#"name="preset_type" value="system""#));
}

#[test]
fn test_preset_edit_form_html_escapes_special_chars() {
    let preset = PromptPreset {
        id: "<id>".into(),
        name: "<Name>".into(),
        instructions: Some("\"Text\"".into()),
        preset_type: PresetType::Quantifier,
        ..Default::default()
    };
    let html = preset_edit_form_html(&preset, "quantifier", true);
    assert!(!html.contains("<Name>"));
    assert!(html.contains("&lt;Name&gt;"));
    assert!(html.contains("&quot;Text&quot;"));
    assert!(html.contains(r#"hx-post="/prompt-presets/&lt;id&gt;""#));
}
