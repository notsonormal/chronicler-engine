use std::collections::HashMap;

use crate::model::character::NpcCard;
use crate::model::map::Room;
use crate::model::state::LogEntry;

use super::prompt::QuantifierPromptBuilder;
use super::test_support::{make_history, make_npc, make_room};
use super::types::{QuantifierPromptContext, RoomInfo};

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

    assert!(system.contains("scene quantifier"));
    assert!(system.contains("npcs_in_room"));
    assert!(system.contains("carla"));
    assert!(system.contains("gabriella"));
    assert!(system.contains("How to determine movement"));
    assert!(system.contains("Read <CurrentRoom>"));
    assert!(system.contains("Read <LatestNarration>"));
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

    assert!(user.contains("Hero"));
    assert!(user.contains("I look around"));
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

    assert!(system.contains("AvailableNpcIds"));
    assert!(user.contains("Hero"));
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

    assert!(system.contains("AvailableRooms"));
    assert!(system.contains("Entrance"));
    assert!(system.contains("Library"));
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

    assert!(
        user.contains("<LatestNarration>"),
        "Query should reference <LatestNarration>"
    );
}
