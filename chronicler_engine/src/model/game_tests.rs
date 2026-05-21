use crate::model::game::generate_game_name;

#[test]
fn test_generate_game_name_first() {
    let name = generate_game_name("Redmist", &[]);
    assert!(name.starts_with("Redmist_"));
    assert!(name.ends_with("_1"));
}

#[test]
fn test_generate_game_name_increments() {
    let existing = vec!["Redmist_2026-05-21_1".to_string()];
    let name = generate_game_name("Redmist", &existing);
    assert_eq!(name, "Redmist_2026-05-21_2");
}

#[test]
fn test_generate_game_name_max_plus_one() {
    let existing = vec![
        "Redmist_2026-05-21_1".to_string(),
        "Redmist_2026-05-21_3".to_string(),
    ];
    let name = generate_game_name("Redmist", &existing);
    assert_eq!(name, "Redmist_2026-05-21_4");
}
