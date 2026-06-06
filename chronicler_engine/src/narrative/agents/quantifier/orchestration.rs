//! [DOC: docs/system/agent_system.md]
//! Quantifier orchestration

use crate::narrative::llm::backend::LlmBackend;

use super::parser::parse_quantifier_response_with_movement;
use super::prompt::QuantifierPromptBuilder;
use super::types::{
    MovementParseResult, QuantifierConfidence, QuantifierParseResult, QuantifierPromptContext,
    QuantifierResult, RoomInfo,
};

pub(crate) fn quantify_room_with_llm_call(
    context: &QuantifierPromptContext,
    fallback_npc_ids: &[String],
    backend: &dyn LlmBackend,
) -> QuantifierResult {
    let builder = QuantifierPromptBuilder::new(QuantifierPromptContext {
        room: context.room,
        previous_room_npcs: context.previous_room_npcs,
        all_known_npcs: context.all_known_npcs,
        all_rooms: context.all_rooms,
        player_name: context.player_name,
        recent_history: context.recent_history,
        player_action: context.player_action,
        quantifier_prompt_override: context.quantifier_prompt_override.clone(),
    });

    let (system_prompt, user_prompt) = builder.build();

    tracing::info!(
        "[Quantifier] Calling backend: {} model: {} for room: {}",
        backend.name(),
        backend.model(),
        context.room.name
    );

    let known_ids: Vec<String> = context
        .all_known_npcs
        .iter()
        .map(|npc| npc.id.clone())
        .collect();

    let max_attempts = 2;
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        match backend.complete("quantifier", &system_prompt, &user_prompt, None) {
            Ok(llm_result) => {
                let response = &llm_result.text;
                tracing::info!("[Quantifier] Player action: {}", context.player_action);
                tracing::info!(
                    "[Quantifier] Received response ({} chars) [attempt {}/{}]",
                    response.len(),
                    attempt,
                    max_attempts
                );
                tracing::debug!(
                    "[Quantifier] Response: {}",
                    &response[..response.len().min(200)]
                );

                let result = parse_quantifier_response_with_movement(
                    response,
                    &known_ids,
                    context.all_rooms,
                );
                tracing::info!(
                    "[Quantifier] Detected NPCs: {:?} (confidence: {:?})",
                    result.npcs.npc_ids,
                    result.npcs.confidence
                );
                if let Some(mt) = &result.movement.movement_type {
                    tracing::info!(
                        "[Quantifier] Detected movement: {:?} destination: {:?}",
                        mt,
                        result.movement.destination
                    );
                } else {
                    tracing::info!("[Quantifier] No movement detected");
                }

                // Retry on Low confidence (unless this was the last attempt)
                if result.npcs.confidence == QuantifierConfidence::Low && attempt < max_attempts {
                    tracing::warn!("[Quantifier] Low confidence on attempt {attempt}, retrying...");
                    continue;
                }

                return result;
            }
            Err(e) => {
                tracing::warn!("[Quantifier] LLM call failed on attempt {attempt}: {e}");
                last_error = Some(e.to_string());
                if attempt < max_attempts {
                    continue;
                }
            }
        }
    }

    tracing::warn!(
        "[Quantifier] All attempts failed, using fallback NPC IDs. Last error: {}",
        last_error.as_deref().unwrap_or("unknown")
    );
    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: fallback_npc_ids.to_vec(),
            confidence: QuantifierConfidence::Low,
        },
        movement: MovementParseResult {
            movement_type: None,
            destination: None,
            confidence: QuantifierConfidence::Low,
        },
    }
}

/// Process quantifier result with confidence-based branching.
///
/// High/Medium confidence: use dynamically quantified NPCs
/// Low confidence: fall back to static NPCs
fn process_quantifier_result(
    result: QuantifierResult,
    state: &crate::model::state::GameState,
    room_npc_ids: &[String],
) -> QuantifierResult {
    match result.npcs.confidence {
        QuantifierConfidence::High | QuantifierConfidence::Medium => {
            tracing::info!("[Quantifier] Using dynamic NPCs: {:?}", result.npcs.npc_ids);
            let npc_cards: Vec<crate::model::character::NpcCard> = result
                .npcs
                .npc_ids
                .iter()
                .filter_map(|id| state.npcs.get(id).cloned())
                .collect();
            QuantifierResult {
                npcs: QuantifierParseResult {
                    npc_ids: npc_cards.iter().map(|n| n.id.clone()).collect(),
                    confidence: result.npcs.confidence,
                },
                movement: result.movement,
            }
        }
        QuantifierConfidence::Low => {
            tracing::info!("[Quantifier] Low confidence, using static NPCs");
            static_npc_result(state, room_npc_ids, result.movement)
        }
    }
}

pub(crate) fn static_npc_result(
    state: &crate::model::state::GameState,
    room_npc_ids: &[String],
    movement: MovementParseResult,
) -> QuantifierResult {
    let npc_ids = if room_npc_ids.is_empty() {
        // Fallback: preserve previous turn's NPCs when no static room NPCs are configured.
        state
            .scene
            .npcs_in_area
            .iter()
            .map(|n| n.id.clone())
            .collect()
    } else {
        room_npc_ids
            .iter()
            .filter_map(|id| state.npcs.get(id))
            .map(|n| n.id.clone())
            .collect()
    };

    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids,
            confidence: QuantifierConfidence::Low,
        },
        movement,
    }
}

pub fn determine_npcs_in_room(
    state: &crate::model::state::GameState,
    current_room: &crate::model::map::Room,
    room_npc_ids: &[String],
    previous_room_npcs: &[crate::model::character::NpcCard],
    player_action: &str,
    backend: &dyn LlmBackend,
    quantifier_prompt_override: Option<String>,
) -> QuantifierResult {
    let all_npcs: Vec<crate::model::character::NpcCard> = state.npcs.values().cloned().collect();

    let recent_history: Vec<_> = state
        .narrative
        .history()
        .iter()
        .rev()
        .take(4)
        .rev()
        .cloned()
        .collect();

    let all_rooms: Vec<RoomInfo> = state
        .map
        .overworld
        .regions
        .iter()
        .flat_map(|region| {
            region.rooms.iter().map(|room| RoomInfo {
                id: room.id.clone(),
                name: room.name.clone(),
            })
        })
        .collect();

    let context = QuantifierPromptContext {
        room: current_room,
        previous_room_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &all_rooms,
        player_name: &state.player.sheet.name,
        recent_history: &recent_history,
        player_action,
        quantifier_prompt_override,
    };

    let result = quantify_room_with_llm_call(&context, room_npc_ids, backend);
    process_quantifier_result(result, state, room_npc_ids)
}
