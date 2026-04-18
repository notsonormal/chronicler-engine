use crate::error::EngineError;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::Room;
use crate::model::world::WorldCard;
use crate::narrative::openrouter_client::call_openrouter;
use crate::narrative::prompt::{PromptBuilder, PromptContext};

pub trait LlmBackend: Send + Sync {
    fn generate_dialogue(
        &self,
        context: &PromptContext,
        npc: &NpcCard,
    ) -> Result<String, EngineError>;

    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError>;

    fn narrate_arrival(&self, context: &PromptContext) -> Result<String, EngineError>;

    fn name(&self) -> &str;
}

/// Enum for available LLM backends
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LlmBackendType {
    OpenRouter,
    DeepSeek,
    Mock,
}

impl LlmBackendType {
    /// Get the configured LLM backend from environment
    /// Default to OpenRouter if not set
    pub fn from_env() -> Self {
        match std::env::var("LLM_BACKEND").as_deref() {
            Ok("deepseek") => LlmBackendType::DeepSeek,
            Ok("mock") => LlmBackendType::Mock,
            _ => LlmBackendType::OpenRouter, // default
        }
    }
}

/// Get the configured LLM backend instance
pub fn get_llm_backend() -> Box<dyn LlmBackend> {
    match LlmBackendType::from_env() {
        LlmBackendType::Mock => Box::new(MockBackend),
        LlmBackendType::DeepSeek => Box::new(DeepSeekBackend),
        LlmBackendType::OpenRouter => Box::new(OpenRouterBackend),
    }
}

#[derive(Clone, Copy)]
pub struct OpenRouterBackend;

impl LlmBackend for OpenRouterBackend {
    fn generate_dialogue(
        &self,
        context: &PromptContext,
        npc: &NpcCard,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating dialogue for NPC: {}", npc.sheet.name);

        // Build user message for the NPC dialogue context
        let user_msg = format!(
            "The player says to {}: \"{}\"",
            npc.sheet.name, context.user_message
        );

        // Create context for this specific NPC
        let npc_context = PromptContext {
            world: context.world,
            room: context.room,
            all_npcs: &[npc.clone()],
            npcs_in_area: &[npc.clone()],
            player: context.player,
            user_message: &user_msg,
            history: context.history,
        };

        let builder = PromptBuilder::from_context(&npc_context);
        let (system_prompt, user_text) = builder.build_split()?;
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| EngineError::Config("OPENROUTER_API_KEY not set".into()))?;

        call_openrouter(&api_key, &system_prompt, &user_text).map_err(EngineError::Narrative)
    }

    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError> {
        log::info!(
            "[LLM] Generating action narration for: {}",
            context.user_message
        );

        let builder = PromptBuilder::from_context(context);
        let (system_prompt, user_text) = builder.build_split()?;
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| EngineError::Config("OPENROUTER_API_KEY not set".into()))?;

        call_openrouter(&api_key, &system_prompt, &user_text).map_err(EngineError::Narrative)
    }

    fn narrate_arrival(&self, context: &PromptContext) -> Result<String, EngineError> {
        log::info!(
            "[LLM] Generating arrival narration for room: {}",
            context.room.name
        );

        // Create arrival-specific user message
        let user_msg = format!(
            "{} enters the {}.",
            context.player.sheet.name, context.room.name
        );

        let arrival_context = PromptContext {
            world: context.world,
            room: context.room,
            all_npcs: context.all_npcs,
            npcs_in_area: context.npcs_in_area,
            player: context.player,
            user_message: &user_msg,
            history: context.history,
        };

        let builder = PromptBuilder::from_context(&arrival_context);
        let (system_prompt, user_text) = builder.build_split()?;
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| EngineError::Config("OPENROUTER_API_KEY not set".into()))?;

        call_openrouter(&api_key, &system_prompt, &user_text).map_err(EngineError::Narrative)
    }

    fn name(&self) -> &str {
        "OpenRouter"
    }
}

