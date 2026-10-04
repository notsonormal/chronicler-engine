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
