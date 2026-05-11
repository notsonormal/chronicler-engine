use std::collections::HashMap;

use chrono::Utc;

use crate::model::character::{CharacterSheet, NpcCard};
use crate::model::map::{Direction, Room};
use crate::model::state::{LogEntry, LogType};

pub fn make_room() -> Room {
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
    }
}

pub fn make_history() -> Vec<LogEntry> {
    vec![
        LogEntry {
            id: 1,
            sender: Some("Narrator".to_string()),
            text: "You enter the front gate.".to_string(),
            log_type: LogType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
        LogEntry {
            id: 2,
            sender: Some("Carla".to_string()),
            text: "I'll follow you inside.".to_string(),
            log_type: LogType::Dialogue,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ]
}

pub fn make_boundary_chars() -> std::collections::HashSet<char> {
    [
        ' ', '.', ',', '!', '?', '\n', '\t', '\r', '\'', '"', ':', ';',
    ]
    .into_iter()
    .collect()
}
