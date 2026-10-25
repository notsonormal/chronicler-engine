//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier response parsing helpers.

use serde::Deserialize;

use crate::application::agents::quantifier::types::{
    MovementParseResult, MovementType, QuantifierConfidence, QuantifierParseResult,
    QuantifierReading,
};

#[derive(Deserialize, Debug)]
struct QuantifierJsonResponse {
    /// An edited prompt preset can ask for `npcs_in_room`.
    #[serde(default, alias = "npcs_in_room")]
    characters_present: Vec<String>,
    #[serde(default)]
    movement: Option<MovementJson>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MovementJson {
    #[serde(rename = "type")]
    movement_type: Option<String>,
    destination: Option<String>,
}

pub(crate) fn extract_characters(
    response: &str,
    known_character_ids: &[String],
) -> QuantifierParseResult {
    let trimmed = response.trim();

    if let Ok((character_ids, _movement)) = try_parse_json_full(trimmed) {
        let valid_ids: Vec<String> = character_ids
            .into_iter()
            .filter(|id| known_character_ids.contains(id))
            .collect();

        // JSON parsed successfully - this is High confidence even if the
        // result is empty (the LLM correctly determined no Characters are present)
        return QuantifierParseResult {
            character_ids: valid_ids,
            confidence: QuantifierConfidence::High,
        };
    }

    let text_ids = extract_character_ids_from_text(trimmed, known_character_ids);

    if !text_ids.is_empty() {
        return QuantifierParseResult {
            character_ids: text_ids,
            confidence: QuantifierConfidence::Medium,
        };
    }

    QuantifierParseResult {
        character_ids: Vec::new(),
        confidence: QuantifierConfidence::Low,
    }
}

pub(crate) fn try_parse_json_full(
    response: &str,
) -> Result<(Vec<String>, Option<MovementJson>), String> {
    match serde_json::from_str::<QuantifierJsonResponse>(response) {
        Ok(parsed) => {
            tracing::debug!(
                "[Quantifier] JSON parsed successfully: characters={:?}, movement={:?}",
                parsed.characters_present,
                parsed.movement
            );
            return Ok((parsed.characters_present, parsed.movement));
        }
        Err(e) => {
            tracing::debug!("[Quantifier] Direct JSON parse failed: {}", e);
        }
    }
    if let Some(json_content) = extract_json_from_code_fence(response) {
        match serde_json::from_str::<QuantifierJsonResponse>(&json_content) {
            Ok(parsed) => {
                tracing::debug!(
                    "[Quantifier] Code fence JSON parsed: characters={:?}, movement={:?}",
                    parsed.characters_present,
                    parsed.movement
                );
                return Ok((parsed.characters_present, parsed.movement));
            }
            Err(e) => {
                tracing::debug!("[Quantifier] Code fence JSON parse failed: {}", e);
            }
        }
    }
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            let json_str = &response[start..=end];
            match serde_json::from_str::<QuantifierJsonResponse>(json_str) {
                Ok(parsed) => {
                    tracing::debug!(
                        "[Quantifier] Extracted JSON parsed: characters={:?}, movement={:?}",
                        parsed.characters_present,
                        parsed.movement
                    );
                    return Ok((parsed.characters_present, parsed.movement));
                }
                Err(e) => {
                    tracing::debug!("[Quantifier] Extracted JSON parse failed: {}", e);
                }
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

pub(crate) fn extract_character_ids_from_text(
    response: &str,
    known_character_ids: &[String],
) -> Vec<String> {
    let response_lower = response.to_lowercase();
    let mut found = Vec::new();

    for id in known_character_ids {
        if response_lower.contains(&id.to_lowercase()) {
            found.push(id.clone());
        }
    }

    found
}

pub(crate) fn parse_with_movement(
    response: &str,
    known_character_ids: &[String],
) -> QuantifierReading {
    let trimmed = response.trim();

    tracing::debug!(
        "[Quantifier Parser] Attempting to parse {} chars",
        trimmed.len()
    );
    if let Ok((character_ids, movement_json)) = try_parse_json_full(trimmed) {
        tracing::debug!(
            "[Quantifier Parser] JSON parsing succeeded: {} Characters found",
            character_ids.len()
        );
        let valid_ids: Vec<String> = character_ids
            .into_iter()
            .filter(|id| known_character_ids.contains(id))
            .collect();
        tracing::debug!(
            "[Quantifier Parser] After filtering: {} valid Characters ({:?})",
            valid_ids.len(),
            valid_ids
        );

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
        });

        return QuantifierReading {
            characters: QuantifierParseResult {
                character_ids: valid_ids,
                confidence: QuantifierConfidence::High,
            },
            movement: movement.unwrap_or(MovementParseResult {
                movement_type: None,
                destination: None,
            }),
        };
    }
    tracing::debug!("[Quantifier Parser] JSON failed, falling back to text extraction");
    let characters = extract_characters(response, known_character_ids);
    tracing::debug!(
        "[Quantifier Parser] Text extraction found {} Characters ({:?}) with {:?} confidence",
        characters.character_ids.len(),
        characters.character_ids,
        characters.confidence
    );
    let movement = MovementParseResult {
        movement_type: None,
        destination: None,
    };
    QuantifierReading {
        characters,
        movement,
    }
}
