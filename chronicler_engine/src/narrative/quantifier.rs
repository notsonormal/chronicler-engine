use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::map::Room;
use crate::model::state::LogEntry;
use crate::narrative::openrouter_client::{call_openrouter_with_model, get_quantifier_model};
use serde::Deserialize;

/// Confidence level of the quantifier's NPC presence detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuantifierConfidence {
    /// JSON parsed successfully and all NPC IDs are valid.
    High,
    /// Text fallback was used; some valid NPC IDs were found.
    Medium,
    /// No valid NPC IDs could be extracted; fallback data should be used.
    Low,
}

/// Result of quantifying which NPCs are in a room.
#[derive(Debug, Clone)]
pub struct QuantifierParseResult {
    /// The NPC IDs detected as present in the room.
    pub npc_ids: Vec<String>,
    /// How confident the quantifier is in this result.
    pub confidence: QuantifierConfidence,
}

/// Context needed to build a quantifier prompt.
pub struct QuantifierPromptContext<'a> {
    pub room: &'a Room,
    pub previous_room_npcs: &'a [NpcCard],
    pub all_known_npcs: &'a [NpcCard],
    pub player_name: &'a str,
    pub recent_history: &'a [LogEntry],
    pub player_action: &'a str,
}

/// Builds compact prompts for the scene quantifier LLM.
/// Produces a minimal context focused on determining room occupancy.
pub struct QuantifierPromptBuilder<'a> {
    context: QuantifierPromptContext<'a>,
}

impl<'a> QuantifierPromptBuilder<'a> {
    pub fn new(context: QuantifierPromptContext<'a>) -> Self {
        Self { context }
    }

    pub fn build(&self) -> (String, String) {
        (self.build_system_prompt(), self.build_user_prompt())
    }

    fn build_system_prompt(&self) -> String {
        let mut prompt = String::from(
            "You are a scene quantifier for a text adventure game. \
             Your task is to determine which NPCs (non-player characters) \
             are present in the current room based on the context provided.\n\n\
             Rules:\n\
             - Only include NPCs that would logically be in the room\n\
             - NPCs from the previous room may have followed the player\n\
             - Use the exact NPC IDs provided in the character list\n\
             - Respond ONLY with a JSON object in this format: \
             {\"npcs_in_room\": [\"id1\", \"id2\"]}\n\
             - If no NPCs are present, respond with: {\"npcs_in_room\": []}\n\n",
        );

        prompt.push_str("Known NPC IDs:\n");
        for npc in self.context.all_known_npcs {
            prompt.push_str(&format!("- {} (\"{}\")\n", npc.id, npc.sheet.name));
        }

        prompt
    }

