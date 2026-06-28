//! [DOC: docs/system/agent_system.md]
//! Quantifier test utilities

use std::collections::HashMap;

use chrono::Utc;

use crate::model::character::{CharacterSheet, NpcCard};
use crate::model::map::{Direction, Room};
use crate::model::state::message_types::{MessageEntry, MessageType};

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

pub fn make_npc(id: &str, name: &str) -> NpcCard {
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
        relationships: vec![],
    }
}

pub fn make_history() -> Vec<MessageEntry> {
    vec![
        MessageEntry {
            id: 1,
            sender: Some("Narrator".to_string()),
            text: "You enter the front gate.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
        MessageEntry {
            id: 2,
            sender: Some("Carla".to_string()),
            text: "I'll follow you inside.".to_string(),
            message_type: MessageType::Dialogue,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ]
}
