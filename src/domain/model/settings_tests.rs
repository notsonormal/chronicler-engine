//! Tests for settings and configuration types.

use std::str::FromStr;

use crate::domain::model::settings::{
    AppSettings, ModePresetRegistry, NarrativePerspective, NarrativeTense, NarratorMode,
    TextCheckMode, TextCheckSettings,
};

#[test]
fn narrator_mode_default_is_novel() {
    assert_eq!(NarratorMode::default(), NarratorMode::Novel);
}

#[test]
fn narrator_mode_as_str_round_trips() {
    assert_eq!(NarratorMode::Novel.as_str(), "novel");
    assert_eq!(
        NarratorMode::InteractiveFiction.as_str(),
        "interactive_fiction"
    );
    assert_eq!(
        NarratorMode::from_str("novel").unwrap(),
        NarratorMode::Novel
    );
    assert_eq!(
        NarratorMode::from_str("interactive_fiction").unwrap(),
        NarratorMode::InteractiveFiction
    );
}

#[test]
fn narrator_mode_display_label_is_the_rendered_copy() {
    assert_eq!(NarratorMode::Novel.display_label(), "Novel");
    assert_eq!(
        NarratorMode::InteractiveFiction.display_label(),
        "Interactive Fiction"
    );
}

#[test]
fn narrator_mode_parse_or_default_falls_back_to_novel() {
    assert_eq!(NarratorMode::parse_or_default("bogus"), NarratorMode::Novel);
}

#[test]
fn narrative_perspective_as_str_round_trips() {
    assert_eq!(NarrativePerspective::Second.as_str(), "second");
    assert_eq!(NarrativePerspective::Third.as_str(), "third");
    assert_eq!(
        NarrativePerspective::from_str("second").unwrap(),
        NarrativePerspective::Second
    );
    assert_eq!(
        NarrativePerspective::from_str("third").unwrap(),
        NarrativePerspective::Third
    );
}

#[test]
fn narrative_tense_as_str_round_trips() {
    assert_eq!(NarrativeTense::Past.as_str(), "past");
    assert_eq!(NarrativeTense::Present.as_str(), "present");
    assert_eq!(
        NarrativeTense::from_str("past").unwrap(),
        NarrativeTense::Past
    );
    assert_eq!(
        NarrativeTense::from_str("present").unwrap(),
        NarrativeTense::Present
    );
}

#[test]
fn narrative_perspective_from_str_unknown_errors() {
    assert!(NarrativePerspective::from_str("first").is_err());
}

#[test]
fn narrative_tense_from_str_unknown_errors() {
    assert!(NarrativeTense::from_str("future").is_err());
}

#[test]
fn mode_preset_registry_default_points_each_mode_at_its_system_preset() {
    let settings = AppSettings::default();
    let novel = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    let if_mode = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::InteractiveFiction);
    assert_eq!(novel.mode, NarratorMode::Novel);
    assert_eq!(if_mode.mode, NarratorMode::InteractiveFiction);
    assert_eq!(novel.system_prompt_preset_id, "system_default");
    assert_eq!(if_mode.system_prompt_preset_id, "system_if_default");
    // Quantifier/impersonate are shared across modes by default.
    assert_eq!(
        novel.quantifier_prompt_preset_id,
        if_mode.quantifier_prompt_preset_id
    );
    assert_eq!(
        novel.impersonate_prompt_preset_id,
        if_mode.impersonate_prompt_preset_id
    );
}

#[test]
fn mode_preset_registry_list_serde_round_trips() {
    let settings = AppSettings::default();

    let json = serde_json::to_string(&settings.mode_preset_registry).expect("serialize");
    assert!(
        json.starts_with("["),
        "registry serializes as a list: {json}"
    );
    let restored: crate::domain::model::settings::ModePresetRegistry =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored, settings.mode_preset_registry);
}

#[test]
fn bundle_for_falls_back_to_constructed_defaults() {
    // Empty registry: every mode gets the constructed default bundle.
    let empty = crate::domain::model::settings::ModePresetRegistry::default();
    let registry = crate::domain::model::settings::ModePresetRegistry(Vec::new());
    for mode in [NarratorMode::Novel, NarratorMode::InteractiveFiction] {
        assert_eq!(registry.bundle_for(mode), empty.bundle_for(mode));
    }

    // Partial registry: the missing entry falls back, the present one wins.
    let mut registry = crate::domain::model::settings::ModePresetRegistry(Vec::new());
    let mut novel = registry.bundle_for(NarratorMode::Novel);
    novel.system_prompt_preset_id = "custom_novel".to_string();
    registry.set_bundle(novel);

    assert_eq!(
        registry
            .bundle_for(NarratorMode::Novel)
            .system_prompt_preset_id,
        "custom_novel"
    );
    assert_eq!(
        registry
            .bundle_for(NarratorMode::InteractiveFiction)
            .system_prompt_preset_id,
        "system_if_default"
    );
}

