use crate::error::EngineError;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::Room;
use crate::model::world::WorldCard;
use serde_json::json;

pub trait LlmBackend: Send + Sync {
    fn generate_dialogue(
        &self,
        world: &WorldCard,
        room: &Room,
        npc: &NpcCard,
        user_message: &Option<String>,
    ) -> Result<String, EngineError>;

    fn narrate_action(
        &self,
        world: &WorldCard,
        room: &Room,
        nearby_npcs: &[&NpcCard],
        player: &PlayerCard,
        player_input: &str,
    ) -> Result<String, EngineError>;

    fn narrate_arrival(
        &self,
        world: &WorldCard,
        room: &Room,
        nearby_npcs: &[&NpcCard],
        player: &PlayerCard,
    ) -> Result<String, EngineError>;

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
        world: &WorldCard,
        room: &Room,
        npc: &NpcCard,
        user_message: &Option<String>,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating dialogue for NPC: {}", npc.sheet.name);
        let (system_prompt, user_text) = build_dialogue_prompts(world, room, npc, user_message);
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| EngineError::Config("OPENROUTER_API_KEY not set".into()))?;

        call_openrouter(&api_key, &system_prompt, &user_text).map_err(EngineError::Narrative)
    }

    fn narrate_action(
        &self,
        world: &WorldCard,
        room: &Room,
        nearby_npcs: &[&NpcCard],
        player: &PlayerCard,
        player_input: &str,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating action narration for: {player_input}");
        let (system_prompt, user_text) =
            build_action_prompts(world, room, nearby_npcs, player, player_input);
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| EngineError::Config("OPENROUTER_API_KEY not set".into()))?;

        call_openrouter(&api_key, &system_prompt, &user_text).map_err(EngineError::Narrative)
    }

    fn narrate_arrival(
        &self,
        world: &WorldCard,
        room: &Room,
        nearby_npcs: &[&NpcCard],
        player: &PlayerCard,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating arrival narration for room: {}", room.name);
        let (system_prompt, user_text) = build_arrival_prompts(world, room, nearby_npcs, player);
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| EngineError::Config("OPENROUTER_API_KEY not set".into()))?;

        let result = call_openrouter(&api_key, &system_prompt, &user_text);
        match result {
            Ok(text) => Ok(text),
            Err(err) => {
                log::error!("[LLM] Arrival narration failed: {err}");
                Err(EngineError::Narrative(err))
            }
        }
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

pub fn build_arrival_prompts(
    world: &WorldCard,
    room: &Room,
    nearby_npcs: &[&NpcCard],
    player: &PlayerCard,
) -> (String, String) {
    let mut system_prompt = String::from(
        "You are the Game Master of a text adventure game. \
The player has just arrived at a new location. \
Narrate their arrival and describe the immediate scene. \
Voice any NPCs present if they would logically greet or react to the player's entrance. \
Keep responses immersive, concise, and in the style of literary fiction. \
Never act or speak on behalf of the player.\n\n",
    );

    system_prompt.push_str(&format!(
        "Player Identity:\n- Name: {}\n- Persona: {}\n- Background: {}\n\n",
        player.sheet.name, player.sheet.personality, player.sheet.scenario
    ));

    system_prompt.push_str(&format!(
        "New Location: {} — {}\n\n",
        room.name, room.description
    ));

    if !nearby_npcs.is_empty() {
        system_prompt.push_str("Characters already here:\n");
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

    let user_text = format!("{} enters the {}.", player.sheet.name, room.name);

    (system_prompt, user_text)
}

fn call_openrouter(api_key: &str, system_prompt: &str, user_text: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "z-ai/glm-4.5-air:free".to_string());

    log::info!("[LLM] Using model: {model}");
    log::debug!("[LLM] System prompt: {system_prompt}");
    log::debug!("[LLM] User text: {user_text}");

    let payload = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": user_text
            }
        ]
    });

    let res = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&payload)
        .send();

    match res {
        Ok(response) => {
            let status = response.status();
            log::info!("[LLM] Response status: {status}");

            if !status.is_success() {
                log::error!("[LLM] Non-success HTTP status: {status}");
                return Err(format!("Error communicating with OpenRouter: {status}"));
            }

            // Try to parse JSON response - get raw text first to log on failure
            let raw_response = response.text().map_err(|e| {
                log::error!("[LLM] Failed to read response body: {e}");
                format!("Failed to read response body: {e}")
            })?;

            match serde_json::from_str::<serde_json::Value>(&raw_response) {
                Ok(json_response) => {
                    log::debug!("[LLM] Raw JSON response: {json_response:?}");

                    // Check for API-level errors in response
                    if let Some(error) = json_response.get("error") {
                        let error_msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown API error");
                        log::error!("[LLM] API error: {error_msg}");
                        return Err(format!("LLM API error: {error_msg}"));
                    }

                    // Try to extract content from the standard response format
                    if let Some(content) = json_response["choices"]
                        .get(0)
                        .and_then(|c| c.get("message"))
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_str())
                    {
                        log::info!(
                            "[LLM] Successfully parsed content ({} chars)",
                            content.len()
                        );
                        return Ok(content.to_string());
                    }

                    // If we got here, the response structure was unexpected
                    log::error!("[LLM] Parse error: Could not find content in response structure");
                    log::error!(
                        "[LLM] Response had keys: {:?}",
                        json_response
                            .as_object()
                            .map(|m| m.keys().cloned().collect::<Vec<_>>())
                            .unwrap_or_default()
                    );
                    Err("The world seems to hold its breath (parse error).".to_string())
                }
                Err(e) => {
                    log::error!("[LLM] JSON parse error: {e}");
                    log::error!("[LLM] Raw response that failed to parse: {raw_response}");
                    Err(format!("Failed to parse LLM response: {e}"))
                }
            }
        }
        Err(e) => {
            log::error!("[LLM] Request failed: {e}");
            Err(format!("Request failed: {e}"))
        }
    }
}