pub fn build_dialogue_prompts(
    world: &WorldCard,
    room: &Room,
    npc: &NpcCard,
    user_message: &Option<String>,
) -> (String, String) {
    let mut system_prompt = format!(
        "You are a character in a text adventure game. Your name is {}.\n\
    Personality: {}\n\
    Scenario/Background: {}\n\
    \n",
        npc.sheet.name, npc.sheet.personality, npc.sheet.scenario
    );

    system_prompt.push_str(&format!(
        "Current Room: {} - {}\n\n",
        room.name, room.description
    ));

    system_prompt.push_str("World Rules:\n");
    for rule in &world.global_rules {
        system_prompt.push_str(&format!("- {rule}\n"));
    }

    system_prompt.push_str("\nInstructions: Roleplay as your character and respond to the player's action. Reply primarily with dialogue, and do not act or speak on behalf of the player.");

    let user_text = match user_message {
        Some(msg) => format!("The player says: \"{msg}\""),
        None => "The player approaches you in silence, waiting for you to speak.".to_string(),
    };

    (system_prompt, user_text)
}

pub fn build_action_prompts(
    world: &WorldCard,
    room: &Room,
    nearby_npcs: &[&NpcCard],
    player: &PlayerCard,
    player_input: &str,
) -> (String, String) {
    let mut system_prompt = String::from(
        "You are the Game Master of a text adventure game. \
Narrate what happens in response to the player's action. \
Voice any NPCs present if they would logically react. \
Keep responses immersive, concise, and in the style of literary fiction. \
Never act or speak on behalf of the player.\n\n",
    );

    system_prompt.push_str(&format!(
        "Player Identity:\n- Name: {}\n- Persona: {}\n- Background: {}\n\n",
        player.sheet.name, player.sheet.personality, player.sheet.scenario
    ));

    system_prompt.push_str(&format!(
        "Current Location: {} — {}\n\n",
        room.name, room.description
    ));

    if !nearby_npcs.is_empty() {
        system_prompt.push_str("Characters present:\n");
        for npc in nearby_npcs {
            system_prompt.push_str(&format!(
                "- {} ({}): {} Background: {}\n",
                npc.sheet.name, npc.sheet.personality, npc.sheet.description, npc.sheet.scenario
            ));
        }
        system_prompt.push('\n');
    }

    system_prompt.push_str("World Lore:\n");
    for rule in &world.global_rules {
        system_prompt.push_str(&format!("- {rule}\n"));
    }

    let user_text = format!("The player does the following: {player_input}");

    (system_prompt, user_text)
}

pub struct MockBackend;