#[test]
fn set_bundle_replaces_matching_mode_and_pushes_missing_mode() {
    use crate::domain::model::settings::ModePresetRegistry;

    let mut registry = ModePresetRegistry::default();
    let mut novel = registry.bundle_for(NarratorMode::Novel);
    novel.system_prompt_preset_id = "custom_novel".to_string();
    registry.set_bundle(novel);
    assert_eq!(registry.0.len(), 2, "replace keeps one entry per mode");

    registry.0.clear();
    let mut if_bundle = registry.bundle_for(NarratorMode::InteractiveFiction);
    if_bundle.system_prompt_preset_id = "custom_if".to_string();
    registry.set_bundle(if_bundle);
    assert_eq!(registry.0.len(), 1, "push adds the missing mode");
    assert_eq!(
        registry
            .bundle_for(NarratorMode::Novel)
            .system_prompt_preset_id,
        "system_default"
    );
}

#[test]
fn app_settings_serde_round_trips_registry() {
    let mut settings = AppSettings::default();
    let mut novel = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    novel.system_prompt_preset_id = "custom_novel".to_string();
    settings.mode_preset_registry.set_bundle(novel);
    let mut if_mode = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::InteractiveFiction);
    if_mode.system_prompt_preset_id = "custom_if".to_string();
    settings.mode_preset_registry.set_bundle(if_mode);

    let json = serde_json::to_string(&settings).expect("serialize");
    let restored: AppSettings = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        restored
            .mode_preset_registry
            .bundle_for(NarratorMode::Novel)
            .system_prompt_preset_id,
        "custom_novel"
    );
    assert_eq!(
        restored
            .mode_preset_registry
            .bundle_for(NarratorMode::InteractiveFiction)
            .system_prompt_preset_id,
        "custom_if"
    );
}

#[test]
fn app_settings_deserializes_old_json_without_registry() {
    let json = r#"{
        "connections": [],
        "narration_connection_id": "",
        "quantifier_connection_id": "",
        "response_length": "",
        "text_check": {"mode": "Disabled", "enable_auto_check": true, "ignored_words": []},
        "agents": []
    }"#;

    let settings: AppSettings = serde_json::from_str(json).expect("deserialize old settings");
    assert_eq!(
        settings
            .mode_preset_registry
            .bundle_for(NarratorMode::Novel)
            .system_prompt_preset_id,
        "system_default"
    );
}

#[test]
fn narration_connection_resolves_a_listed_id() {
    let settings = AppSettings::default();
    let conn = settings
        .narration_connection()
        .expect("default settings list their own narrator id");
    assert_eq!(conn.id, settings.narration_connection_id);
}

#[test]
fn narration_connection_errors_on_a_dangling_id() {
    let settings = AppSettings {
        narration_connection_id: "does-not-exist".into(),
        ..AppSettings::default()
    };

    let err = settings
        .narration_connection()
        .expect_err("a dangling id is a configuration fault");
    let msg = err.to_string();
    assert!(
        msg.contains("does-not-exist"),
        "error should name the missing id, got: {msg}"
    );
}

#[test]
fn quantifier_connection_resolves_a_listed_id() {
    let settings = AppSettings::default();
    let conn = settings
        .quantifier_connection()
        .expect("default settings list their own quantifier id");
    assert_eq!(conn.id, settings.quantifier_connection_id);
}

#[test]
fn quantifier_connection_errors_on_a_dangling_id() {
    let settings = AppSettings {
        quantifier_connection_id: "also-missing".into(),
        ..AppSettings::default()
    };

    let err = settings
        .quantifier_connection()
        .expect_err("a dangling id is a configuration fault");
    assert!(err.to_string().contains("also-missing"));
}

#[test]
fn text_check_effective_auto_check_drops_disabled_mode() {
    let settings = TextCheckSettings {
        mode: TextCheckMode::Disabled,
        enable_auto_check: true,
        ignored_words: Vec::new(),
    };
    assert!(!settings.effective_enable_auto_check());
}