pub struct MockBackend;

impl LlmBackend for MockBackend {
    fn generate_dialogue(
        &self,
        _world: &WorldCard,
        _room: &Room,
        _npc: &NpcCard,
        user_message: &Option<String>,
    ) -> Result<String, EngineError> {
        Ok(match user_message {
            Some(msg) => format!("[MockGenerated] Replying to: {msg}"),
            None => "[MockGenerated] Standard greeting.".to_string(),
        })
    }

    fn narrate_action(
        &self,
        _world: &WorldCard,
        _room: &Room,
        _nearby_npcs: &[&NpcCard],
        _player: &PlayerCard,
        player_input: &str,
    ) -> Result<String, EngineError> {
        Ok(format!("[MockNarration] {player_input}"))
    }

    fn narrate_arrival(
        &self,
        _world: &WorldCard,
        room: &Room,
        _nearby_npcs: &[&NpcCard],
        _player: &PlayerCard,
    ) -> Result<String, EngineError> {
        Ok(format!("[MockArrival] You enter the {}.", room.name))
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
        _world: &WorldCard,
        _room: &Room,
        _npc: &NpcCard,
        _user_message: &Option<String>,
    ) -> Result<String, EngineError> {
        Ok("[DeepSeek] Dialogue not yet implemented. Use OpenRouter for now.".to_string())
    }

    fn narrate_action(
        &self,
        _world: &WorldCard,
        _room: &Room,
        _nearby_npcs: &[&NpcCard],
        _player: &PlayerCard,
        _player_input: &str,
    ) -> Result<String, EngineError> {
        Ok("[DeepSeek] Narration not yet implemented. Use OpenRouter for now.".to_string())
    }

    fn narrate_arrival(
        &self,
        _world: &WorldCard,
        _room: &Room,
        _nearby_npcs: &[&NpcCard],
        _player: &PlayerCard,
    ) -> Result<String, EngineError> {
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
            },
            inventory: vec![],
        }
    }

    #[test]
    fn test_mock_narrate_action() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let result =
            backend.narrate_action(&world, &room, &[], &player, "I look around carefully.");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[MockNarration] I look around carefully.");
    }

    #[test]
    fn test_mock_narrate_arrival() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let result = backend.narrate_arrival(&world, &room, &[], &player);
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
    fn test_arrival_prompt_construction() {
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
            },
            inventory: vec![],
        };
        let player = make_test_player();
        let (prompt, user) = build_arrival_prompts(&world, &room, &[&npc], &player);

        assert!(prompt.contains("Characters already here:"));
        assert!(prompt.contains("Carla"));
        assert!(user.contains("Hero enters the Test Room."));
    }
}
