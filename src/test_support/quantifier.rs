//! Quantifier test utilities

use std::collections::HashMap;

use crate::domain::model::character::{CharacterSheet, CharacterCard};
use crate::domain::model::map::{Direction, Room};
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::test_support::fixtures::default_story_row;

pub fn make_room() -> Room {
    Room {
        id: "entrance_hall".to_string(),
        name: "Entrance Hall".to_string(),
        description: "A grand entrance hall with marble floors.".to_string(),
        exits: HashMap::from([(Direction::North, "library".to_string())]),
        items: vec![],
        image_path: None,
        navigation_description: None,
    }
}

pub fn make_character(id: &str, name: &str) -> CharacterCard {
    CharacterCard {
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
        relationships: vec![],
    }
}

pub fn make_history() -> Vec<StoryRow> {
    vec![
        StoryRow {
            text: "You enter the front gate.".to_string(),
            message_type: MessageType::Narration,
            ..default_story_row(MessageType::Narration)
        },
        StoryRow {
            text: "I'll follow you inside.".to_string(),
            message_type: MessageType::Input,
            ..default_story_row(MessageType::Input)
        },
    ]
}