    fn build_user_prompt(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str(&format!(
            "Current room: {} — {}\n\n",
            self.context.room.name, self.context.room.description
        ));

        if !self.context.previous_room_npcs.is_empty() {
            prompt.push_str("NPCs from previous room (may have followed the player):\n");
            for npc in self.context.previous_room_npcs {
                prompt.push_str(&format!(
                    "- {} ({}): {}\n",
                    npc.id, npc.sheet.name, npc.sheet.description
                ));
            }
            prompt.push('\n');
        }

        if !self.context.room.npcs.is_empty() {
            prompt.push_str("NPCs configured for this room: ");
            prompt.push_str(&self.context.room.npcs.join(", "));
            prompt.push_str("\n\n");
        }

        if !self.context.recent_history.is_empty() {
            prompt.push_str("Recent events:\n");
            for entry in self.context.recent_history {
                let sender = entry.sender.as_deref().unwrap_or("Narrator");
                prompt.push_str(&format!("{}: {}\n", sender, entry.text));
            }
            prompt.push('\n');
        }

        prompt.push_str(&format!(
            "{}: {}\n\n",
            self.context.player_name, self.context.player_action
        ));

        prompt.push_str("Which NPCs are in this room? Respond with JSON only.");

        prompt
    }
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

/// Serde struct for deserializing the expected JSON response.
#[derive(Deserialize, Debug)]
struct QuantifierJsonResponse {
    #[serde(default)]
    npcs_in_room: Vec<String>,
}

/// Parse the quantifier LLM response to extract NPC IDs.
///
/// Strategy:
/// 1. Try to parse as JSON `{"npcs_in_room": ["id1", "id2"]}`
/// 2. If JSON fails, fall back to text extraction by matching known NPC IDs
/// 3. Validate all extracted IDs against the known NPC list
/// 4. Return results with confidence level
pub fn parse_quantifier_response(
    response: &str,
    known_npc_ids: &[String],
) -> QuantifierParseResult {
    // Normalize: trim whitespace and extract JSON if embedded in markdown
    let trimmed = response.trim();

    // Strategy 1: Try JSON parse
    if let Ok(parsed) = try_parse_json(trimmed) {
        let valid_ids: Vec<String> = parsed
            .into_iter()
            .filter(|id| known_npc_ids.contains(id))
            .collect();

        // JSON parsed successfully - this is High confidence even if the
        // result is empty (the LLM correctly determined no NPCs are present)
        return QuantifierParseResult {
            npc_ids: valid_ids,
            confidence: QuantifierConfidence::High,
        };
    }

    // Strategy 2: Text fallback - match known NPC IDs in the response
    let text_ids = extract_npc_ids_from_text(trimmed, known_npc_ids);

    if !text_ids.is_empty() {
        return QuantifierParseResult {
            npc_ids: text_ids,
            confidence: QuantifierConfidence::Medium,
        };
    }

    // Strategy 3: No valid IDs found
    QuantifierParseResult {
        npc_ids: Vec::new(),
        confidence: QuantifierConfidence::Low,
    }
}

/// Try to parse the response as JSON, handling markdown code fences.
fn try_parse_json(response: &str) -> Result<Vec<String>, String> {
    // Try direct JSON parse first
    if let Ok(parsed) = serde_json::from_str::<QuantifierJsonResponse>(response) {
        return Ok(parsed.npcs_in_room);
    }

    // Try extracting JSON from markdown code fences (```json ... ```)
    if let Some(json_content) = extract_json_from_code_fence(response) {
        if let Ok(parsed) = serde_json::from_str::<QuantifierJsonResponse>(&json_content) {
            return Ok(parsed.npcs_in_room);
        }
    }

    // Try finding JSON object in the response
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            let json_str = &response[start..=end];
            if let Ok(parsed) = serde_json::from_str::<QuantifierJsonResponse>(json_str) {
                return Ok(parsed.npcs_in_room);
            }
        }
    }

    Err("Failed to parse JSON".to_string())
}

/// Extract JSON content from markdown code fences.
fn extract_json_from_code_fence(response: &str) -> Option<String> {
    // Handle ```json ... ``` blocks
    let start_marker = "```json";
    let end_marker = "```";

    let start_idx = response.find(start_marker)?;
    let content_start = start_idx + start_marker.len();
    let remaining = &response[content_start..];

    let end_idx = remaining.find(end_marker)?;
    Some(remaining[..end_idx].trim().to_string())
}

/// Extract NPC IDs from natural language text by matching against known IDs.
fn extract_npc_ids_from_text(response: &str, known_npc_ids: &[String]) -> Vec<String> {
    let response_lower = response.to_lowercase();
    let mut found = Vec::new();

    for id in known_npc_ids {
        // Check if the NPC ID appears in the response
        if response_lower.contains(&id.to_lowercase()) {
            found.push(id.clone());
        }
    }

    found
}

// ---------------------------------------------------------------------------
// QuantifierBackend
// ---------------------------------------------------------------------------

/// Backend for calling the quantifier LLM.
///
/// Uses a separate model (configured via `QUANTIFIER_MODEL` env var)
/// from the main narrative generator.
pub struct QuantifierBackend;