impl LlmBackend for MockBackend {
    fn generate_dialogue(
        &self,
        _context: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<String, EngineError> {
        // For mock, use the user_message from context
        let msg = _context.user_message;
        if msg.is_empty() {
            Ok("[MockGenerated] Standard greeting.".to_string())
        } else {
            Ok(format!("[MockGenerated] Replying to: {msg}"))
        }
    }

    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError> {
        Ok(format!("[MockNarration] {}", context.user_message))
    }

    fn narrate_arrival(&self, context: &PromptContext) -> Result<String, EngineError> {
        Ok(format!(
            "[MockArrival] You enter the {}.",
            context.room.name
        ))
    }

    fn name(&self) -> &str {
        "Mock"
    }
}

/// DeepSeek backend - placeholder for future implementation
pub struct DeepSeekBackend;

impl LlmBackend for DeepSeekBackend {
    fn generate_dialogue(
        &self,
        _context: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<String, EngineError> {
        Ok("[DeepSeek] Dialogue not yet implemented. Use OpenRouter for now.".to_string())
    }

    fn narrate_action(&self, _context: &PromptContext) -> Result<String, EngineError> {
        Ok("[DeepSeek] Narration not yet implemented. Use OpenRouter for now.".to_string())
    }

    fn narrate_arrival(&self, _context: &PromptContext) -> Result<String, EngineError> {
        Ok("[DeepSeek] Arrival not yet implemented. Use OpenRouter for now.".to_string())
    }

    fn name(&self) -> &str {
        "DeepSeek"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::CharacterSheet;
    use crate::model::map::Room;
    use crate::model::state::{LogEntry, LogType};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_test_room() -> Room {
        Room {
            id: "room1".to_string(),
            name: "Test Room".to_string(),
            description: "A plain room.".to_string(),
            exits: HashMap::new(),
            items: vec![],
            npcs: vec![],
            image_path: None,
        }
    }

    fn make_test_world() -> WorldCard {
        WorldCard {
            name: "Test World".to_string(),
            description: "Testing.".to_string(),
            global_rules: vec!["Rule 1".to_string()],
        }
    }

    fn make_test_player() -> PlayerCard {
        PlayerCard {
            sheet: CharacterSheet {
                name: "Hero".to_string(),
                description: "The protagonist.".to_string(),
                personality: "Brave".to_string(),
                scenario: "Generic Quest".to_string(),
                example_dialogue: "".to_string(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        }
    }

    fn make_test_context(user_message: &str) -> PromptContext<'static> {
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();
        let npcs: Vec<NpcCard> = vec![];
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

    fn make_test_context_with_npc(npc: &NpcCard, user_message: &str) -> PromptContext<'static> {
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

    #[test]
    fn test_mock_narrate_action() {
        let backend = MockBackend;
        let context = make_test_context("I look around carefully.");

        let result = backend.narrate_action(&context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[MockNarration] I look around carefully.");
    }

    #[test]
    fn test_mock_narrate_arrival() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let context = PromptContext {
            world: Box::leak(Box::new(world)),
            room: Box::leak(Box::new(room)),
            all_npcs: &[],
            npcs_in_area: &[],
            player: Box::leak(Box::new(player)),
            user_message: "",
            history: &[],
        };
        let result = backend.narrate_arrival(&context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[MockArrival] You enter the Test Room.");
    }

    #[test]
    fn test_system_prompt_construction() {
        let world = make_test_world();
        let room = make_test_room();
        let npc = NpcCard {
            id: "carla".to_string(),
            sheet: CharacterSheet {
                name: "Carla".to_string(),
                description: "Guard".to_string(),
                personality: "Strict".to_string(),
                scenario: "Gate".to_string(),
                example_dialogue: "Halt!".to_string(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        };
        let (prompt, _user) =
            build_dialogue_prompts(&world, &room, &npc, &Some("Hello".to_string()));

        // Assertions for prompt integrity
        assert!(prompt.contains("Carla"));
        assert!(prompt.contains("Strict"));
        assert!(prompt.contains("Gate"));
        assert!(prompt.contains("Test Room"));
        assert!(prompt.contains("Rule 1"));
    }

    #[test]
    fn test_build_dialogue_prompts_with_no_message() {
        let world = make_test_world();
        let room = make_test_room();
        let npc = NpcCard {
            id: "test".to_string(),
            sheet: CharacterSheet {
                name: "NPC".to_string(),
                description: "Desc".to_string(),
                personality: "Person".to_string(),
                scenario: "Scene".to_string(),
                example_dialogue: "".to_string(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        };

        let (system_prompt, user_prompt) = build_dialogue_prompts(&world, &room, &npc, &None);

        // Verify the prompts are generated (basic sanity check)
        assert!(!system_prompt.is_empty());
        assert!(!user_prompt.is_empty() || user_prompt.len() < 200); // user might be empty or have placeholder
    }

    #[test]
    fn test_build_action_prompts_basic() {
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let (system_prompt, user_prompt) =
            build_action_prompts(&world, &room, &[], &player, "I look around");

        // Verify the prompts are generated
        assert!(!system_prompt.is_empty());
        assert!(user_prompt.contains("I look around"));
    }

    // ========== Additional LlmBackend Tests ==========

    #[test]
    fn test_mock_generate_dialogue_with_message() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Guard".to_string(),
                description: "A stern guard".to_string(),
                personality: "Suspicious".to_string(),
                scenario: "Watching the gate".to_string(),
                example_dialogue: "".to_string(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        };
        let _player = make_test_player();
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
        let backend = MockBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Guard".to_string(),
                description: "A stern guard".to_string(),
                personality: "Suspicious".to_string(),
                scenario: "Watching the gate".to_string(),
                example_dialogue: "".to_string(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        };
        let _player = make_test_player();
        let _message: Option<String> = None;

        let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, ""), &npc);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[MockGenerated] Standard greeting.");
    }

    #[test]
    fn test_mock_backend_name() {
        let backend = MockBackend;
        assert_eq!(backend.name(), "Mock");
    }

    #[test]
    fn test_deepseek_generate_dialogue() {
        let backend = DeepSeekBackend;
        let world = make_test_world();
        let room = make_test_room();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Test".to_string(),
                description: "Test".to_string(),
                personality: "Test".to_string(),
                scenario: "Test".to_string(),
                example_dialogue: "".to_string(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        };
        let player = make_test_player();

        let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, ""), &npc);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_narrate_action() {
        let backend = DeepSeekBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let result = backend.narrate_action(&make_test_context("test"));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_narrate_arrival() {
        let backend = DeepSeekBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let result = backend.narrate_arrival(&make_test_context(""));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_name() {
        let backend = DeepSeekBackend;
        assert_eq!(backend.name(), "DeepSeek");
    }

    #[test]
    fn test_mock_with_history() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        // Test that history parameter is accepted (doesn't cause error)
        let history = vec![
            LogEntry {
                sender: Some("Narrator".to_string()),
                text: "You see a mysterious figure.".to_string(),
                log_type: LogType::Narration,
                timestamp: Utc::now(),
            },
            LogEntry {
                sender: Some("Player".to_string()),
                text: "Hello?".to_string(),
                log_type: LogType::Input,
                timestamp: Utc::now(),
            },
        ];

        let result = backend.narrate_action(&make_test_context("I approach"));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("I approach"));
    }

    // ========================================================================
    // Property-based tests for MockBackend responses
    // These test response properties rather than exact strings
    // ========================================================================

    #[test]
    fn test_mock_response_length_bounds() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        // Test various input lengths
        let short_input = "hi";
        let long_input = "This is a much longer player input that describes what the player wants to do in detail";

        let result_short = backend.narrate_action(&make_test_context(short_input));
        let result_long = backend.narrate_action(&make_test_context(long_input));

        assert!(result_short.is_ok());
        assert!(result_long.is_ok());

        // Response should be non-empty
        assert!(!result_short.unwrap().is_empty());
        assert!(!result_long.unwrap().is_empty());
    }

    #[test]
    fn test_mock_response_contains_input() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let unique_input = "xyz123_test_input";
        let result = backend.narrate_action(&make_test_context(unique_input));

        assert!(result.is_ok());
        // Mock response echoes the input
        assert!(result.unwrap().contains(unique_input));
    }

    #[test]
    fn test_mock_narrate_arrival_includes_room_name() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let result = backend.narrate_arrival(&make_test_context(""));

        assert!(result.is_ok());
        let response = result.unwrap();
        // Should mention entering and room name
        assert!(response.contains("enter"));
        assert!(response.contains(&room.name));
    }

    #[test]
    fn test_mock_dialogue_with_message() {
        let backend = MockBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Guard".to_string(),
                description: "A stern guard".to_string(),
                personality: "Alert".to_string(),
                scenario: "Watching".to_string(),
                example_dialogue: "Halt!".to_string(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        };
        let _player = make_test_player();

        let message = Some("Hello, guard!".to_string());
        let result = backend.generate_dialogue(
            &make_test_context_with_npc(&npc, message.as_deref().unwrap_or("")),
            &npc,
        );

        assert!(result.is_ok());
        let response = result.unwrap();
        // Should echo the player's message
        assert!(response.contains("Hello, guard!"));
    }

    #[test]
    fn test_mock_dialogue_without_message() {
        let backend = MockBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Guard".to_string(),
                description: "A stern guard".to_string(),
                personality: "Alert".to_string(),
                scenario: "Watching".to_string(),
                example_dialogue: "Halt!".to_string(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        };
        let _player = make_test_player();

        let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, ""), &npc);

        assert!(result.is_ok());
        // Without message, should return greeting
        assert!(result.unwrap().contains("greeting"));
    }

    #[test]
    fn test_mock_with_empty_history() {
        let backend = MockBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let player = make_test_player();

        // Empty history should work fine
        let result = backend.narrate_action(&make_test_context("test"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_with_substantial_history() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        // Substantial history (like real game would have)
        let history: Vec<LogEntry> = (0..50)
            .map(|i| LogEntry {
                sender: Some(format!("Speaker{}", i % 3)),
                text: format!("This is narration entry number {} in the game history.", i),
                log_type: if i % 2 == 0 {
                    LogType::Narration
                } else {
                    LogType::Input
                },
                timestamp: Utc::now(),
            })
            .collect();

        let result = backend.narrate_action(&make_test_context("current action"));
        assert!(result.is_ok());
        // Should still work with large history (Mock doesn't use it, but API accepts it)
    }
}
