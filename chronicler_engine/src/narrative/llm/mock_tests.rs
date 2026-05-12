use crate::model::character::{CharacterSheet, NpcCard};
use crate::model::state::{LogEntry, LogType};
use crate::narrative::llm::backend::LlmBackend;
use crate::narrative::llm::mock::MockBackend;
use crate::narrative::llm::test_support::{
    make_test_context, make_test_context_with_npc, make_test_room, make_test_world,
};
use chrono::Utc;

#[test]
fn test_mock_narrate_action() {
    let backend = MockBackend::default();
    let context = make_test_context("I look around carefully.");

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "[MockNarration] I look around carefully.");
}

#[test]
fn test_mock_generate_dialogue_with_message() {
    let backend = MockBackend::default();
    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "Guard".to_string(),
            description: "A stern guard".to_string(),
            personality: "Suspicious".to_string(),
            scenario: "Watching the gate".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };
    let message = "Hello, guard!";

    let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, message), &npc);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "[MockGenerated] Replying to: Hello, guard!"
    );
}

#[test]
fn test_mock_generate_dialogue_no_message() {
    let backend = MockBackend::default();
    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "Guard".to_string(),
            description: "A stern guard".to_string(),
            personality: "Suspicious".to_string(),
            scenario: "Watching the gate".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };

    let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, ""), &npc);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "[MockGenerated] Standard greeting.");
}

#[test]
fn test_mock_backend_name() {
    let backend = MockBackend::default();
    assert_eq!(backend.name(), "Mock");
}

#[test]
fn test_mock_with_history() {
    let backend = MockBackend::default();

    let _history = [
        LogEntry {
            id: 1,
            sender: Some("Narrator".to_string()),
            text: "You see a mysterious figure.".to_string(),
            log_type: LogType::Narration,
            timestamp: Utc::now(),
            ..Default::default()
        },
        LogEntry {
            id: 2,
            sender: Some("Player".to_string()),
            text: "Hello?".to_string(),
            log_type: LogType::Input,
            timestamp: Utc::now(),
            ..Default::default()
        },
    ];

    let result = backend.narrate_action(&make_test_context("I approach"));
    assert!(result.is_ok());
    assert!(result.unwrap().contains("I approach"));
}

#[test]
fn test_mock_response_length_bounds() {
    let backend = MockBackend::default();

    let short_input = "hi";
    let long_input =
        "This is a much longer player input that describes what the player wants to do in detail";

    let result_short = backend.narrate_action(&make_test_context(short_input));
    let result_long = backend.narrate_action(&make_test_context(long_input));

    assert!(result_short.is_ok());
    assert!(result_long.is_ok());

    assert!(!result_short.unwrap().is_empty());
    assert!(!result_long.unwrap().is_empty());
}

#[test]
fn test_mock_response_contains_input() {
    let backend = MockBackend::default();

    let unique_input = "xyz123_test_input";
    let result = backend.narrate_action(&make_test_context(unique_input));

    assert!(result.is_ok());
    assert!(result.unwrap().contains(unique_input));
}

#[test]
fn test_mock_narrate_arrival_includes_room_name() {
    let backend = MockBackend::default();
    let room = make_test_room();

    let result = backend.narrate_arrival(&make_test_context(""));

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("enter"));
    assert!(response.contains(&room.name));
}

#[test]
fn test_mock_dialogue_with_message() {
    let backend = MockBackend::default();
    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "Guard".to_string(),
            description: "A stern guard".to_string(),
            personality: "Alert".to_string(),
            scenario: "Watching".to_string(),
            example_dialogue: "Halt!".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };

    let message = Some("Hello, guard!".to_string());
    let result = backend.generate_dialogue(
        &make_test_context_with_npc(&npc, message.as_deref().unwrap_or("")),
        &npc,
    );

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("Hello, guard!"));
}

#[test]
fn test_mock_dialogue_without_message() {
    let backend = MockBackend::default();
    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "Guard".to_string(),
            description: "A stern guard".to_string(),
            personality: "Alert".to_string(),
            scenario: "Watching".to_string(),
            example_dialogue: "Halt!".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };

    let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, ""), &npc);

    assert!(result.is_ok());
    assert!(result.unwrap().contains("greeting"));
}

#[test]
fn test_mock_with_empty_history() {
    let backend = MockBackend::default();

    let result = backend.narrate_action(&make_test_context("test"));
    assert!(result.is_ok());
}

