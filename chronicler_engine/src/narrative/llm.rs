use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::settings::{AppSettings, Connection};
use crate::narrative::llm_client::{call_ollama, call_openrouter_with_model};
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
pub use crate::model::llm_backend::LlmBackendType;

use std::sync::atomic::{AtomicU8, Ordering};

static TEST_BACKEND_OVERRIDE: AtomicU8 = AtomicU8::new(0); // 0=none, 1=mock, 2=deepseek, 3=openrouter, 4=ollama

pub fn set_test_backend(backend: LlmBackendType) {
    let val = match backend {
        LlmBackendType::Mock => 1,
        LlmBackendType::DeepSeek => 2,
        LlmBackendType::OpenRouter => 3,
        LlmBackendType::Ollama => 4,
    };
    TEST_BACKEND_OVERRIDE.store(val, Ordering::SeqCst);
}

pub fn clear_test_backend() {
    TEST_BACKEND_OVERRIDE.store(0, Ordering::SeqCst);
}

pub struct TestBackendGuard;

impl Drop for TestBackendGuard {
    fn drop(&mut self) {
        clear_test_backend();
    }
}

pub fn with_test_backend(backend: LlmBackendType) -> TestBackendGuard {
    set_test_backend(backend);
    TestBackendGuard
}

/// Create an LLM backend for a specific connection.
/// [DOC: docs/system/llm_processing.md]
pub fn get_llm_backend_for(connection: &Connection) -> Box<dyn LlmBackend> {
    match connection.provider {
        LlmBackendType::Mock => Box::new(MockBackend),
        LlmBackendType::DeepSeek => Box::new(DeepSeekBackend::from_connection(connection)),
        LlmBackendType::OpenRouter => Box::new(OpenRouterBackend::from_connection(connection)),
        LlmBackendType::Ollama => Box::new(OllamaBackend::from_connection(connection)),
    }
}

/// Get the LLM backend for the current narration connection.
/// Respects test overrides for backward compatibility with tests.
/// [DOC: docs/system/llm_processing.md]
pub fn get_llm_backend() -> Box<dyn LlmBackend> {
    let override_type = TEST_BACKEND_OVERRIDE.load(Ordering::SeqCst);
    if override_type != 0 {
        let provider = match override_type {
            1 => LlmBackendType::Mock,
            2 => LlmBackendType::DeepSeek,
            3 => LlmBackendType::OpenRouter,
            4 => LlmBackendType::Ollama,
            _ => LlmBackendType::OpenRouter,
        };
        let conn = Connection::new("test-override", "Test Override", provider);
        return get_llm_backend_for(&conn);
    }

    let settings = crate::settings::load_settings().unwrap_or_default();
    let connection = settings
        .get_narration_connection()
        .cloned()
        .unwrap_or_else(|| Connection::new("default", "Default", LlmBackendType::Mock));
    get_llm_backend_for(&connection)
}

/// Backward-compatible helper used by some tests.
pub fn get_llm_backend_with_settings(settings: &AppSettings) -> Box<dyn LlmBackend> {
    let connection = settings
        .get_narration_connection()
        .cloned()
        .unwrap_or_else(|| Connection::new("default", "Default", LlmBackendType::Mock));
    get_llm_backend_for(&connection)
}

/// Merge system and user prompts into a single user message.
/// Used for models that ignore the system role.
pub fn merge_single_user_message(system_prompt: &str, user_text: &str) -> String {
    format!("[SYSTEM]\n{system_prompt}\n\n{user_text}")
}

#[derive(Clone, Default)]
pub struct OpenRouterBackend {
    api_key: String,
    model: String,
    single_user_message: bool,
}

