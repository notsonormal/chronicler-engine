use crate::model::character::NpcCard;
use crate::model::map::Room;
use crate::model::state::LogEntry;
use crate::narrative::quantifier::backends::{MockQuantifierBackend, QuantifierBackendTrait};
use crate::narrative::quantifier::core::{action_boundary_contains, quantify_room_with_llm_call};
use crate::narrative::quantifier::parser::{
    compute_npc_events, extract_movement_from_text, parse_quantifier_response,
    parse_quantifier_response_with_movement,
};
use crate::narrative::quantifier::prompt::QuantifierPromptBuilder;
use crate::narrative::quantifier::types::{
    MovementParseResult, MovementType, NpcEventType, QuantifierConfidence, QuantifierPromptContext,
    RoomInfo,
};

use crate::model::character::CharacterSheet;
use crate::model::map::Direction;
use crate::model::state::LogType;
use chrono::Utc;
use std::collections::HashMap;

fn make_room() -> Room {
    Room {
        id: "entrance_hall".to_string(),
        name: "Entrance Hall".to_string(),
        description: "A grand entrance hall with marble floors.".to_string(),
        exits: HashMap::from([(Direction::North, "library".to_string())]),
        items: vec![],
        npcs: vec!["gabriella".to_string()],
        image_path: None,
        navigation_description: None,
    }
}

fn make_npc(id: &str, name: &str) -> NpcCard {
    NpcCard {
        id: id.to_string(),
        sheet: CharacterSheet {
            name: name.to_string(),
            description: format!("A character named {name}."),
            personality: "Mysterious".to_string(),
            scenario: "Investigating".to_string(),
            example_dialogue: String::new(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
    }
}

fn make_history() -> Vec<LogEntry> {
    vec![
        LogEntry {
            id: 1,
            sender: Some("Narrator".to_string()),
            text: "You enter the front gate.".to_string(),
            log_type: LogType::Narration,
            timestamp: Utc::now(),
        },
        LogEntry {
            id: 2,
            sender: Some("Carla".to_string()),
            text: "I'll follow you inside.".to_string(),
            log_type: LogType::Dialogue,
            timestamp: Utc::now(),
        },
    ]
}

fn make_boundary_chars() -> std::collections::HashSet<char> {
    [
        ' ', '.', ',', '!', '?', '\n', '\t', '\r', '\'', '"', ':', ';',
    ]
    .into_iter()
    .collect()
}

#[test]
fn test_quantifier_prompt_builder_basic() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let gabriella = make_npc("gabriella", "Gabriella");
    let all_npcs = vec![carla.clone(), gabriella.clone()];
    let previous_npcs = vec![carla];
    let history = make_history();

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I walk into the entrance hall.",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (system, user) = builder.build();

    // System prompt should contain instructions and known NPC IDs
    assert!(system.contains("scene quantifier"));
    assert!(system.contains("npcs_in_room"));
    assert!(system.contains("carla"));
    assert!(system.contains("gabriella"));

    // Decision framework should be present
    assert!(system.contains("How to determine movement"));
    assert!(system.contains("Read <CurrentRoom>"));
    assert!(system.contains("Read <LatestNarration>"));

    // User prompt should contain room info and previous NPCs
    assert!(user.contains("Entrance Hall"));
    assert!(user.contains("Carla"));
    assert!(user.contains("Hero"));

    // Plain-text format: old XML wrappers must not be present
    assert!(!system.contains("<QuantifierTask>"));
    assert!(!system.contains("<AuxiliaryInstructions>"));
    assert!(!user.contains("<Query>"));
}

#[test]
fn test_quantifier_prompt_builder_token_budget() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let gabriella = make_npc("gabriella", "Gabriella");
    let all_npcs = vec![carla, gabriella];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history = make_history();

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (system, user) = builder.build();

    // Total prompt should be under ~4000 chars (roughly 1000 tokens)
    let total_chars = system.len() + user.len();
    assert!(
        total_chars < 4000,
        "Quantifier prompt too long: {total_chars} chars"
    );
}

#[test]
fn test_quantifier_prompt_builder_empty_history() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (_, user) = builder.build();

    // Should not crash with empty history
    assert!(user.contains("Hero"));
    assert!(user.contains("I look around"));
}

