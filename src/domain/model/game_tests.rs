use crate::domain::model::game::{Game, NewGame, PresetSelection};
use crate::domain::model::prompt_preset::PresetType;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use crate::domain::model::utils::game_name::{default_display_name, generate_game_name};

#[test]
fn test_generate_game_name_first() {
    let name = generate_game_name("Redmist", &[]);
    assert!(name.starts_with("Redmist_"));
    assert!(name.ends_with("_1"));
}

#[test]
fn test_generate_game_name_increments() {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let existing = vec![format!("Redmist_{today}_1")];
    let name = generate_game_name("Redmist", &existing);
    assert_eq!(name, format!("Redmist_{today}_2"));
}

#[test]
fn test_generate_game_name_max_plus_one() {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let existing = vec![format!("Redmist_{today}_1"), format!("Redmist_{today}_3")];
    let name = generate_game_name("Redmist", &existing);
    assert_eq!(name, format!("Redmist_{today}_4"));
}

#[test]
fn test_default_display_name_formats_world_date_and_ordinal() {
    assert_eq!(
        default_display_name("Redmist Estate_2026-09-29_1"),
        "Redmist Estate — 29 Sep 2026 (1)"
    );
}

#[test]
fn test_default_display_name_preserves_underscores_in_world_name() {
    assert_eq!(
        default_display_name("Some_World_2026-01-05_3"),
        "Some_World — 05 Jan 2026 (3)"
    );
}

#[test]
fn test_default_display_name_passes_through_a_plain_name() {
    assert_eq!(default_display_name("Hand Written"), "Hand Written");
}

#[test]
fn test_default_display_name_passes_through_a_bad_date() {
    assert_eq!(
        default_display_name("World_not-a-date_1"),
        "World_not-a-date_1"
    );
}

fn new_game(name: &str) -> NewGame {
    NewGame {
        world_name: "Test World".to_string(),
        world_key: "test".to_string(),
        persona_key: "test_player".to_string(),
        persona_name: "Test Player".to_string(),
        name: name.to_string(),
        narrator_mode: NarratorMode::Novel,
        narrative_perspective: NarrativePerspective::Third,
        narrative_tense: NarrativeTense::Past,
        system_prompt_preset_id: "system_default".to_string(),
        quantifier_prompt_preset_id: "quantifier_default".to_string(),
        impersonate_prompt_preset_id: "impersonate_default".to_string(),
        options_prompt_preset_id: "options_default".to_string(),
        options_always_on: false,
    }
}

#[test]
fn test_new_game_display_name_renders_the_stable_name() {
    assert_eq!(
        new_game("Redmist Estate_2026-09-29_1").display_name(),
        "Redmist Estate — 29 Sep 2026 (1)"
    );
}

#[test]
fn test_new_game_display_name_passes_a_plain_name_through() {
    assert_eq!(new_game("Hand Written").display_name(), "Hand Written");
}

#[test]
fn test_active_preset_id_round_trips_every_preset_type() {
    let mut game = Game::default();
    let types = [
        PresetType::System,
        PresetType::Quantifier,
        PresetType::Impersonate,
        PresetType::Options,
    ];

    for preset_type in types {
        game.set_active_preset_id(preset_type, format!("{}_slot", preset_type.as_str()));
    }

    assert_eq!(game.active_system_prompt_preset_id, "system_slot");
    assert_eq!(game.active_quantifier_prompt_preset_id, "quantifier_slot");
    assert_eq!(game.active_impersonate_prompt_preset_id, "impersonate_slot");
    assert_eq!(game.active_options_prompt_preset_id, "options_slot");
    for preset_type in types {
        assert_eq!(
            game.active_preset_id(preset_type),
            format!("{}_slot", preset_type.as_str())
        );
    }
}

#[test]
fn test_preset_selection_slots_pairs_every_field_with_its_type() {
    let selection = PresetSelection::new("sys", "quant", "imp", "opt");

    assert_eq!(
        selection.slots(),
        [
            (PresetType::System, "sys"),
            (PresetType::Quantifier, "quant"),
            (PresetType::Impersonate, "imp"),
            (PresetType::Options, "opt"),
        ]
    );
}