impl QuantifierBackend {
    /// Quantify which NPCs are in the current room using LLM inference.
    ///
    /// Returns a `QuantifierParseResult` with detected NPC IDs and confidence level.
    /// If the LLM call fails entirely, returns the `fallback_npc_ids` with `Low` confidence.
    pub fn quantify_room(
        &self,
        api_key: &str,
        context: &QuantifierPromptContext,
        fallback_npc_ids: &[String],
    ) -> Result<QuantifierParseResult, EngineError> {
        let builder = QuantifierPromptBuilder::new(QuantifierPromptContext {
            room: context.room,
            previous_room_npcs: context.previous_room_npcs,
            all_known_npcs: context.all_known_npcs,
            player_name: context.player_name,
            recent_history: context.recent_history,
            player_action: context.player_action,
        });

        let (system_prompt, user_prompt) = builder.build();
        let model = get_quantifier_model();

        log::info!(
            "[Quantifier] Calling model: {} for room: {}",
            model,
            context.room.name
        );

        let known_ids: Vec<String> = context
            .all_known_npcs
            .iter()
            .map(|npc| npc.id.clone())
            .collect();

        match call_openrouter_with_model(api_key, &system_prompt, &user_prompt, &model) {
            Ok(response) => {
                log::info!("[Quantifier] Received response ({} chars)", response.len());
                log::debug!(
                    "[Quantifier] Response: {}",
                    &response[..response.len().min(200)]
                );

                let result = parse_quantifier_response(&response, &known_ids);
                log::info!(
                    "[Quantifier] Detected NPCs: {:?} (confidence: {:?})",
                    result.npc_ids,
                    result.confidence
                );
                Ok(result)
            }
            Err(e) => {
                log::warn!("[Quantifier] LLM call failed: {e}, using fallback NPC IDs");
                Ok(QuantifierParseResult {
                    npc_ids: fallback_npc_ids.to_vec(),
                    confidence: QuantifierConfidence::Low,
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::CharacterSheet;
    use crate::model::map::Direction;
    use crate::model::state::LogType;
    use chrono::Utc;
    use std::collections::HashMap;

    // ---- Helper factories ----

    fn make_room() -> Room {
        Room {
            id: "entrance_hall".to_string(),
            name: "Entrance Hall".to_string(),
            description: "A grand entrance hall with marble floors.".to_string(),
            exits: HashMap::from([(Direction::North, "library".to_string())]),
            items: vec![],
            npcs: vec!["gabriella".to_string()],
            image_path: None,
        }
    }

    fn make_npc(id: &str, name: &str) -> NpcCard {
        NpcCard {
            id: id.to_string(),
            sheet: CharacterSheet {
                name: name.to_string(),
                description: format!("A character named {name}."),
                personality: "Mysterious".to_string(),
                scenario: "Investigating".to_string(),
                example_dialogue: String::new(),
                image_path: None,
                headshot_image: None,
            },
            inventory: vec![],
        }
    }

    fn make_history() -> Vec<LogEntry> {
        vec![
            LogEntry {
                sender: Some("Narrator".to_string()),
                text: "You enter the front gate.".to_string(),
                log_type: LogType::Narration,
                timestamp: Utc::now(),
            },
            LogEntry {
                sender: Some("Carla".to_string()),
                text: "I'll follow you inside.".to_string(),
                log_type: LogType::Dialogue,
                timestamp: Utc::now(),
            },
        ]
    }

    // ---- QuantifierPromptBuilder tests ----

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
            player_name: "Hero",
            recent_history: &history,
            player_action: "I walk into the entrance hall.",
        };

        let builder = QuantifierPromptBuilder::new(context);
        let (system, user) = builder.build();

        // System prompt should contain instructions and known NPC IDs
        assert!(system.contains("scene quantifier"));
        assert!(system.contains("npcs_in_room"));
        assert!(system.contains("carla"));
        assert!(system.contains("gabriella"));

        // User prompt should contain room info and previous NPCs
        assert!(user.contains("Entrance Hall"));
        assert!(user.contains("Carla"));
        assert!(user.contains("Hero"));
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
            player_name: "Hero",
            recent_history: &history,
            player_action: "I walk into the entrance hall.",
        };

        let builder = QuantifierPromptBuilder::new(context);
        let (system, user) = builder.build();

        // Total prompt should be under ~4000 chars (roughly 1000 tokens)
        let total_chars = system.len() + user.len();
        assert!(
            total_chars < 4000,
            "Quantifier prompt too long: {total_chars} chars"
        );
    }

    #[test]
    fn test_quantifier_prompt_builder_includes_history() {
        let room = make_room();
        let all_npcs: Vec<NpcCard> = vec![];
        let previous_npcs: Vec<NpcCard> = vec![];
        let history = make_history();

        let context = QuantifierPromptContext {
            room: &room,
            previous_room_npcs: &previous_npcs,
            all_known_npcs: &all_npcs,
            player_name: "Hero",
            recent_history: &history,
            player_action: "I look around.",
        };

        let builder = QuantifierPromptBuilder::new(context);
        let (_, user) = builder.build();

        assert!(user.contains("Recent events"));
        assert!(user.contains("You enter the front gate"));
        assert!(user.contains("I'll follow you inside"));
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
            player_name: "Hero",
            recent_history: &history,
            player_action: "I look around.",
        };

        let builder = QuantifierPromptBuilder::new(context);
        let (_, user) = builder.build();

        // Should not crash with empty history
        assert!(user.contains("Hero"));
        assert!(user.contains("I look around"));
    }

    // ---- Response parsing tests ----

    #[test]
    fn test_parse_json_response() {
        let response = r#"{"npcs_in_room": ["carla", "gabriella"]}"#;
        let known_ids = vec![
            "carla".to_string(),
            "gabriella".to_string(),
            "guard".to_string(),
        ];

        let result = parse_quantifier_response(response, &known_ids);
        assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
        assert_eq!(result.confidence, QuantifierConfidence::High);
    }

    #[test]
    fn test_parse_json_with_markdown_code_fence() {
        let response = "```json\n{\"npcs_in_room\": [\"carla\"]}\n```";
        let known_ids = vec!["carla".to_string(), "gabriella".to_string()];

        let result = parse_quantifier_response(response, &known_ids);
        assert_eq!(result.npc_ids, vec!["carla"]);
        assert_eq!(result.confidence, QuantifierConfidence::High);
    }

    #[test]
    fn test_parse_json_embedded_in_text() {
        let response = "Based on the context, the NPCs present are:\n{\"npcs_in_room\": [\"carla\"]}\nThat's who I see.";
        let known_ids = vec!["carla".to_string()];

        let result = parse_quantifier_response(response, &known_ids);
        assert_eq!(result.npc_ids, vec!["carla"]);
        assert_eq!(result.confidence, QuantifierConfidence::High);
    }

    #[test]
    fn test_parse_text_fallback() {
        let response = "Both Carla and Gabriella are in the room.";
        let known_ids = vec![
            "carla".to_string(),
            "gabriella".to_string(),
            "guard".to_string(),
        ];

        let result = parse_quantifier_response(response, &known_ids);
        assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
        assert_eq!(result.confidence, QuantifierConfidence::Medium);
    }

    #[test]
    fn test_parse_filters_unknown_npcs() {
        let response = r#"{"npcs_in_room": ["carla", "harry", "gabriella"]}"#;
        let known_ids = vec!["carla".to_string(), "gabriella".to_string()];

        let result = parse_quantifier_response(response, &known_ids);
        // "harry" should be filtered out since it's not in known_ids
        assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
        assert_eq!(result.confidence, QuantifierConfidence::High);
    }

    #[test]
    fn test_parse_empty_response() {
        let response = r#"{"npcs_in_room": []}"#;
        let known_ids = vec!["carla".to_string()];

        let result = parse_quantifier_response(response, &known_ids);
        assert!(result.npc_ids.is_empty());
        // Empty valid JSON is still High confidence (the LLM correctly said no one is there)
        assert_eq!(result.confidence, QuantifierConfidence::High);
    }

    #[test]
    fn test_parse_completely_invalid_response() {
        let response = "I don't understand the question.";
        let known_ids = vec!["carla".to_string()];

        let result = parse_quantifier_response(response, &known_ids);
        assert!(result.npc_ids.is_empty());
        assert_eq!(result.confidence, QuantifierConfidence::Low);
    }

    #[test]
    fn test_parse_text_fallback_case_insensitive() {
        let response = "Carla follows the player into the hall.";
        let known_ids = vec!["carla".to_string()];

        let result = parse_quantifier_response(response, &known_ids);
        assert_eq!(result.npc_ids, vec!["carla"]);
        assert_eq!(result.confidence, QuantifierConfidence::Medium);
    }

    #[test]
    fn test_parse_json_empty_array() {
        let response = r#"{"npcs_in_room": []}"#;
        let known_ids: Vec<String> = vec![];

        let result = parse_quantifier_response(response, &known_ids);
        assert!(result.npc_ids.is_empty());
    }

    #[test]
    fn test_quantifier_confidence_levels() {
        // High: JSON parsed, valid IDs
        let high_result =
            parse_quantifier_response(r#"{"npcs_in_room": ["carla"]}"#, &["carla".to_string()]);
        assert_eq!(high_result.confidence, QuantifierConfidence::High);

        // Medium: Text fallback
        let medium_result = parse_quantifier_response("Carla is here.", &["carla".to_string()]);
        assert_eq!(medium_result.confidence, QuantifierConfidence::Medium);

        // Low: No valid IDs
        let low_result = parse_quantifier_response("Random text.", &["carla".to_string()]);
        assert_eq!(low_result.confidence, QuantifierConfidence::Low);
    }
}