impl OpenRouterBackend {
    fn from_connection(connection: &Connection) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
            single_user_message: connection.single_user_message,
        }
    }

    fn call(&self, system_prompt: &str, user_text: &str) -> Result<String, EngineError> {
        let (system, user) = if self.single_user_message {
            ("", merge_single_user_message(system_prompt, user_text))
        } else {
            (system_prompt, user_text.to_string())
        };
        call_openrouter_with_model(&self.api_key, system, &user, &self.model)
            .map_err(EngineError::Narrative)
    }
}

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

        self.call(&system_prompt, &user_text)
    }

    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError> {
        log::info!(
            "[LLM] Generating action narration for: {}",
            context.user_message
        );

        let builder = PromptBuilder::from_context(context);
        let (system_prompt, user_text) = builder.build_split()?;

        self.call(&system_prompt, &user_text)
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

        self.call(&system_prompt, &user_text)
    }

    fn narrate_continuation(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        _trigger_prompt: &str,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating continuation narration");

        self.call(system_prompt, user_prompt)
    }

    fn narrate_action_from_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating action from prompt");

        self.call(system_prompt, user_prompt)
    }

    fn name(&self) -> &str {
        "OpenRouter"
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct DeepSeekBackend {
    api_key: String,
    model: String,
}

impl DeepSeekBackend {
    fn from_connection(connection: &Connection) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
        }
    }
}

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

#[derive(Clone, Default)]
pub struct OllamaBackend {
    base_url: String,
    model: String,
    single_user_message: bool,
}

impl OllamaBackend {
    fn from_connection(connection: &Connection) -> Self {
        Self {
            base_url: connection.resolve_base_url(),
            model: connection.model.clone(),
            single_user_message: connection.single_user_message,
        }
    }

    fn call(&self, system_prompt: &str, user_text: &str) -> Result<String, EngineError> {
        let (system, user) = if self.single_user_message {
            ("", merge_single_user_message(system_prompt, user_text))
        } else {
            (system_prompt, user_text.to_string())
        };
        call_ollama(&self.base_url, &self.model, system, &user).map_err(EngineError::Narrative)
    }
}

