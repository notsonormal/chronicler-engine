use crate::domain::model::utils::game_name::generate_game_name;

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
