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

/// Basic room information for the quantifier prompt.
pub struct RoomInfo {
    pub id: String,
    pub name: String,
}

/// Type of movement detected by the quantifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovementType {
    /// Player is entering a new room ("I walk through the gate", "enter the kitchen")
    Entering,
    /// Player is already in a room (contextual, rarely used)
    In,
    /// Player is leaving the current room ("I leave the house", "go outside")
    Leaving,
}

/// Result of detecting movement intent from the quantifier.
#[derive(Debug, Clone)]
pub struct MovementParseResult {
    /// Type of movement detected, if any.
    pub movement_type: Option<MovementType>,
    /// Destination room ID or name, if detected.
    pub destination: Option<String>,
    /// Confidence level of the movement detection.
    pub confidence: QuantifierConfidence,
}

/// Combined result from the quantifier: NPCs in room + movement intent.
#[derive(Debug, Clone)]
pub struct QuantifierResult {
    /// NPCs detected as present in the room.
    pub npcs: QuantifierParseResult,
    /// Movement intent detected, if any.
    pub movement: MovementParseResult,
}

/// Context needed to build a quantifier prompt.
pub struct QuantifierPromptContext<'a> {
    pub room: &'a Room,
    pub previous_room_npcs: &'a [NpcCard],
    pub all_known_npcs: &'a [NpcCard],
    pub all_rooms: &'a [RoomInfo],
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
            r#"<QuantifierTask>
You are a scene quantifier for a text adventure game.
Your task is to determine which NPCs are present in the current room
and whether the player is moving to a new location.

Respond ONLY with a JSON object in this exact format:
{"npcs_in_room": ["id1", "id2"], "movement": {"type": "entering|in|leaving", "destination": "room_id"}}

Rules:
- Only include NPCs that would logically be in the room based on context.
- NPCs from the previous room may have followed the player.
- Use the exact NPC IDs provided in the AvailableNpcIds list.
- Movement is determined by narrative context, not explicit commands.
- If no NPCs are present, return an empty array: {"npcs_in_room": []}
- If no movement detected, set type to null: {"movement": {"type": null}}
</QuantifierTask>

<AvailableNpcIds>
"#,
        );

        for npc in self.context.all_known_npcs {
            prompt.push_str(&format!(
                "  <Npc id=\"{}\" name=\"{}\"/>\n",
                npc.id, npc.sheet.name
            ));
        }

        prompt.push_str("</AvailableNpcIds>\n\n<AvailableRooms>\n");

        for room in self.context.all_rooms {
            prompt.push_str(&format!(
                "  <Room id=\"{}\" name=\"{}\"/>\n",
                room.id, room.name
            ));
        }

        prompt.push_str("</AvailableRooms>\n");

        prompt
    }

    fn build_user_prompt(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str("<CurrentRoom>\n");
        prompt.push_str(&format!("  <Name>{}</Name>\n", self.context.room.name));
        prompt.push_str(&format!(
            "  <Description>{}</Description>\n",
            self.context.room.description
        ));

        // Add navigation description if available
        if let Some(nav_desc) = &self.context.room.navigation_description {
            prompt.push_str(&format!("  <Navigation>{nav_desc}</Navigation>\n"));
        }

        prompt.push_str("</CurrentRoom>\n\n");

        if !self.context.previous_room_npcs.is_empty() {
            prompt.push_str("<PreviousRoomNpcs>\n");
            for npc in self.context.previous_room_npcs {
                prompt.push_str(&format!(
                    "  <Npc id=\"{}\" name=\"{}\">{}</Npc>\n",
                    npc.id, npc.sheet.name, npc.sheet.description
                ));
            }
            prompt.push_str("</PreviousRoomNpcs>\n\n");
        }

        if !self.context.room.npcs.is_empty() {
            prompt.push_str("<RoomConfiguredNpcs>\n");
            prompt.push_str("  ");
            prompt.push_str(&self.context.room.npcs.join(", "));
            prompt.push_str("\n</RoomConfiguredNpcs>\n\n");
        }

        if !self.context.recent_history.is_empty() {
            prompt.push_str("<RecentHistory>\n");
            for entry in self.context.recent_history {
                let sender = entry.sender.as_deref().unwrap_or("Narrator");
                prompt.push_str(&format!(
                    "  <Entry sender=\"{}\">{}</Entry>\n",
                    sender, entry.text
                ));
            }
            prompt.push_str("</RecentHistory>\n\n");
        }

        prompt.push_str(&format!(
            "<PlayerAction>\n  {}: {}\n</PlayerAction>\n\n",
            self.context.player_name, self.context.player_action
        ));

        prompt.push_str(
            r#"<Query>
Based on the context above, determine:
- Which NPCs are present in the current room
- Whether the player is entering, leaving, or remaining

Respond ONLY with the JSON format specified in <QuantifierTask>.
</Query>
"#,
        );

        prompt
    }
}

