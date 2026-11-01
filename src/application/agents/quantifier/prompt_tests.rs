use std::collections::HashMap;

use crate::domain::model::character::CharacterCard;
use crate::domain::model::map::Room;
use crate::domain::model::state::message_types::StoryRow;

use super::prompt::QuantifierPromptBuilder;
use crate::test_support::fixtures::TestPersona;
use crate::test_support::quantifier::{make_history, make_character, make_room};
use super::types::{QuantifierPromptContext, RoomInfo};

#[test]
fn test_quantifier_prompt_builder_basic() {
    let room = make_room();
    let carla = make_character("carla", "Carla");
    let gabriella = make_character("gabriella", "Gabriella");
    let all_characters = vec![carla.clone(), gabriella.clone()];
    let previous_characters = vec![carla];
    let history = make_history();

    let override_text = r#"You are a scene quantifier for a text adventure game.
Your task is to determine which Characters are present in the current room.

Respond ONLY with a JSON object in this exact format:
{"characters_present": ["id1", "id2"]}

How to determine movement:
1. Read <CurrentRoom> — this is where the player is right now.
2. Read <LatestMessage> — this is what just happened.
"#;

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        persona: &TestPersona::standard(),
        recent_history: &history,
        latest_message: "I walk into the entrance hall.",
        quantifier_prompt_override: Some(override_text.to_string()),
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (system, user) = builder.build();

    assert!(system.contains("scene quantifier"));
    assert!(system.contains("characters_present"));
    assert!(system.contains("carla"));
    assert!(system.contains("gabriella"));
    assert!(system.contains("How to determine movement"));
    assert!(system.contains("Read <CurrentRoom>"));
    assert!(system.contains("Read <LatestMessage>"));
    assert!(user.contains("Entrance Hall"));
    assert!(user.contains("Carla"));
    assert!(user.contains("Hero"));
    assert!(!system.contains("<QuantifierTask>"));
    assert!(!system.contains("<AuxiliaryInstructions>"));
    assert!(!user.contains("<Query>"));
}

#[test]
fn test_quantifier_prompt_builder_token_budget() {
    let room = make_room();
    let carla = make_character("carla", "Carla");
    let gabriella = make_character("gabriella", "Gabriella");
    let all_characters = vec![carla, gabriella];
    let previous_characters: Vec<CharacterCard> = vec![];
    let history = make_history();

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        persona: &TestPersona::standard(),
        recent_history: &history,
        latest_message: "I look around.",
        quantifier_prompt_override: None,
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (system, user) = builder.build();

    let total_chars = system.len() + user.len();
    assert!(
        total_chars < 4000,
        "Quantifier prompt too long: {total_chars} chars"
    );
}

#[test]
fn test_quantifier_prompt_includes_navigation() {
    let room = Room {
        id: "test_room".to_string(),
        name: "Test Room".to_string(),
        description: "A test room.".to_string(),
        exits: HashMap::new(),
        items: vec![],
        image_path: None,
        navigation_description: Some("You can go north to the kitchen.".to_string()),
    };

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &[],
        all_known_characters: &[],
        all_rooms: &[RoomInfo {
            id: "kitchen".to_string(),
            name: "Kitchen".to_string(),
        }],
        persona: &TestPersona::named("Player"),
        recent_history: &[],
        latest_message: "I walk to the kitchen",
        quantifier_prompt_override: None,
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (_, user_prompt) = builder.build();

    assert!(user_prompt.contains("<Navigation>"));
    assert!(user_prompt.contains("You can go north to the kitchen"));
}

#[test]
fn test_quantifier_prompt_builder_empty_characters() {
    let room = make_room();
    let all_characters: Vec<CharacterCard> = vec![];
    let previous_characters: Vec<CharacterCard> = vec![];
    let history: Vec<StoryRow> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        persona: &TestPersona::standard(),
        recent_history: &history,
        latest_message: "I look around.",
        quantifier_prompt_override: None,
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (system, user) = builder.build();

    assert!(system.contains("available_character_ids"));
    assert!(
        user.contains("<LatestMessage>\n  I look around.\n</LatestMessage>"),
        "the newest Message still renders: {user}"
    );
}

#[test]
fn test_quantifier_prompt_builder_all_rooms() {
    let room = make_room();
    let all_characters = vec![make_character("carla", "Carla")];
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
        previous_characters_present: &[],
        all_known_characters: &all_characters,
        all_rooms: &all_rooms,
        persona: &TestPersona::standard(),
        recent_history: &[],
        latest_message: "I look around.",
        quantifier_prompt_override: None,
    };

    let builder = QuantifierPromptBuilder::new(context);
    let (system, _) = builder.build();

    assert!(system.contains("available_rooms"));
    assert!(system.contains("Entrance"));
    assert!(system.contains("Library"));
}
