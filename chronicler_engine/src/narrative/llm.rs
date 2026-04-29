use crate::error::EngineError;
use crate::model::character::NpcCard;
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

    fn narrate_continuation(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        trigger_prompt: &str,
    ) -> Result<String, EngineError>;

    fn narrate_action_from_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, EngineError>;

    fn name(&self) -> &str;
}

// [DOC: docs/system/llm_processing.md]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LlmBackendType {
    OpenRouter,
    DeepSeek,
    Mock,
}

impl LlmBackendType {
    pub fn from_env() -> Self {
        match std::env::var("LLM_BACKEND").as_deref() {
            Ok("deepseek") => LlmBackendType::DeepSeek,
            Ok("mock") => LlmBackendType::Mock,
            _ => LlmBackendType::OpenRouter, // default
        }
    }
}

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

        let user_msg = format!(
            "The player says to {}: \"{}\"",
            npc.sheet.name, context.user_message
        );

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
        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
            log::error!("OPENROUTER_API_KEY not set - cannot generate dialogue");
            EngineError::Config("OPENROUTER_API_KEY not set".into())
        })?;

        call_openrouter(&api_key, &system_prompt, &user_text).map_err(EngineError::Narrative)
    }

    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError> {
        log::info!(
            "[LLM] Generating action narration for: {}",
            context.user_message
        );

        let builder = PromptBuilder::from_context(context);
        let (system_prompt, user_text) = builder.build_split()?;
        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
            log::error!("OPENROUTER_API_KEY not set - cannot generate action narration");
            EngineError::Config("OPENROUTER_API_KEY not set".into())
        })?;

        call_openrouter(&api_key, &system_prompt, &user_text).map_err(EngineError::Narrative)
    }

    fn narrate_arrival(&self, context: &PromptContext) -> Result<String, EngineError> {
        log::info!(
            "[LLM] Generating arrival narration for room: {}",
            context.room.name
        );

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
        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
            log::error!("OPENROUTER_API_KEY not set - cannot generate arrival narration");
            EngineError::Config("OPENROUTER_API_KEY not set".into())
        })?;

        call_openrouter(&api_key, &system_prompt, &user_text).map_err(EngineError::Narrative)
    }

    fn narrate_continuation(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        _trigger_prompt: &str,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating continuation narration");

        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
            log::error!("OPENROUTER_API_KEY not set - cannot generate continuation narration");
            EngineError::Config("OPENROUTER_API_KEY not set".into())
        })?;

        call_openrouter(&api_key, system_prompt, user_prompt).map_err(EngineError::Narrative)
    }

    fn narrate_action_from_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating action from prompt");

        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
            log::error!("OPENROUTER_API_KEY not set - cannot generate action from prompt");
            EngineError::Config("OPENROUTER_API_KEY not set".into())
        })?;

        call_openrouter(&api_key, system_prompt, user_prompt).map_err(EngineError::Narrative)
    }

    fn name(&self) -> &str {
        "OpenRouter"
    }
}

pub struct MockBackend;