/// Serde struct for deserializing the expected JSON response.
#[derive(Deserialize, Debug)]
struct QuantifierJsonResponse {
    #[serde(default)]
    npcs_in_room: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    movement: Option<MovementJson>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct MovementJson {
    /// Maps JSON "type" field to Rust "movement_type" field
    #[serde(rename = "type")]
    movement_type: Option<String>,
    destination: Option<String>,
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
    if let Ok((npc_ids, _movement)) = try_parse_json_full(trimmed) {
        let valid_ids: Vec<String> = npc_ids
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

/// Parse the quantifier LLM response to extract both NPCs and movement.
///
/// Returns a combined `QuantifierResult` with NPC presence and movement intent.
pub fn parse_quantifier_response_with_movement(
    response: &str,
    known_npc_ids: &[String],
    _all_rooms: &[RoomInfo],
) -> QuantifierResult {
    let trimmed = response.trim();

    // Try JSON parse first
    if let Ok((npc_ids, movement_json)) = try_parse_json_full(trimmed) {
        let valid_ids: Vec<String> = npc_ids
            .into_iter()
            .filter(|id| known_npc_ids.contains(id))
            .collect();

        let movement = movement_json.map(|m| MovementParseResult {
            movement_type: m
                .movement_type
                .as_ref()
                .and_then(|t| match t.to_lowercase().as_str() {
                    "entering" => Some(MovementType::Entering),
                    "leaving" => Some(MovementType::Leaving),
                    _ => None,
                }),
            destination: m.destination,
            confidence: QuantifierConfidence::High,
        });

        return QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: valid_ids,
                confidence: QuantifierConfidence::High,
            },
            movement: movement.unwrap_or(MovementParseResult {
                movement_type: None,
                destination: None,
                confidence: QuantifierConfidence::High,
            }),
        };
    }

    // Text fallback for NPCs
    let text_ids = extract_npc_ids_from_text(trimmed, known_npc_ids);
    let npcs = if !text_ids.is_empty() {
        QuantifierParseResult {
            npc_ids: text_ids,
            confidence: QuantifierConfidence::Medium,
        }
    } else {
        QuantifierParseResult {
            npc_ids: Vec::new(),
            confidence: QuantifierConfidence::Low,
        }
    };

    // Movement is only detected via JSON - no text fallback
    let movement = MovementParseResult {
        movement_type: None,
        destination: None,
        confidence: QuantifierConfidence::Low,
    };

    QuantifierResult { npcs, movement }
}

/// Try to parse the response as JSON, handling markdown code fences.
fn try_parse_json_full(response: &str) -> Result<(Vec<String>, Option<MovementJson>), String> {
    // Try direct JSON parse first
    if let Ok(parsed) = serde_json::from_str::<QuantifierJsonResponse>(response) {
        return Ok((parsed.npcs_in_room, parsed.movement));
    }

    // Try extracting JSON from markdown code fences (```json ... ```)
    if let Some(json_content) = extract_json_from_code_fence(response) {
        if let Ok(parsed) = serde_json::from_str::<QuantifierJsonResponse>(&json_content) {
            return Ok((parsed.npcs_in_room, parsed.movement));
        }
    }

    // Try finding JSON object in the response
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            let json_str = &response[start..=end];
            if let Ok(parsed) = serde_json::from_str::<QuantifierJsonResponse>(json_str) {
                return Ok((parsed.npcs_in_room, parsed.movement));
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

/// Extract movement intent from text response using keyword matching.
/// Used by fragments.rs for direct movement detection from LLM narration.
pub fn extract_movement_from_text(
    response: &str,
    all_rooms: &[RoomInfo],
) -> Option<MovementParseResult> {
    let response_lower = response.to_lowercase();

    let entering_keywords = [
        "enters",
        "enter",
        "walk into",
        "go into",
        "go to",
        "head to",
        "travel to",
    ];
    let leaving_keywords = [
        "leaves", "leave", "exits", "exit", "go out", "walk out", "head out",
    ];

    let movement_type = if entering_keywords.iter().any(|k| response_lower.contains(k)) {
        Some(MovementType::Entering)
    } else if leaving_keywords.iter().any(|k| response_lower.contains(k)) {
        Some(MovementType::Leaving)
    } else {
        None
    };

    let destination = all_rooms
        .iter()
        .find(|r| response_lower.contains(&r.name.to_lowercase()))
        .map(|r| r.name.clone());

    if movement_type.is_some() || destination.is_some() {
        Some(MovementParseResult {
            movement_type,
            destination,
            confidence: QuantifierConfidence::Medium,
        })
    } else {
        None
    }
}

/// Backend for calling the quantifier LLM.
///
/// Uses a separate model (configured via `QUANTIFIER_MODEL` env var)
/// from the main narrative generator.
pub struct QuantifierBackend;

impl QuantifierBackend {
    /// Quantify which NPCs are in the current room and detect movement intent.
    ///
    /// Returns a `QuantifierResult` containing both NPC presence and movement detection.
    /// - `npcs`: Detected NPC IDs with confidence level.
    /// - `movement`: Detected movement type (entering/leaving/in) and destination.
    ///
    /// If the LLM call fails entirely, returns the `fallback_npc_ids` with `Low` confidence.
    pub fn quantify_room(
        &self,
        api_key: &str,
        context: &QuantifierPromptContext,
        fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError> {
        let builder = QuantifierPromptBuilder::new(QuantifierPromptContext {
            room: context.room,
            previous_room_npcs: context.previous_room_npcs,
            all_known_npcs: context.all_known_npcs,
            all_rooms: context.all_rooms,
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
                log::info!("[Quantifier] Player action: {}", context.player_action);
                log::info!("[Quantifier] Received response ({} chars)", response.len());
                log::debug!(
                    "[Quantifier] Response: {}",
                    &response[..response.len().min(200)]
                );

                let result = parse_quantifier_response_with_movement(
                    &response,
                    &known_ids,
                    context.all_rooms,
                );
                log::info!(
                    "[Quantifier] Detected NPCs: {:?} (confidence: {:?})",
                    result.npcs.npc_ids,
                    result.npcs.confidence
                );
                if let Some(mt) = &result.movement.movement_type {
                    log::info!(
                        "[Quantifier] Detected movement: {:?} destination: {:?}",
                        mt,
                        result.movement.destination
                    );
                } else {
                    log::info!("[Quantifier] No movement detected");
                }
                Ok(result)
            }
            Err(e) => {
                log::warn!("[Quantifier] LLM call failed: {e}, using fallback NPC IDs");
                Ok(QuantifierResult {
                    npcs: QuantifierParseResult {
                        npc_ids: fallback_npc_ids.to_vec(),
                        confidence: QuantifierConfidence::Low,
                    },
                    movement: MovementParseResult {
                        movement_type: None,
                        destination: None,
                        confidence: QuantifierConfidence::Low,
                    },
                })
            }
        }
    }
}

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
            navigation_description: None,
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
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
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
            all_rooms: &[],
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
            all_rooms: &[],
            player_name: "Hero",
            recent_history: &history,
            player_action: "I look around.",
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

    #[test]
    fn test_parse_quantifier_response_no_movement() {
        let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#;

        let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

        assert_eq!(result.npcs.npc_ids, vec!["carla"]);
        assert_eq!(result.movement.movement_type, None);
        assert_eq!(result.movement.destination, None);
    }

    #[test]
    fn test_parse_movement_entering_with_destination() {
        // The actual LLM response format we're using
        let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "entering", "destination": "entrance_hall"}}"#;

        let result = parse_quantifier_response_with_movement(
            response,
            &["carla".to_string()],
            &[RoomInfo {
                id: "entrance_hall".to_string(),
                name: "Entrance Hall".to_string(),
            }],
        );

        assert_eq!(result.npcs.npc_ids, vec!["carla"]);
        assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
        assert_eq!(
            result.movement.destination,
            Some("entrance_hall".to_string())
        );
    }

    #[test]
    fn test_parse_movement_leaving_with_destination() {
        let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "leaving", "destination": "tavern"}}"#;