impl LlmBackend for OllamaBackend {
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
        self.call(&system_prompt, &user_text)
    }

    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError> {
        log::info!(
            "[LLM] Generating action narration for: {}",
            context.user_message
        );

        let builder = PromptBuilder::from_context(context);
        let (system_prompt, user_text) = builder.build_split()?;
        self.call(&system_prompt, &user_text)
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
        self.call(&system_prompt, &user_text)
    }

    fn narrate_continuation(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        _trigger_prompt: &str,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating continuation narration");
        self.call(system_prompt, user_prompt)
    }

    fn narrate_action_from_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating action from prompt");
        self.call(system_prompt, user_prompt)
    }

    fn name(&self) -> &str {
        "Ollama"
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
                summary: None,
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
                summary: None,
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
                summary: None,
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
        let backend = DeepSeekBackend::default();
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
                summary: None,
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
        let backend = DeepSeekBackend::default();
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

        let result = backend.narrate_action(&make_test_context("test"));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_narrate_arrival() {
        let backend = DeepSeekBackend::default();
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

        let result = backend.narrate_arrival(&make_test_context(""));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_name() {
        let backend = DeepSeekBackend::default();
        assert_eq!(backend.name(), "DeepSeek");
    }

    #[test]
    fn test_mock_with_history() {
        let backend = MockBackend;
        let _world = make_test_world();
        let _room = make_test_room();
        let _player = make_test_player();

        let _history = [
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
                summary: None,
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
                summary: None,
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
                text: format!("This is narration entry number {i} in the game history."),
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
        let backend = DeepSeekBackend::default();
        assert_eq!(backend.name(), "DeepSeek");
    }

    fn make_settings_with_provider(provider: LlmBackendType) -> AppSettings {
        let conn = Connection::new("test-conn", "Test", provider);
        AppSettings {
            connections: vec![conn],
            narration_connection_id: "test-conn".into(),
            quantifier_connection_id: "test-conn".into(),
        }
    }

    #[test]
    fn test_get_llm_backend_with_settings_all_types() {
        let openrouter_settings = make_settings_with_provider(LlmBackendType::OpenRouter);
        assert_eq!(
            get_llm_backend_with_settings(&openrouter_settings).name(),
            "OpenRouter"
        );

        let mock_settings = make_settings_with_provider(LlmBackendType::Mock);
        assert_eq!(get_llm_backend_with_settings(&mock_settings).name(), "Mock");

        let deepseek_settings = make_settings_with_provider(LlmBackendType::DeepSeek);
        assert_eq!(
            get_llm_backend_with_settings(&deepseek_settings).name(),
            "DeepSeek"
        );

        let ollama_settings = make_settings_with_provider(LlmBackendType::Ollama);
        assert_eq!(
            get_llm_backend_with_settings(&ollama_settings).name(),
            "Ollama"
        );
    }

    #[test]
    fn test_ollama_backend_name() {
        let backend = OllamaBackend::default();
        assert_eq!(backend.name(), "Ollama");
    }

    #[test]
    fn test_llm_backend_type_from_env_default() {
        assert_eq!(LlmBackendType::from_env(), LlmBackendType::OpenRouter);
    }

    #[test]
    fn test_mock_narrate_continuation() {
        let backend = MockBackend;
        let result = backend.narrate_continuation("system", "user", "trigger_info");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("trigger_info"));
    }

    #[test]
    fn test_mock_narrate_action_from_prompt() {
        let backend = MockBackend;
        let result = backend.narrate_action_from_prompt("system prompt", "user action");
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("action") || response.contains("Continuation"));
    }

    #[test]
    fn test_deepseek_narrate_continuation() {
        let backend = DeepSeekBackend::default();
        let result = backend.narrate_continuation("system", "user", "trigger");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_narrate_action_from_prompt() {
        let backend = DeepSeekBackend::default();
        let result = backend.narrate_action_from_prompt("system", "user");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_deepseek_error_message_descriptive() {
        let backend = DeepSeekBackend::default();
        let result = backend.narrate_action(&make_test_context("test"));
        assert!(result.unwrap().contains("DeepSeek"));
    }

    #[test]
    fn test_mock_narrate_continuation_empty_trigger() {
        let backend = MockBackend;
        let result = backend.narrate_continuation("system", "user", "");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("[Trigger: ]"));
    }

    #[test]
    fn test_mock_narrate_continuation_special_chars() {
        let backend = MockBackend;
        let result = backend.narrate_continuation("sys", "user", "trigger with <special> & chars");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("trigger with"));
    }

    #[test]
    fn test_mock_narrate_action_from_prompt_multiline() {
        let backend = MockBackend;
        let result = backend.narrate_action_from_prompt(
            "system prompt\nwith multiple lines",
            "user prompt\nalso multiline",
        );
        assert!(result.is_ok());
        assert!(result.unwrap().contains("user prompt"));
    }

    #[test]
    fn test_mock_narrate_action_from_prompt_empty() {
        let backend = MockBackend;
        let result = backend.narrate_action_from_prompt("", "");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("..."));
    }

    #[test]
    fn test_mock_generate_dialogue_very_long_message() {
        let backend = MockBackend;
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
        };
        let result =
            backend.generate_dialogue(&make_test_context_with_npc(&npc, long_message), &npc);
        assert!(result.is_ok());
        assert!(result.unwrap().contains(long_message));
    }

    #[test]
    fn test_mock_narrate_action_special_characters() {
        let backend = MockBackend;
        let special_msg = "Player says: \"Hello <world> & goodbye!\"";
        let result = backend.narrate_action(&make_test_context(special_msg));
        assert!(result.is_ok());
        assert!(result.unwrap().contains(special_msg));
    }

    #[test]
    fn test_mock_narrate_arrival_different_rooms() {
        let backend = MockBackend;

        let mut room1 = make_test_room();
        room1.name = "Tavern".to_string();
        let world = make_test_world();
        let player = make_test_player();
        let context1 = PromptContext {
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
        let context2 = PromptContext {
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
    fn test_deepseek_all_methods_return_not_implemented() {
        let backend = DeepSeekBackend::default();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Test".to_string(),
                description: "Test".to_string(),
                personality: "Test".to_string(),
                scenario: "Test".to_string(),
                example_dialogue: "".to_string(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        };

        let dialogue_result =
            backend.generate_dialogue(&make_test_context_with_npc(&npc, "test"), &npc);
        assert!(dialogue_result.unwrap().contains("not yet implemented"));

        let action_result = backend.narrate_action(&make_test_context("test"));
        assert!(action_result.unwrap().contains("not yet implemented"));

        let arrival_result = backend.narrate_arrival(&make_test_context("test"));
        assert!(arrival_result.unwrap().contains("not yet implemented"));

        let continuation_result = backend.narrate_continuation("sys", "user", "trigger");
        assert!(continuation_result.unwrap().contains("not yet implemented"));

        let prompt_result = backend.narrate_action_from_prompt("sys", "user");
        assert!(prompt_result.unwrap().contains("not yet implemented"));
    }

    #[test]
    fn test_context_with_different_player_names() {
        let backend = MockBackend;

        let player1 = make_test_player();
        let mut room1 = make_test_room();
        room1.name = "Tavern".to_string();
        let world = make_test_world();

        let context1 = PromptContext {
            world: Box::leak(Box::new(world.clone())),
            room: Box::leak(Box::new(room1.clone())),
            all_npcs: Box::leak(Box::new(vec![])),
            npcs_in_area: &[],
            player: Box::leak(Box::new(player1)),
            user_message: Box::leak(Box::new("test".to_string())),
            history: &[],
        };
        let result1 = backend.narrate_arrival(&context1);
        assert!(result1.is_ok());
        // Mock arrival includes room name
        assert!(result1.unwrap().contains("Tavern"));

        let mut room2 = make_test_room();
        room2.name = "Dungeon".to_string();
        let player2 = make_test_player();
        let context2 = PromptContext {
            world: Box::leak(Box::new(world)),
            room: Box::leak(Box::new(room2)),
            all_npcs: Box::leak(Box::new(vec![])),
            npcs_in_area: &[],
            player: Box::leak(Box::new(player2)),
            user_message: Box::leak(Box::new("test".to_string())),
            history: &[],
        };
        let result2 = backend.narrate_arrival(&context2);
        assert!(result2.is_ok());
        assert!(result2.unwrap().contains("Dungeon"));
    }

    #[test]
    fn test_mock_dialogue_with_unicode() {
        let backend = MockBackend;
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
        };
        let result =
            backend.generate_dialogue(&make_test_context_with_npc(&npc, "こんにちは"), &npc);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("こんにちは"));
    }

    #[test]
    fn test_mock_narrate_action_unicode() {
        let backend = MockBackend;
        let result = backend.narrate_action(&make_test_context("アクション"));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("アクション"));
    }

    #[test]
    fn test_mock_narrate_continuation_unicode_trigger() {
        let backend = MockBackend;
        let result = backend.narrate_continuation("system", "user", "トリガー");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("トリガー"));
    }

    #[test]
    fn test_context_with_empty_world_description() {
        let backend = MockBackend;
        let world = WorldCard {
            name: "Empty World".to_string(),
            description: "".to_string(),
            global_rules: vec![],
            ..Default::default()
        };
        let room = make_test_room();
        let player = make_test_player();

        let context = PromptContext {
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
        let backend = MockBackend;
        let world = WorldCard {
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
        let player = make_test_player();

        let context = PromptContext {
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
        let backend = MockBackend;
        let long_system = "You are a game master. ".repeat(50);
        let long_user = "The player performs an action. ".repeat(50);
        let result = backend.narrate_action_from_prompt(&long_system, &long_user);
        assert!(result.is_ok());
        // Should still contain first line
        assert!(result.unwrap().contains("The player performs"));
    }

    #[test]
    fn test_npc_with_no_triggers() {
        let backend = MockBackend;
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
        };
        let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, "Hello"), &npc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_npc_with_multiple_triggers() {
        use crate::model::trigger::{ComparisonOperator, Trigger, TriggerAction, TriggerCondition};
        let backend = MockBackend;
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
        };
        let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, "Test"), &npc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_player_with_empty_inventory() {
        let backend = MockBackend;
        let player = PlayerCard {
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

        let context = PromptContext {
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
    fn test_player_with_items_in_inventory() {
        let backend = MockBackend;
        let player = PlayerCard {
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

        let context = PromptContext {
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
        let backend = MockBackend;
        let room = Room {
            id: "room_with_items".to_string(),
            name: "Storage Room".to_string(),
            description: "A room full of items".to_string(),
            exits: HashMap::new(),
            items: vec![
                "Chest".to_string(),
                "Barrel".to_string(),
                "Table".to_string(),
            ],
            npcs: vec![],
            image_path: None,
            navigation_description: None,
        };
        let world = make_test_world();
        let player = make_test_player();

        let context = PromptContext {
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
        let backend = MockBackend;
        let mut exits = HashMap::new();
        exits.insert(Direction::North, "hallway".to_string());
        exits.insert(Direction::East, "kitchen".to_string());
        exits.insert(Direction::South, "garden".to_string());

        let room = Room {
            id: "room_with_exits".to_string(),
            name: "Central Room".to_string(),
            description: "A central room with many exits".to_string(),
            exits,
            items: vec![],
            npcs: vec![],
            image_path: None,
            navigation_description: None,
        };
        let world = make_test_world();
        let player = make_test_player();

        let context = PromptContext {
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
        let backend = MockBackend;
        let world = WorldCard {
            name: "World with Image".to_string(),
            description: "A world with default room image".to_string(),
            global_rules: vec!["Rule".to_string()],
            default_room_image: Some("default_room.png".to_string()),
        };
        let room = make_test_room();
        let player = make_test_player();

        let context = PromptContext {
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
        let backend = MockBackend;
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
        };
        let result =
            backend.generate_dialogue(&make_test_context_with_npc(&npc, "What do you sell?"), &npc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_context_with_npcs_in_area() {
        let backend = MockBackend;
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
        };

        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();
        let all_npcs = vec![npc1.clone(), npc2.clone()];
        let npcs_in_area = vec![npc1.clone()];

        let context = PromptContext {
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

    #[test]
    fn test_merge_single_user_message_format() {
        let merged = merge_single_user_message("system content", "user content");
        assert!(merged.starts_with("[SYSTEM]\n"));
        assert!(merged.contains("system content"));
        assert!(merged.contains("user content"));
        // System content should come before user content
        let system_pos = merged.find("system content").unwrap();
        let user_pos = merged.find("user content").unwrap();
        assert!(system_pos < user_pos);
    }

    #[test]
    fn test_merge_single_user_message_preserves_multiline() {
        let system = "Line 1\nLine 2";
        let user = "User Line 1\nUser Line 2";
        let merged = merge_single_user_message(system, user);
        assert!(merged.contains("Line 1\nLine 2"));
        assert!(merged.contains("User Line 1\nUser Line 2"));
    }

    #[test]
    fn test_merge_single_user_message_empty_system() {
        let merged = merge_single_user_message("", "user content");
        assert_eq!(merged, "[SYSTEM]\n\n\nuser content");
    }

    #[test]
    fn test_merge_single_user_message_empty_user() {
        let merged = merge_single_user_message("system content", "");
        assert_eq!(merged, "[SYSTEM]\nsystem content\n\n");
    }

    #[test]
    fn test_merge_single_user_message_both_empty() {
        let merged = merge_single_user_message("", "");
        assert_eq!(merged, "[SYSTEM]\n\n\n");
    }
}
