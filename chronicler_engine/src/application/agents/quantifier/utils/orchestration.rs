//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier orchestration — LLM call + result processing + entry point.

use crate::application::llm_recorder::LlmCallRecorder;
use crate::domain::model::agent::AgentContext;
use crate::domain::model::character::NpcCard;
use crate::domain::model::state::game_state::GameState;
use crate::error::EngineError;

use crate::application::agents::quantifier::parser::parse_with_movement;
use crate::application::agents::quantifier::prompt::QuantifierPromptBuilder;
use crate::application::agents::quantifier::types::{
    MovementParseResult, QuantifierConfidence, QuantifierParseResult, QuantifierPromptContext,
    QuantifierResult, RoomInfo,
};

pub fn determine_npcs_in_room(
    ctx: &AgentContext,
    main_response: &str,
    recorder: &LlmCallRecorder,
    quantifier_prompt_override: Option<String>,
) -> Result<QuantifierResult, EngineError> {
    let state = ctx.state;
    let current_room = ctx
        .current_room
        .ok_or_else(|| EngineError::RoomNotFound("current room not set in AgentContext".into()))?;
    let map = ctx.map;
    let persona = ctx.persona;
    let npcs = ctx.npcs;

    let previous_room_npcs: Vec<NpcCard> = state.scene.npcs_in_area.clone();

    let all_npcs: Vec<NpcCard> = npcs.values().cloned().collect();

    let recent_history: Vec<_> = state
        .narrative
        .history()
        .iter()
        .rev()
        .take(4)
        .rev()
        .cloned()
        .collect();

    let all_rooms: Vec<RoomInfo> = map
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
        previous_room_npcs: &previous_room_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &all_rooms,
        player_name: &persona.sheet.name,
        recent_history: &recent_history,
        player_action: main_response,
        quantifier_prompt_override,
    };

    let result = quantify_room_with_llm_call(&context, recorder);
    Ok(process_quantifier_result(result, state, npcs))
}

pub(crate) fn quantify_room_with_llm_call(
    context: &QuantifierPromptContext,
    recorder: &LlmCallRecorder,
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
        recorder.provider().name(),
        recorder.provider().model(),
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
        match recorder.complete("quantifier", &system_prompt, &user_prompt, None) {
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

                let result = parse_with_movement(response, &known_ids);
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

                if result.npcs.confidence.is_low() && attempt < max_attempts {
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
            npc_ids: Vec::new(),
            confidence: QuantifierConfidence::Low,
        },
        movement: MovementParseResult {
            movement_type: None,
            destination: None,
            confidence: QuantifierConfidence::Low,
        },
    }
}

pub(crate) fn process_quantifier_result(
    result: QuantifierResult,
    state: &GameState,
    npcs: &std::collections::HashMap<String, crate::domain::model::character::NpcCard>,
) -> QuantifierResult {
    match result.npcs.confidence {
        QuantifierConfidence::High | QuantifierConfidence::Medium => {
            tracing::info!("[Quantifier] Using dynamic NPCs: {:?}", result.npcs.npc_ids);
            let npc_cards: Vec<crate::domain::model::character::NpcCard> = result
                .npcs
                .npc_ids
                .iter()
                .filter_map(|id| npcs.get(id).cloned())
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
            static_npc_result(state, result.movement)
        }
    }
}

pub(crate) fn static_npc_result(
    state: &GameState,
    movement: MovementParseResult,
) -> QuantifierResult {
    let npc_ids = state
        .scene
        .npcs_in_area
        .iter()
        .map(|n| n.id.clone())
        .collect();

    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids,
            confidence: QuantifierConfidence::Low,
        },
        movement,
    }
}