#[test]
fn test_parse_json_response() {
    let response = r#"{"npcs_in_room": ["carla", "gabriella"]}"#;
    let known_ids = vec![
        "carla".to_string(),
        "gabriella".to_string(),
        "guard".to_string(),
    ];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_json_with_markdown_code_fence() {
    let response = "```json\n{\"npcs_in_room\": [\"carla\"]}\n```";
    let known_ids = vec!["carla".to_string(), "gabriella".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla"]);
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_json_embedded_in_text() {
    let response = "Based on the context, the NPCs present are:\n{\"npcs_in_room\": [\"carla\"]}\nThat's who I see.";
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla"]);
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_text_fallback() {
    let response = "Both Carla and Gabriella are in the room.";
    let known_ids = vec![
        "carla".to_string(),
        "gabriella".to_string(),
        "guard".to_string(),
    ];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_parse_filters_unknown_npcs() {
    let response = r#"{"npcs_in_room": ["carla", "harry", "gabriella"]}"#;
    let known_ids = vec!["carla".to_string(), "gabriella".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    // "harry" should be filtered out since it's not in known_ids
    assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_empty_response() {
    let response = r#"{"npcs_in_room": []}"#;
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert!(result.npc_ids.is_empty());
    // Empty valid JSON is still High confidence (the LLM correctly said no one is there)
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_completely_invalid_response() {
    let response = "I don't understand the question.";
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert!(result.npc_ids.is_empty());
    assert_eq!(result.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_parse_text_fallback_case_insensitive() {
    let response = "Carla follows the player into the hall.";
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla"]);
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_parse_json_empty_array() {
    let response = r#"{"npcs_in_room": []}"#;
    let known_ids: Vec<String> = vec![];

    let result = parse_quantifier_response(response, &known_ids);
    assert!(result.npc_ids.is_empty());
}

#[test]
fn test_quantifier_confidence_levels() {
    // High: JSON parsed, valid IDs
    let high_result =
        parse_quantifier_response(r#"{"npcs_in_room": ["carla"]}"#, &["carla".to_string()]);
    assert_eq!(high_result.confidence, QuantifierConfidence::High);

    // Medium: Text fallback
    let medium_result = parse_quantifier_response("Carla is here.", &["carla".to_string()]);
    assert_eq!(medium_result.confidence, QuantifierConfidence::Medium);

    // Low: No valid IDs
    let low_result = parse_quantifier_response("Random text.", &["carla".to_string()]);
    assert_eq!(low_result.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_parse_quantifier_response_no_movement() {
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.destination, None);
}

#[test]
fn test_parse_movement_entering_with_destination() {
    // The actual LLM response format we're using
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "entering", "destination": "entrance_hall"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string()],
        &[RoomInfo {
            id: "entrance_hall".to_string(),
            name: "Entrance Hall".to_string(),
        }],
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(
        result.movement.destination,
        Some("entrance_hall".to_string())
    );
}

#[test]
fn test_parse_movement_leaving_with_destination() {
    let response =
        r#"{"npcs_in_room": ["carla"], "movement": {"type": "leaving", "destination": "tavern"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string()],
        &[RoomInfo {
            id: "tavern".to_string(),
            name: "Tavern".to_string(),
        }],
    );

    assert_eq!(result.movement.movement_type, Some(MovementType::Leaving));
    assert_eq!(result.movement.destination, Some("tavern".to_string()));
}

#[test]
fn test_parse_movement_no_movement_field() {
    // No movement field at all - should detect no movement
    let response = r#"{"npcs_in_room": ["carla"]}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.destination, None);
}

#[test]
fn test_parse_movement_empty_movement_object() {
    // Empty movement object - should detect no movement
    let response = r#"{"npcs_in_room": ["carla"], "movement": {}}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.destination, None);
}

#[test]
fn test_parse_movement_unknown_type_becomes_none() {
    // Unknown movement type should map to None (not panic)
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "teleporting", "destination": "castle"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string()],
        &[RoomInfo {
            id: "castle".to_string(),
            name: "Castle".to_string(),
        }],
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    // Unknown type maps to None
    assert_eq!(result.movement.movement_type, None);
    // But destination should still be captured
    assert_eq!(result.movement.destination, Some("castle".to_string()));
}

#[test]
fn test_parse_movement_case_insensitive_type() {
    // Type should be case-insensitive
    let response =
        r#"{"npcs_in_room": ["carla"], "movement": {"type": "ENTERING", "destination": "hall"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string()],
        &[RoomInfo {
            id: "hall".to_string(),
            name: "Hall".to_string(),
        }],
    );

    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
}

#[test]
fn test_quantifier_prompt_includes_navigation() {
    let room = Room {
        id: "test_room".to_string(),
        name: "Test Room".to_string(),
        description: "A test room.".to_string(),
        exits: HashMap::new(),
        items: vec![],
        npcs: vec![],
        image_path: None,
        navigation_description: Some("You can go north to the kitchen.".to_string()),
    };

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &[],
        all_rooms: &[RoomInfo {
            id: "kitchen".to_string(),
            name: "Kitchen".to_string(),
        }],
        player_name: "Player",
        recent_history: &[],
        player_action: "I walk to the kitchen",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (_, user_prompt) = builder.build();

    assert!(user_prompt.contains("<Navigation>"));
    assert!(user_prompt.contains("You can go north to the kitchen"));
}

#[test]
fn test_compute_npc_events_empty_previous() {
    // First encounter - Carla enters
    let previous: Vec<String> = vec![];
    let current = vec!["carla".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].npc_id, "carla");
    assert_eq!(result.events[0].event_type, NpcEventType::Entered);
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_compute_npc_events_no_changes() {
    // Carla was there and still is there - no events
    let previous = vec!["carla".to_string()];
    let current = vec!["carla".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert!(result.events.is_empty());
    assert_eq!(result.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_compute_npc_events_npc_left() {
    // Carla was there, now she's gone
    let previous = vec!["carla".to_string()];
    let current: Vec<String> = vec![];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].npc_id, "carla");
    assert_eq!(result.events[0].event_type, NpcEventType::Left);
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_compute_npc_events_mixed() {
    // Carla stays, Gabriella enters, Derek leaves
    let previous = vec!["carla".to_string(), "derek".to_string()];
    let current = vec!["carla".to_string(), "gabriella".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 2);

    // Should have Entered for Gabriella
    let entered = result
        .events
        .iter()
        .find(|e| e.npc_id == "gabriella")
        .expect("Gabriella should be in events");
    assert_eq!(entered.event_type, NpcEventType::Entered);

    // Should have Left for Derek
    let left = result
        .events
        .iter()
        .find(|e| e.npc_id == "derek")
        .expect("Derek should be in events");
    assert_eq!(left.event_type, NpcEventType::Left);

    // Carla should NOT be in events (she stayed)
    assert!(!result.events.iter().any(|e| e.npc_id == "carla"));
}

#[test]
fn test_compute_npc_events_multiple_entered() {
    // Carla and Gabriella both enter at once
    let previous: Vec<String> = vec![];
    let current = vec!["carla".to_string(), "gabriella".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 2);
    assert!(
        result
            .events
            .iter()
            .all(|e| e.event_type == NpcEventType::Entered)
    );
    assert!(result.events.iter().any(|e| e.npc_id == "carla"));
    assert!(result.events.iter().any(|e| e.npc_id == "gabriella"));
}

#[test]
fn test_parse_quantifier_response_with_movement_only() {
    // Verify NPCs and movement are parsed correctly (npc_events computed separately in fragments.rs)
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
}

#[test]
fn test_extract_movement_from_text_entering() {
    let rooms = vec![RoomInfo {
        id: "kitchen".to_string(),
        name: "Kitchen".to_string(),
    }];
    let result = extract_movement_from_text("I enter the kitchen", &rooms);
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.movement_type, Some(MovementType::Entering));
}

#[test]
fn test_extract_movement_from_text_leaving() {
    let rooms = vec![RoomInfo {
        id: "hall".to_string(),
        name: "Hall".to_string(),
    }];
    let result = extract_movement_from_text("I leave the hall", &rooms);
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_from_text_walk_into() {
    let rooms = vec![RoomInfo {
        id: "garden".to_string(),
        name: "Garden".to_string(),
    }];
    let result = extract_movement_from_text("I walk into the garden", &rooms);
    assert!(result.is_some());
}

#[test]
fn test_extract_movement_from_text_head_to() {
    let rooms = vec![RoomInfo {
        id: "tower".to_string(),
        name: "Tower".to_string(),
    }];
    let result = extract_movement_from_text("I head to the tower", &rooms);
    assert!(result.is_some());
}

#[test]
fn test_extract_movement_destination_not_found() {
    let rooms = vec![RoomInfo {
        id: "kitchen".to_string(),
        name: "Kitchen".to_string(),
    }];
    // Room name doesn't appear in text
    let result = extract_movement_from_text("I go somewhere else", &rooms);
    // Should not have destination
    assert!(result.map(|r| r.destination.is_none()).unwrap_or(true));
}

#[test]
fn test_extract_movement_from_text_empty() {
    let rooms = vec![];
    let result = extract_movement_from_text("", &rooms);
    assert!(result.is_none());
}

#[test]
fn test_extract_movement_case_insensitive() {
    let rooms = vec![RoomInfo {
        id: "FOREST".to_string(),
        name: "Forest".to_string(),
    }];
    let result = extract_movement_from_text("I ENTER the FOREST", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Entering));
}

#[test]
fn test_extract_movement_go_out() {
    let rooms = vec![RoomInfo {
        id: "outside".to_string(),
        name: "Outside".to_string(),
    }];
    let result = extract_movement_from_text("I go out", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_walk_out() {
    let rooms = vec![RoomInfo {
        id: "garden".to_string(),
        name: "Garden".to_string(),
    }];
    let result = extract_movement_from_text("I walk out to the garden", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_head_out() {
    let rooms = vec![RoomInfo {
        id: "courtyard".to_string(),
        name: "Courtyard".to_string(),
    }];
    let result = extract_movement_from_text("I head out to the courtyard", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_exits() {
    let rooms = vec![RoomInfo {
        id: "hallway".to_string(),
        name: "Hallway".to_string(),
    }];
    let result = extract_movement_from_text("I exit to the hallway", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_travel_to() {
    let rooms = vec![RoomInfo {
        id: "village".to_string(),
        name: "Village".to_string(),
    }];
    let result = extract_movement_from_text("I travel to the village", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Entering));
}

#[test]
fn test_extract_movement_go_into() {
    let rooms = vec![RoomInfo {
        id: "cave".to_string(),
        name: "Cave".to_string(),
    }];
    let result = extract_movement_from_text("I go into the cave", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Entering));
}

#[test]
fn test_extract_movement_destination_found() {
    let rooms = vec![RoomInfo {
        id: "kitchen".to_string(),
        name: "Kitchen".to_string(),
    }];
    let result = extract_movement_from_text("I walk to the kitchen", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().destination, Some("Kitchen".to_string()));
}

#[test]
fn test_quantifier_prompt_builder_empty_npcs() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (system, user) = builder.build();

    // Should handle empty NPCs gracefully
    assert!(system.contains("AvailableNpcIds"));
    assert!(user.contains("Hero"));
}

#[test]
fn test_quantifier_prompt_builder_with_room_npcs() {
    let mut room = make_room();
    room.npcs = vec!["gabriella".to_string(), "carla".to_string()];

    let gabriella = make_npc("gabriella", "Gabriella");
    let all_npcs = vec![gabriella];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I enter the room.",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (_, user) = builder.build();

    // Should include room configured NPCs
    assert!(user.contains("RoomConfiguredNpcs"));
    assert!(user.contains("gabriella"));
    assert!(user.contains("carla"));
}

#[test]
fn test_parse_movement_text_fallback_no_movement() {
    // When JSON parsing fails, movement should be Low confidence (no text fallback)
    // But NPC extraction should still work via text matching
    let response = "carla is standing in the room.";
    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Medium);
    // Movement is only from JSON, so it should be None/Low
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_parse_movement_json_invalid_npcs_filtered() {
    // Unknown NPCs in JSON should be filtered out
    let response = r#"{"npcs_in_room": ["carla", "unknown_npc", "gabriella"], "movement": {"type": "entering"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string(), "gabriella".to_string()],
        &[],
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
}

#[test]
fn test_mock_quantifier_backend_auto_detect() {
    let carla = make_npc("carla", "Carla");
    let gabriella = make_npc("gabriella", "Gabriella");
    let all_npcs = vec![carla, gabriella];

    let backend = MockQuantifierBackend::default();

    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "I talk to Carla about the quest.",
    };

    let result = backend.quantify_room(&context, &[]).unwrap();

    // Should auto-detect "Carla" from player action
    assert!(result.npcs.npc_ids.contains(&"carla".to_string()));
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
}

#[test]
fn test_mock_quantifier_backend_explicit_npcs() {
    let all_npcs = vec![make_npc("carla", "Carla")];

    let backend = MockQuantifierBackend {
        npcs_to_return: vec!["carla".to_string()],
        movement_to_return: Some(MovementParseResult {
            movement_type: Some(MovementType::Entering),
            destination: Some("entrance".to_string()),
            confidence: QuantifierConfidence::High,
        }),
    };

    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "I walk in.",
    };

    let result = backend.quantify_room(&context, &[]).unwrap();

    assert_eq!(result.npcs.npc_ids, vec!["carla".to_string()]);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(result.movement.destination, Some("entrance".to_string()));
}

#[test]
fn test_mock_quantifier_no_match_when_different_action() {
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];

    let backend = MockQuantifierBackend::default();

    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "I look at the wall.", // No NPC name mentioned
    };

    let result = backend.quantify_room(&context, &[]).unwrap();

    // Should not auto-detect any NPCs
    assert!(result.npcs.npc_ids.is_empty());
}

#[test]
fn test_action_boundary_substring_at_start_no_boundary_after() {
    // "carla" in "carlax" should NOT match (no boundary after)
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("carlax", "carla", &boundary_chars);
    assert!(
        !result,
        "Should not match when followed by non-boundary char"
    );
}

#[test]
fn test_action_boundary_substring_at_end_no_boundary_before() {
    // "carla" in "xcarla" should NOT match (no boundary before)
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("xcarla", "carla", &boundary_chars);
    assert!(
        !result,
        "Should not match when preceded by non-boundary char"
    );
}

#[test]
fn test_quantifier_prompt_builder_all_rooms() {
    let room = make_room();
    let all_npcs = vec![make_npc("carla", "Carla")];
    let all_rooms = vec![
        RoomInfo {
            id: "entrance".to_string(),
            name: "Entrance".to_string(),
        },
        RoomInfo {
            id: "library".to_string(),
            name: "Library".to_string(),
        },
    ];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &all_npcs,
        all_rooms: &all_rooms,
        player_name: "Hero",
        recent_history: &[],
        player_action: "I look around.",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (system, _) = builder.build();

    // System prompt should include all rooms
    assert!(system.contains("AvailableRooms"));
    assert!(system.contains("Entrance"));
    assert!(system.contains("Library"));
}

#[test]
fn test_parse_response_json_with_movement_null_type() {
    // JSON with movement.type = null should result in None movement type
    let response =
        r#"{"npcs_in_room": ["carla"], "movement": {"type": null, "destination": "hall"}}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    // Destination should still be captured even with null type
    assert_eq!(result.movement.destination, Some("hall".to_string()));
}

#[test]
fn test_compute_npc_events_both_empty() {
    let previous: Vec<String> = vec![];
    let current: Vec<String> = vec![];

    let result = compute_npc_events(&previous, &current);

    assert!(result.events.is_empty());
    assert_eq!(result.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_compute_npc_events_all_left() {
    // All previous NPCs left
    let previous = vec!["carla".to_string(), "gabriella".to_string()];
    let current: Vec<String> = vec![];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 2);
    assert!(
        result
            .events
            .iter()
            .all(|e| e.event_type == NpcEventType::Left)
    );
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_compute_npc_events_all_entered() {
    // All new NPCs entered
    let previous: Vec<String> = vec![];
    let current = vec!["carla".to_string(), "gabriella".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 2);
    assert!(
        result
            .events
            .iter()
            .all(|e| e.event_type == NpcEventType::Entered)
    );
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_quantifier_prompt_uses_latest_narration_tag() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (_system, user) = builder.build();

    // Should use <LatestNarration> instead of <PlayerAction>
    assert!(
        user.contains("<LatestNarration>"),
        "User prompt should contain <LatestNarration> tag"
    );
    assert!(
        !user.contains("<PlayerAction>"),
        "User prompt should not contain old <PlayerAction> tag"
    );
}

#[test]
fn test_quantifier_prompt_references_latest_narration_in_query() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (_system, user) = builder.build();

    // Query should tell LLM to focus on <LatestNarration>
    assert!(
        user.contains("<LatestNarration>"),
        "Query should reference <LatestNarration>"
    );
}

#[test]
fn test_quantifier_retry_on_low_confidence() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let mut call_count = 0;
    let mock_llm = |_: &str, _: &str, _: &str| -> Result<String, String> {
        call_count += 1;
        if call_count == 1 {
            // First call: completely invalid response → Low confidence
            Ok("I am not sure what to say here.".to_string())
        } else {
            // Second call: valid JSON → High confidence
            Ok(r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string())
        }
    };

    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], "mock", mock_llm);

    assert!(result.is_ok());
    let result = result.unwrap();

    // Should have retried (2 calls)
    assert_eq!(call_count, 2, "Expected retry on low confidence");

    // Should get the high-confidence result from the second call
    assert_eq!(result.npcs.npc_ids, vec!["carla".to_string()]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
    assert_eq!(result.movement.movement_type, None);
}

#[test]
fn test_quantifier_no_retry_when_high_confidence() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let mut call_count = 0;
    let mock_llm = |_: &str, _: &str, _: &str| -> Result<String, String> {
        call_count += 1;
        Ok(r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string())
    };

    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], "mock", mock_llm);

    assert!(result.is_ok());
    let result = result.unwrap();

    // Should NOT retry when first response is high confidence
    assert_eq!(call_count, 1, "Should not retry on high confidence");
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
}
