use crate::model::scenario::StartingScenario;

#[test]
fn test_starting_scenario_serde() {
    let json = r#"{
        "id": "redmist_intro",
        "name": "Redmist Estate",
        "description": "A mysterious Victorian mansion",
        "starting_room_id": "entrance_hall",
        "text": "You arrive at the estate..."
    }"#;
    let scenario: StartingScenario = serde_json::from_str(json).unwrap();
    assert_eq!(scenario.id, "redmist_intro");
    assert_eq!(scenario.starting_room_id, "entrance_hall");
}

#[test]
fn test_starting_scenario_optional_text() {
    let json = r#"{
        "id": "simple_scenario",
        "name": "Simple",
        "description": "A test scenario",
        "starting_room_id": "room_1"
    }"#;
    let scenario: StartingScenario = serde_json::from_str(json).unwrap();
    assert_eq!(scenario.text, "");
}

#[test]
fn test_starting_scenario_missing_starting_room_id_defaults_to_start() {
    let json = r#"{
        "id": "no_room",
        "name": "No Room",
        "description": "Missing starting_room_id",
        "text": ""
    }"#;
    let scenario: StartingScenario = serde_json::from_str(json).unwrap();
    assert_eq!(scenario.starting_room_id, "start");
}