        let result = parse_quantifier_response_with_movement(
            response,
            &["carla".to_string()],
            &[RoomInfo {
                id: "tavern".to_string(),
                name: "Tavern".to_string(),
            }],
        );

        assert_eq!(result.movement.movement_type, Some(MovementType::Leaving));
        assert_eq!(result.movement.destination, Some("tavern".to_string()));
    }

    #[test]
    fn test_parse_movement_no_movement_field() {
        // No movement field at all - should detect no movement
        let response = r#"{"npcs_in_room": ["carla"]}"#;

        let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

        assert_eq!(result.npcs.npc_ids, vec!["carla"]);
        assert_eq!(result.movement.movement_type, None);
        assert_eq!(result.movement.destination, None);
    }

    #[test]
    fn test_parse_movement_empty_movement_object() {
        // Empty movement object - should detect no movement
        let response = r#"{"npcs_in_room": ["carla"], "movement": {}}"#;

        let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

        assert_eq!(result.npcs.npc_ids, vec!["carla"]);
        assert_eq!(result.movement.movement_type, None);
        assert_eq!(result.movement.destination, None);
    }

    #[test]
    fn test_parse_movement_unknown_type_becomes_none() {
        // Unknown movement type should map to None (not panic)
        let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "teleporting", "destination": "castle"}}"#;

        let result = parse_quantifier_response_with_movement(
            response,
            &["carla".to_string()],
            &[RoomInfo {
                id: "castle".to_string(),
                name: "Castle".to_string(),
            }],
        );

        assert_eq!(result.npcs.npc_ids, vec!["carla"]);
        // Unknown type maps to None
        assert_eq!(result.movement.movement_type, None);
        // But destination should still be captured
        assert_eq!(result.movement.destination, Some("castle".to_string()));
    }

    #[test]
    fn test_parse_movement_case_insensitive_type() {
        // Type should be case-insensitive
        let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "ENTERING", "destination": "hall"}}"#;

        let result = parse_quantifier_response_with_movement(
            response,
            &["carla".to_string()],
            &[RoomInfo {
                id: "hall".to_string(),
                name: "Hall".to_string(),
            }],
        );

        assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    }

    #[test]
    fn test_quantifier_prompt_includes_navigation() {
        let room = Room {
            id: "test_room".to_string(),
            name: "Test Room".to_string(),
            description: "A test room.".to_string(),
            exits: HashMap::new(),
            items: vec![],
            npcs: vec![],
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
}