#[test]
fn test_mock_with_substantial_history() {
    let backend = MockBackend::default();

    let _history: Vec<LogEntry> = (0..50)
        .map(|i| LogEntry {
            id: i as u64,
            sender: Some(format!("Speaker{}", i % 3)),
            text: format!("This is narration entry number {i} in the game history."),
            log_type: if i % 2 == 0 {
                LogType::Narration
            } else {
                LogType::Input
            },
            timestamp: Utc::now(),
            ..Default::default()
        })
        .collect();

    let result = backend.narrate_action(&make_test_context("current action"));
    assert!(result.is_ok());
}

#[test]
fn test_mock_narrate_continuation() {
    let backend = MockBackend::default();
    let result = backend.narrate_continuation("system", "user", "trigger_info", None);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("trigger_info"));
}

#[test]
fn test_mock_narrate_action_from_prompt() {
    let backend = MockBackend::default();
    let result = backend.narrate_action_from_prompt("system prompt", "user action", None);
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("action") || response.contains("Continuation"));
}

#[test]
fn test_mock_narrate_continuation_empty_trigger() {
    let backend = MockBackend::default();
    let result = backend.narrate_continuation("system", "user", "", None);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("[Trigger: ]"));
}

#[test]
fn test_mock_narrate_continuation_special_chars() {
    let backend = MockBackend::default();
    let result =
        backend.narrate_continuation("sys", "user", "trigger with <special> & chars", None);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("trigger with"));
}

#[test]
fn test_mock_narrate_action_from_prompt_multiline() {
    let backend = MockBackend::default();
    let result = backend.narrate_action_from_prompt(
        "system prompt\nwith multiple lines",
        "user prompt\nalso multiline",
        None,
    );
    assert!(result.is_ok());
    assert!(result.unwrap().contains("user prompt"));
}

#[test]
fn test_mock_narrate_action_from_prompt_empty() {
    let backend = MockBackend::default();
    let result = backend.narrate_action_from_prompt("", "", None);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("..."));
}

#[test]
fn test_mock_generate_dialogue_very_long_message() {
    let backend = MockBackend::default();
    let long_message = "This is a very long user message that goes on and on and on to test how the mock backend handles lengthy inputs without any issues whatsoever because it should just echo back whatever it receives.";
    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "LongNameNPC".to_string(),
            description: "A very long description that describes this NPC in great detail"
                .to_string(),
            personality: "Some personality traits that are described here".to_string(),
            scenario: "A scenario description that is also quite lengthy".to_string(),
            example_dialogue: "Example dialogue text".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };
    let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, long_message), &npc);
    assert!(result.is_ok());
    assert!(result.unwrap().contains(long_message));
}

#[test]
fn test_mock_narrate_action_special_characters() {
    let backend = MockBackend::default();
    let special_msg = "Player says: \"Hello <world> & goodbye!\"";
    let result = backend.narrate_action(&make_test_context(special_msg));
    assert!(result.is_ok());
    assert!(result.unwrap().contains(special_msg));
}

#[test]
fn test_mock_narrate_arrival_different_rooms() {
    let backend = MockBackend::default();

    let mut room1 = make_test_room();
    room1.name = "Tavern".to_string();
    let world = make_test_world();
    let player = crate::narrative::llm::test_support::make_test_player();
    let context1 = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world.clone())),
        room: Box::leak(Box::new(room1.clone())),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player.clone())),
        user_message: Box::leak(Box::new("".to_string())),
        history: &[],
    };
    let result1 = backend.narrate_arrival(&context1);
    assert!(result1.is_ok());
    assert!(result1.unwrap().contains("Tavern"));

    let mut room2 = make_test_room();
    room2.name = "Dungeon".to_string();
    let context2 = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room2)),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("".to_string())),
        history: &[],
    };
    let result2 = backend.narrate_arrival(&context2);
    assert!(result2.is_ok());
    assert!(result2.unwrap().contains("Dungeon"));
}

#[test]
fn test_mock_dialogue_with_unicode() {
    let backend = MockBackend::default();
    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "日本語NPC".to_string(),
            description: "A Japanese NPC".to_string(),
            personality: "Friendly".to_string(),
            scenario: "Test".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };
    let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, "こんにちは"), &npc);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("こんにちは"));
}

#[test]
fn test_mock_narrate_action_unicode() {
    let backend = MockBackend::default();
    let result = backend.narrate_action(&make_test_context("アクション"));
    assert!(result.is_ok());
    assert!(result.unwrap().contains("アクション"));
}