#[test]
fn text_check_effective_auto_check_keeps_an_enabled_mode() {
    let settings = TextCheckSettings {
        mode: TextCheckMode::Spell,
        enable_auto_check: true,
        ignored_words: Vec::new(),
    };
    assert!(settings.effective_enable_auto_check());
}

#[test]
fn text_check_set_mode_and_auto_check_normalises_the_pair() {
    let mut settings = TextCheckSettings::default();

    settings.set_mode_and_auto_check(TextCheckMode::Disabled, true);
    assert!(!settings.enable_auto_check);

    settings.set_mode_and_auto_check(TextCheckMode::Grammar, true);
    assert!(settings.enable_auto_check);
}

#[test]
fn active_preset_id_reads_the_slot_for_its_type_and_mode() {
    use crate::domain::model::prompt_preset::PresetType;

    let mut settings = AppSettings::default();
    settings.set_active_preset(PresetType::System, NarratorMode::Novel, "sys_n".into());
    settings.set_active_preset(
        PresetType::System,
        NarratorMode::InteractiveFiction,
        "sys_if".into(),
    );
    settings.set_active_preset(PresetType::Quantifier, NarratorMode::Novel, "quant".into());
    settings.set_active_preset(
        PresetType::Impersonate,
        NarratorMode::InteractiveFiction,
        "imp_if".into(),
    );

    assert_eq!(
        settings.active_preset_id(PresetType::System, NarratorMode::Novel),
        "sys_n"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::System, NarratorMode::InteractiveFiction),
        "sys_if"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::Quantifier, NarratorMode::Novel),
        "quant"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::Impersonate, NarratorMode::InteractiveFiction),
        "imp_if"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::Quantifier, NarratorMode::InteractiveFiction),
        "quantifier_default",
        "an untouched slot keeps its per-mode default"
    );
}

#[test]
fn set_active_preset_touches_only_its_own_slot() {
    use crate::domain::model::prompt_preset::PresetType;

    let mut settings = AppSettings::default();
    settings.set_active_preset(
        PresetType::Quantifier,
        NarratorMode::InteractiveFiction,
        "new_q".into(),
    );

    assert_eq!(
        settings.active_preset_id(PresetType::Quantifier, NarratorMode::InteractiveFiction),
        "new_q"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::Quantifier, NarratorMode::Novel),
        "quantifier_default"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::System, NarratorMode::Novel),
        "system_default"
    );
}

#[test]
fn options_slot_ignores_the_narrator_mode() {
    use crate::domain::model::prompt_preset::PresetType;

    let mut settings = AppSettings::default();
    settings.set_active_preset(PresetType::Options, NarratorMode::Novel, "opt".into());

    assert_eq!(
        settings.active_preset_id(PresetType::Options, NarratorMode::Novel),
        "opt"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::Options, NarratorMode::InteractiveFiction),
        "opt",
        "an Options preset serves both modes from the one settings slot"
    );
    assert_eq!(settings.active_options_prompt_preset_id, "opt");
}

#[test]
fn active_preset_id_falls_back_when_the_mode_has_no_bundle() {
    use crate::domain::model::prompt_preset::PresetType;

    let settings = AppSettings {
        mode_preset_registry: ModePresetRegistry(Vec::new()),
        active_options_prompt_preset_id: "opt_custom".into(),
        ..AppSettings::default()
    };

    assert_eq!(
        settings.active_preset_id(PresetType::System, NarratorMode::InteractiveFiction),
        "system_if_default"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::Impersonate, NarratorMode::Novel),
        "impersonate_default"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::Options, NarratorMode::Novel),
        "opt_custom",
        "the Options slot never consults the mode registry"
    );
}

#[test]
fn set_active_preset_adds_a_bundle_for_an_absent_mode() {
    use crate::domain::model::prompt_preset::PresetType;

    let mut settings = AppSettings::default();
    settings.mode_preset_registry.0.clear();
    settings.set_active_preset(PresetType::System, NarratorMode::Novel, "sys_n".into());

    assert_eq!(settings.mode_preset_registry.0.len(), 1);
    assert_eq!(
        settings.active_preset_id(PresetType::System, NarratorMode::Novel),
        "sys_n"
    );
    assert_eq!(
        settings.active_preset_id(PresetType::System, NarratorMode::InteractiveFiction),
        "system_if_default",
        "the other mode still falls back to its own default"
    );
}
