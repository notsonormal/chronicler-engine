use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartingScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub starting_room_id: String,
    #[serde(default)]
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
