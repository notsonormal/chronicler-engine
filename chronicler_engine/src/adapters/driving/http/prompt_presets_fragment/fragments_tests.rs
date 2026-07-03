use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::adapters::driving::http::prompt_presets_fragment::fragments::{
    preset_card_html, preset_edit_form_html, preset_view_form_html,
};

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
    assert!(html.contains("View</button>"));
    assert!(html.contains("Duplicate</button>"));
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
    assert!(html.contains("Duplicate</button>"));
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
fn test_preset_card_html_default_and_active_preset() {
    let preset = PromptPreset {
        id: "default-active".into(),
        name: "Default Active".into(),
        instructions: Some("Default active prompt.".into()),
        is_default: true,
        ..Default::default()
    };
    let html = preset_card_html(&preset, true);
    assert!(html.contains(r#"badge">Default</span>"#));
    assert!(html.contains(r#"badge primary">Active</span>"#));
    assert!(!html.contains("Set Active</button>"));
    assert!(!html.contains("Edit</button>"));
    assert!(!html.contains("Delete</button>"));
    assert!(html.contains("View</button>"));
    assert!(html.contains("Duplicate</button>"));
    assert!(html.contains("class=\"preset-card default active\""));
}

#[test]
fn test_preset_card_html_no_instructions_uses_empty_preview() {
    let preset = PromptPreset {
        id: "no-instr".into(),
        name: "No Instructions".into(),
        ..Default::default()
    };
    let html = preset_card_html(&preset, false);
    assert!(html.contains("No Instructions"));
    assert!(html.contains("class=\"card-details preset-preview\">"));
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
fn test_preset_edit_form_html_with_all_optional_fields_none() {
    let preset = PromptPreset {
        id: "edit-empty".into(),
        name: "Empty Edit".into(),
        ..Default::default()
    };
    let html = preset_edit_form_html(&preset, "narrator", true);
    assert!(html.contains("Empty Edit"));
    assert!(html.contains(r#"name="preset_type" value="narrator""#));
    assert!(html.contains("<textarea id=\"edit-role-edit-empty\" name=\"role\" rows=\"4\"></textarea>"));
    assert!(html.contains("<textarea id=\"edit-instructions-edit-empty\" name=\"instructions\" rows=\"10\"></textarea>"));
    assert!(html.contains("<textarea id=\"edit-style-edit-empty\" name=\"writing_style\" rows=\"4\"></textarea>"));
    assert!(html.contains("<textarea id=\"edit-output-edit-empty\" name=\"output_format\" rows=\"6\"></textarea>"));
    assert!(html.contains("Save</button>"));
    assert!(html.contains("Cancel</button>"));
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

#[test]
fn test_preset_view_form_html_renders() {
    let preset = PromptPreset {
        id: "view-1".into(),
        name: "Viewable".into(),
        role: Some("Role text.".into()),
        instructions: Some("Instructions text.".into()),
        writing_style: Some("Style text.".into()),
        output_format: Some("Format text.".into()),
        is_default: true,
        preset_type: PresetType::System,
    };
    let html = preset_view_form_html(&preset);
    assert!(html.contains("view-form"));
    assert!(html.contains("View Viewable"));
    assert!(html.contains("Role text."));
    assert!(html.contains("Instructions text."));
    assert!(html.contains("Style text."));
    assert!(html.contains("Format text."));
    assert!(html.contains("disabled"));
    assert!(html.contains("readonly"));
    assert!(html.contains("Close</button>"));
    assert!(!html.contains("Save</button>"));
    assert!(html.contains(r#"hx-get="/fragment/prompt-presets/view-1""#));
}

#[test]
fn test_preset_view_form_html_with_all_optional_fields_none() {
    let preset = PromptPreset {
        id: "view-empty".into(),
        name: "Empty View".into(),
        ..Default::default()
    };
    let html = preset_view_form_html(&preset);
    assert!(html.contains("View Empty View"));
    assert!(html.contains("<textarea rows=\"4\" disabled></textarea>"));
    assert!(html.contains("<textarea rows=\"10\" disabled></textarea>"));
    assert!(html.contains("<textarea rows=\"6\" disabled></textarea>"));
    assert!(html.contains("Close</button>"));
    assert!(!html.contains("Save</button>"));
}

#[test]
fn test_preset_view_form_html_with_json_content() {
    let preset = PromptPreset {
        id: "quantifier_default".into(),
        name: "Default".into(),
        role: Some("You are a scene quantifier for a text adventure game.".into()),
        instructions: Some("Your task is to determine which NPCs are present in the current room and whether the player actually moved to a new location.\n\nHow to determine movement:\n1. Read <CurrentRoom> — this is where the player is right now.".into()),
        writing_style: None,
        output_format: Some("Respond ONLY with a JSON object in this exact format:\n{\"npcs_in_room\": [\"id1\", \"id2\"], \"movement\": {\"type\": \"entering|in|leaving\", \"destination\": \"room_id\"}}".into()),
        is_default: true,
        preset_type: PresetType::Quantifier,
    };
    let html = preset_view_form_html(&preset);
    assert!(html.contains("scene quantifier"), "role should be present");
    assert!(
        html.contains("determine which NPCs"),
        "instructions should be present"
    );
    assert!(
        html.contains("npcs_in_room"),
        "output_format with JSON braces should be present"
    );
}

#[test]
fn test_preset_view_form_html_escapes_special_chars() {
    let preset = PromptPreset {
        id: "<id>".into(),
        name: "<Name>".into(),
        instructions: Some("\"Text\"".into()),
        preset_type: PresetType::Quantifier,
        ..Default::default()
    };
    let html = preset_view_form_html(&preset);
    assert!(!html.contains("<Name>"));
    assert!(html.contains("&lt;Name&gt;"));
    assert!(html.contains("&quot;Text&quot;"));
    assert!(html.contains(r#"hx-get="/fragment/prompt-presets/&lt;id&gt;""#));
}