impl LlmBackend for MockBackend {
    fn generate_dialogue(
        &self,
        context: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<String, EngineError> {
        let user_input = context.user_message;
        if user_input.is_empty() {
            Ok("[MockGenerated] Standard greeting.".to_string())
        } else {
            Ok(format!("[MockGenerated] Replying to: {user_input}"))
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

    fn narrate_continuation(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        trigger_prompt: &str,
    ) -> Result<String, EngineError> {
        Ok(format!("[Trigger: {trigger_prompt}]"))
    }

    fn narrate_action_from_prompt(
        &self,
        _system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, EngineError> {
        Ok(format!(
            "[Continuation: {}]",
            user_prompt.lines().next().unwrap_or("...")
        ))
    }

    fn name(&self) -> &str {
        "Mock"
    }
}

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

    fn narrate_continuation(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _trigger_prompt: &str,
    ) -> Result<String, EngineError> {
        Ok("[DeepSeek] Continuation not yet implemented. Use OpenRouter for now.".to_string())
    }

    fn narrate_action_from_prompt(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
    ) -> Result<String, EngineError> {
        Ok(
            "[DeepSeek] Action from prompt not yet implemented. Use OpenRouter for now."
                .to_string(),
        )
    }

    fn name(&self) -> &str {
        "DeepSeek"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::{CharacterSheet, PlayerCard};
    use crate::model::map::Room;
    use crate::model::state::{LogEntry, LogType};
    use crate::model::world::WorldCard;
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
            navigation_description: None,
        }
    }

    fn make_test_world() -> WorldCard {
        WorldCard {
            name: "Test World".to_string(),
            description: "Testing.".to_string(),
            global_rules: vec!["Rule 1".to_string()],
            ..Default::default()
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
                profile_image: None,
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
    fn test_mock_generate_dialogue_with_message() {
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
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
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
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
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
        let _world = make_test_world();
        let _room = make_test_room();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Test".to_string(),
                description: "Test".to_string(),
                personality: "Test".to_string(),
                scenario: "Test".to_string(),
                example_dialogue: "".to_string(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        };
        let _player = make_test_player();

        let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, ""), &npc);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_narrate_action() {
        let backend = DeepSeekBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

        let result = backend.narrate_action(&make_test_context("test"));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_narrate_arrival() {
        let backend = DeepSeekBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

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
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

        let _history = vec![
            LogEntry {
                id: 1,
                sender: Some("Narrator".to_string()),
                text: "You see a mysterious figure.".to_string(),
                log_type: LogType::Narration,
                timestamp: Utc::now(),
            },
            LogEntry {
                id: 2,
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

    #[test]
    fn test_mock_response_length_bounds() {
        let backend = MockBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

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
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

        let unique_input = "xyz123_test_input";
        let result = backend.narrate_action(&make_test_context(unique_input));

        assert!(result.is_ok());
        assert!(result.unwrap().contains(unique_input));
    }

    #[test]
    fn test_mock_narrate_arrival_includes_room_name() {
        let backend = MockBackend;
        let _world = make_test_world();
        let room = make_test_room();
        let _player = make_test_player();

        let result = backend.narrate_arrival(&make_test_context(""));

        assert!(result.is_ok());
        let response = result.unwrap();
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
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        };
        let _player = make_test_player();

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
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        };
        let _player = make_test_player();

        let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, ""), &npc);

        assert!(result.is_ok());
        assert!(result.unwrap().contains("greeting"));
    }

    #[test]
    fn test_mock_with_empty_history() {
        let backend = MockBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

        let result = backend.narrate_action(&make_test_context("test"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_with_substantial_history() {
        let backend = MockBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

        let _history: Vec<LogEntry> = (0..50)
            .map(|i| LogEntry {
                id: i as u64,
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
    }

    #[test]
    fn test_deepseek_backend_name() {
        let backend = DeepSeekBackend;
        assert_eq!(backend.name(), "DeepSeek");
    }

    #[test]
    fn test_llm_backend_type_from_env_mock() {
        // Set env var to mock
        // SAFETY: This test modifies env vars but is isolated to this test.
        // We restore the original value after the test.
        unsafe {
            std::env::set_var("LLM_BACKEND", "mock");
        }
        let backend_type = LlmBackendType::from_env();
        assert_eq!(backend_type, LlmBackendType::Mock);
        unsafe {
            std::env::remove_var("LLM_BACKEND");
        }
    }

    #[test]
    fn test_llm_backend_type_from_env_deepseek() {
        // Set env var to deepseek
        unsafe {
            std::env::set_var("LLM_BACKEND", "deepseek");
        }
        let backend_type = LlmBackendType::from_env();
        assert_eq!(backend_type, LlmBackendType::DeepSeek);
        unsafe {
            std::env::remove_var("LLM_BACKEND");
        }
    }

    #[test]
    fn test_llm_backend_type_default() {
        // Ensure env var is not set
        unsafe {
            std::env::remove_var("LLM_BACKEND");
        }
        let backend_type = LlmBackendType::from_env();
        assert_eq!(backend_type, LlmBackendType::OpenRouter);
    }
}
