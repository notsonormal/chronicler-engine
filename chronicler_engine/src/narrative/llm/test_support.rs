use std::collections::HashMap;

use crate::model::character::{CharacterSheet, PlayerCard};
use crate::model::map::Room;
use crate::model::world::WorldCard;
use crate::narrative::prompt::PromptContext;

pub fn make_test_room() -> Room {
    // [DOC: docs/reference/testing.md]
    Room {
        id: "room1".to_string(),
        name: "Test Room".to_string(),
        description: "A plain room.".to_string(),
        exits: HashMap::new(),
        items: vec![],
        npcs: vec![],
        image_path: None,
        navigation_description: None,
    }
}

pub fn make_test_world() -> WorldCard {
    // [DOC: docs/reference/testing.md]
    WorldCard {
        name: "Test World".to_string(),
        description: "Testing.".to_string(),
        global_rules: vec!["Rule 1".to_string()],
        ..Default::default()
    }
}

pub fn make_test_player() -> PlayerCard {
    // [DOC: docs/reference/testing.md]
    PlayerCard {
        sheet: CharacterSheet {
            name: "Hero".to_string(),
            description: "The protagonist.".to_string(),
            personality: "Brave".to_string(),
            scenario: "Generic Quest".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    }
}

pub fn make_test_context(user_message: &str) -> PromptContext<'static> {
    // [DOC: docs/reference/testing.md]
    let world = make_test_world();
    let room = make_test_room();
    let player = make_test_player();
    let npcs: Vec<crate::model::character::NpcCard> = vec![];
    let user_msg = user_message.to_string();
    PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(npcs)),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new(user_msg)),
        history: &[],
    }
}

pub fn make_test_context_with_npc(
    npc: &crate::model::character::NpcCard,
    user_message: &str,
) -> PromptContext<'static> {
    // [DOC: docs/reference/testing.md]
    let world = make_test_world();
    let room = make_test_room();
    let player = make_test_player();
    let npcs = vec![npc.clone()];
    let npcs_in_area = vec![npc.clone()];
    let user_msg = user_message.to_string();
    PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(npcs)),
        npcs_in_area: Box::leak(Box::new(npcs_in_area)),
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new(user_msg)),
        history: &[],
    }
}
