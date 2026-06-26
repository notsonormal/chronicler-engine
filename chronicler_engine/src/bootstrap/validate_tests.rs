use crate::bootstrap::validate_loaded_data;
use crate::model::character::{CharacterSheet, NpcCard};
use crate::model::map::{MapDef, Overworld};
use crate::model::trigger::Trigger;
use crate::model::world::WorldCard;
use crate::test_support::{TestMap, TestNpc, TestPlayer, TestWorld};

#[test]
fn test_validate_loaded_data_success() {
    let world = WorldCard {
        key: "test".to_string(),
        name: "Test".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };
    let map = TestMap::single_room("start");
    let player = TestPlayer::standard();
    let npc = TestNpc::named("npc1", "NPC");

    let result = validate_loaded_data(&world, &map, &player, &[npc]);
    assert!(result.is_ok(), "Expected validation to pass: {result:?}");
}

#[test]
fn test_validate_loaded_data_missing_starting_room() {
    let world = WorldCard {
        key: "test".to_string(),
        name: "Test".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        scenarios: vec![crate::model::scenario::StartingScenario {
            id: "intro".to_string(),
            name: "Intro".to_string(),
            description: "".to_string(),
            text: "".to_string(),
            starting_room_id: "missing_room".to_string(),
            npcs: vec![],
        }],
        default_scenario_id: None,
        default_room_image: None,
    };

    let map = TestMap::single_room("room_a");
    let player = TestPlayer::standard();

    let result = validate_loaded_data(&world, &map, &player, &[]);
    assert!(
        result.is_err(),
        "Expected validation to fail for missing starting room"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("starting room 'missing_room' not found in map"),
        "Error should mention missing starting room: {err}"
    );
}

#[test]
fn test_validate_loaded_data_basic_manifest_succeeds() {
    let world = WorldCard {
        key: "test".to_string(),
        name: "Test".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };
    let map = TestMap::single_room("start");
    let player = TestPlayer::standard();

    let result = validate_loaded_data(&world, &map, &player, &[]);
    assert!(
        result.is_ok(),
        "Basic world with no NPCs and empty scenarios should validate successfully"
    );
}

#[test]
fn test_validate_loaded_data_invalid_trigger_room() {
    let world = TestWorld::minimal();
    let map = TestMap::single_room("room_a");
    let player = TestPlayer::standard();

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "NPC".to_string(),
            description: "An NPC".to_string(),
            personality: "Friendly".to_string(),
            scenario: "Default".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: crate::model::trigger::TriggerRequirement::TimesMet(
                crate::model::trigger::ComparisonOperator::Eq,
                0,
            ),
            narration: crate::model::trigger::TriggerNarration {
                name: "Test".to_string(),
                narration_prompt: "Hello".to_string(),
            },
            repeat: false,
            room_id: Some("nonexistent_room".to_string()),
        }],
        relationships: vec![],
    };

    let result = validate_loaded_data(&world, &map, &player, &[npc]);
    assert!(
        result.is_err(),
        "Expected validation to fail for invalid trigger room"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("non-existent room_id"),
        "Error should mention room_id: {err}"
    );
}

#[test]
fn test_validate_loaded_data_multiple_errors() {
    let world = WorldCard {
        key: "test".to_string(),
        name: "Test".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        scenarios: vec![crate::model::scenario::StartingScenario {
            id: "intro".to_string(),
            name: "Intro".to_string(),
            description: "".to_string(),
            text: "".to_string(),
            starting_room_id: "missing".to_string(),
            npcs: vec!["missing_npc".to_string()],
        }],
        default_scenario_id: None,
        default_room_image: None,
    };

    let map = MapDef {
        overworld: Overworld {
            id: "overworld".to_string(),
            name: "Overworld".to_string(),
            regions: vec![],
        },
    };

    let player = TestPlayer::standard();

    let result = validate_loaded_data(&world, &map, &player, &[]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("starting room 'missing' not found in map"),
        "Should have starting room error: {err}"
    );
}