#[test]
fn test_mock_narrate_continuation_unicode_trigger() {
    let backend = MockBackend::default();
    let result = backend.narrate_continuation("system", "user", "トリガー", None);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("トリガー"));
}

#[test]
fn test_context_with_empty_world_description() {
    let backend = MockBackend::default();
    let world = crate::model::world::WorldCard {
        name: "Empty World".to_string(),
        description: "".to_string(),
        global_rules: vec![],
        ..Default::default()
    };
    let room = make_test_room();
    let player = crate::narrative::llm::test_support::make_test_player();

    let context = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("test".to_string())),
        history: &[],
    };

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
}

#[test]
fn test_context_with_many_global_rules() {
    let backend = MockBackend::default();
    let world = crate::model::world::WorldCard {
        name: "Rules World".to_string(),
        description: "A world with many rules".to_string(),
        global_rules: vec![
            "Rule 1".to_string(),
            "Rule 2".to_string(),
            "Rule 3".to_string(),
            "Rule 4".to_string(),
            "Rule 5".to_string(),
        ],
        ..Default::default()
    };
    let room = make_test_room();
    let player = crate::narrative::llm::test_support::make_test_player();

    let context = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("test".to_string())),
        history: &[],
    };

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
}

#[test]
fn test_mock_narrate_action_from_prompt_very_long_input() {
    let backend = MockBackend::default();
    let long_system = "You are a game master. ".repeat(50);
    let long_user = "The player performs an action. ".repeat(50);
    let result = backend.narrate_action_from_prompt(&long_system, &long_user, None);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("The player performs"));
}

#[test]
fn test_npc_with_no_triggers() {
    let backend = MockBackend::default();
    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "Plain NPC".to_string(),
            description: "A plain NPC with no triggers".to_string(),
            personality: "Neutral".to_string(),
            scenario: "Standing around".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };
    let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, "Hello"), &npc);
    assert!(result.is_ok());
}

#[test]
fn test_npc_with_multiple_triggers() {
    use crate::model::trigger::{ComparisonOperator, Trigger, TriggerAction, TriggerCondition};
    let backend = MockBackend::default();
    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "Trigger NPC".to_string(),
            description: "An NPC with multiple triggers".to_string(),
            personality: "Variable".to_string(),
            scenario: "Complex".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![
            Trigger {
                condition: TriggerCondition::TimesMet(ComparisonOperator::Eq, 1),
                action: TriggerAction {
                    name: "Trigger One".to_string(),
                    narration_prompt: "trigger1".to_string(),
                },
                repeat: false,
                room_id: None,
            },
            Trigger {
                condition: TriggerCondition::TimesMet(ComparisonOperator::Gte, 2),
                action: TriggerAction {
                    name: "Trigger Two".to_string(),
                    narration_prompt: "trigger2".to_string(),
                },
                repeat: true,
                room_id: None,
            },
        ],
        relationships: vec![],
    };
    let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, "Test"), &npc);
    assert!(result.is_ok());
}

#[test]
fn test_player_with_empty_inventory() {
    let backend = MockBackend::default();
    let player = crate::model::character::PlayerCard {
        sheet: CharacterSheet {
            name: "Hero".to_string(),
            description: "A hero with no items".to_string(),
            personality: "Brave".to_string(),
            scenario: "Quest".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    };
    let world = make_test_world();
    let room = make_test_room();

    let context = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("test".to_string())),
        history: &[],
    };

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
}

#[test]
fn test_mock_with_empty_response() {
    let backend = MockBackend::with_empty_response();
    let result = backend.narrate_action(&make_test_context("I look around."));
    assert!(
        result.is_ok(),
        "with_empty_response should return Ok, not Err: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        "",
        "with_empty_response should return an empty string"
    );
}

#[test]
fn test_mock_with_failing_trigger_narration() {
    let backend = MockBackend::with_failing_trigger_narration();
    // narrate_action should still succeed
    let narrate_result = backend.narrate_action(&make_test_context("look"));
    assert!(
        narrate_result.is_ok(),
        "narrate_action should succeed even with trigger_narration_should_fail set"
    );
    // narrate_action_from_prompt (used for trigger narration) should fail
    let trigger_result = backend.narrate_action_from_prompt("sys", "user", None);
    assert!(
        trigger_result.is_err(),
        "narrate_action_from_prompt should fail when trigger_narration_should_fail is set"
    );
    assert!(
        trigger_result
            .unwrap_err()
            .to_string()
            .contains("mock_trigger"),
        "Error message should identify this as a trigger narration failure"
    );
}

