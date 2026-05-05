use serde::Deserialize;

use crate::narrative::quantifier::types::{
    MovementParseResult, MovementType, NpcEvent, NpcEventList, NpcEventType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult, RoomInfo,
};

/// Serde struct for deserializing the expected JSON response.
#[derive(Deserialize, Debug)]
struct QuantifierJsonResponse {
    #[serde(default)]
    npcs_in_room: Vec<String>,
    #[serde(default)]
    movement: Option<MovementJson>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MovementJson {
    #[serde(rename = "type")]
    movement_type: Option<String>,
    destination: Option<String>,
}

/// [DOC: docs/reference/quantifier_prompt.md]
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

/// [DOC: docs/system/llm_processing.md]
pub fn compute_npc_events(previous_npc_ids: &[String], current_npc_ids: &[String]) -> NpcEventList {
    let previous_set: std::collections::HashSet<_> = previous_npc_ids.iter().collect();
    let current_set: std::collections::HashSet<_> = current_npc_ids.iter().collect();

    let mut events = Vec::new();

    for npc_id in current_npc_ids {
        if !previous_set.contains(npc_id) {
            events.push(NpcEvent {
                npc_id: npc_id.clone(),
                event_type: NpcEventType::Entered,
            });
        }
    }

    for npc_id in previous_npc_ids {
        if !current_set.contains(npc_id) {
            events.push(NpcEvent {
                npc_id: npc_id.clone(),
                event_type: NpcEventType::Left,
            });
        }
    }

    // If we detected any events, use Medium confidence (since we can't be 100% sure)
    // If no events, use Low (nothing happened)
    let confidence = if !events.is_empty() {
        QuantifierConfidence::Medium
    } else {
        QuantifierConfidence::Low
    };

    NpcEventList { events, confidence }
}

/// [DOC: docs/reference/quantifier_prompt.md]
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
pub(crate) fn try_parse_json_full(
    response: &str,
) -> Result<(Vec<String>, Option<MovementJson>), String> {
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

fn extract_json_from_code_fence(response: &str) -> Option<String> {
    let start_marker = "```json";
    let end_marker = "```";

    let start_idx = response.find(start_marker)?;
    let content_start = start_idx + start_marker.len();
    let remaining = &response[content_start..];

    let end_idx = remaining.find(end_marker)?;
    Some(remaining[..end_idx].trim().to_string())
}

pub(crate) fn extract_npc_ids_from_text(response: &str, known_npc_ids: &[String]) -> Vec<String> {
    let response_lower = response.to_lowercase();
    let mut found = Vec::new();

    for id in known_npc_ids {
        if response_lower.contains(&id.to_lowercase()) {
            found.push(id.clone());
        }
    }

    found
}

/// [DOC: docs/reference/quantifier_prompt.md]
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
