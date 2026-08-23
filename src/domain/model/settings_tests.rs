//! Tests for settings and configuration types.

use std::str::FromStr;

use crate::domain::model::settings::{AppSettings, NarrativePerspective, NarrativeTense};

#[test]
fn narrative_perspective_default_is_third() {
    let settings = AppSettings::default();
    assert_eq!(settings.narrative_perspective, NarrativePerspective::Third);
}

#[test]
fn narrative_tense_default_is_past() {
    let settings = AppSettings::default();
    assert_eq!(settings.narrative_tense, NarrativeTense::Past);
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
fn app_settings_serde_round_trips_second_present() {
    let settings = AppSettings {
        narrative_perspective: NarrativePerspective::Second,
        narrative_tense: NarrativeTense::Present,
        ..Default::default()
    };

    let json = serde_json::to_string(&settings).expect("serialize");
    let restored: AppSettings = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(restored.narrative_tense, NarrativeTense::Present);
}

#[test]
fn app_settings_deserializes_old_json_without_voice_fields() {
    let json = r#"{
        "connections": [],
        "narration_connection_id": "",
        "quantifier_connection_id": "",
        "response_length": "",
        "text_check": {"mode": "Disabled", "enable_auto_check": true, "ignored_words": []},
        "agents": []
    }"#;

    let settings: AppSettings = serde_json::from_str(json).expect("deserialize old settings");
    assert_eq!(settings.narrative_perspective, NarrativePerspective::Third);
    assert_eq!(settings.narrative_tense, NarrativeTense::Past);
}