#[test]
fn test_player_with_items_in_inventory() {
    let backend = MockBackend::default();
    let player = crate::model::character::PlayerCard {
        sheet: CharacterSheet {
            name: "Equipped Hero".to_string(),
            description: "A hero with items".to_string(),
            personality: "Prepared".to_string(),
            scenario: "Quest".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![
            "Sword".to_string(),
            "Shield".to_string(),
            "Potion".to_string(),
        ],
    };
    let world = make_test_world();
    let room = make_test_room();

    let context = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("test".to_string())),
        history: &[],
    };

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
}

#[test]
fn test_room_with_items() {
    let backend = MockBackend::default();
    let room = crate::model::map::Room {
        id: "room_with_items".to_string(),
        name: "Storage Room".to_string(),
        description: "A room full of items".to_string(),
        exits: std::collections::HashMap::new(),
        items: vec![
            "Chest".to_string(),
            "Barrel".to_string(),
            "Table".to_string(),
        ],
        image_path: None,
        navigation_description: None,
    };
    let world = make_test_world();
    let player = crate::narrative::llm::test_support::make_test_player();

    let context = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("look".to_string())),
        history: &[],
    };

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
}

#[test]
fn test_room_with_exits() {
    use crate::model::map::Direction;
    let backend = MockBackend::default();
    let mut exits = std::collections::HashMap::new();
    exits.insert(Direction::North, "hallway".to_string());
    exits.insert(Direction::East, "kitchen".to_string());
    exits.insert(Direction::South, "garden".to_string());

    let room = crate::model::map::Room {
        id: "room_with_exits".to_string(),
        name: "Central Room".to_string(),
        description: "A central room with many exits".to_string(),
        exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };
    let world = make_test_world();
    let player = crate::narrative::llm::test_support::make_test_player();

    let context = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("exits".to_string())),
        history: &[],
    };

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
}

#[test]
fn test_world_with_default_room_image() {
    let backend = MockBackend::default();
    let world = crate::model::world::WorldCard {
        name: "World with Image".to_string(),
        description: "A world with default room image".to_string(),
        global_rules: vec!["Rule".to_string()],
        starting_room_id: "room1".to_string(),
        scenarios: vec![],
        default_room_image: Some("default_room.png".to_string()),
    };
    let room = make_test_room();
    let player = crate::narrative::llm::test_support::make_test_player();

    let context = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(vec![])),
        npcs_in_area: &[],
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("test".to_string())),
        history: &[],
    };

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
}

#[test]
fn test_npc_with_inventory() {
    let backend = MockBackend::default();
    let npc = NpcCard {
        id: "npc_with_items".to_string(),
        sheet: CharacterSheet {
            name: "Merchant".to_string(),
            description: "A merchant with items".to_string(),
            personality: "Greedy".to_string(),
            scenario: "Trading".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec!["Gold".to_string(), "Gem".to_string(), "Map".to_string()],
        triggers: vec![],
        relationships: vec![],
    };
    let result =
        backend.generate_dialogue(&make_test_context_with_npc(&npc, "What do you sell?"), &npc);
    assert!(result.is_ok());
}

#[test]
fn test_context_with_npcs_in_area() {
    let backend = MockBackend::default();
    let npc1 = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "Guard1".to_string(),
            description: "First guard".to_string(),
            personality: "Alert".to_string(),
            scenario: "Watching".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };
    let npc2 = NpcCard {
        id: "npc2".to_string(),
        sheet: CharacterSheet {
            name: "Guard2".to_string(),
            description: "Second guard".to_string(),
            personality: "Alert".to_string(),
            scenario: "Watching".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    };

    let world = make_test_world();
    let room = make_test_room();
    let player = crate::narrative::llm::test_support::make_test_player();
    let all_npcs = vec![npc1.clone(), npc2.clone()];
    let npcs_in_area = vec![npc1.clone()];

    let context = crate::narrative::prompt::PromptContext {
        world: Box::leak(Box::new(world)),
        room: Box::leak(Box::new(room)),
        all_npcs: Box::leak(Box::new(all_npcs)),
        npcs_in_area: Box::leak(Box::new(npcs_in_area)),
        player: Box::leak(Box::new(player)),
        user_message: Box::leak(Box::new("look".to_string())),
        history: &[],
    };

    let result = backend.narrate_action(&context);
    assert!(result.is_ok());
}
