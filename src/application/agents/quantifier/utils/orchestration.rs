//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier orchestration — LLM call + result processing + entry point.

use crate::application::llm_recorder::LlmCallRecorder;
use crate::domain::model::agent::{AgentSettings, HookInput};
use crate::domain::model::character::NpcCard;
use crate::error::EngineError;

use crate::application::agents::quantifier::parser::parse_with_movement;
use crate::application::agents::quantifier::prompt::QuantifierPromptBuilder;
use crate::application::agents::quantifier::types::{
    MovementParseResult, QuantifierConfidence, QuantifierParseResult, QuantifierPromptContext,
    QuantifierResult, RoomInfo,
};

pub fn determine_npcs_in_room(
    input: &HookInput,
    recorder: &LlmCallRecorder,
    settings: &AgentSettings,
) -> Result<QuantifierResult, EngineError> {
    let text = input
        .text
        .ok_or_else(|| EngineError::Config("the analysis needs the saved text".into()))?;
    let world = input.world;
    let current_room = world.room.ok_or_else(|| {
        EngineError::RoomNotFound("current room not set in the hook input".into())
    })?;

    let previous_room_npcs: Vec<NpcCard> = world.present.to_vec();
    let all_npcs: Vec<NpcCard> = world.characters.values().cloned().collect();

    let all_rooms: Vec<RoomInfo> = world
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
        previous_room_npcs: &previous_room_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &all_rooms,
        player_name: &world.persona.sheet.name,
        recent_history: input.story,
        player_action: text,
        quantifier_prompt_override: settings.preset.clone(),
    };

    let result = quantify_room_with_llm_call(&context, recorder, settings.max_tokens);
    Ok(process_quantifier_result(result, world))
}

pub(crate) fn quantify_room_with_llm_call(
    context: &QuantifierPromptContext,
    recorder: &LlmCallRecorder,
    max_tokens: Option<u32>,
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
        "[Quantifier] Calling provider: {} for room: {}",
        recorder.provider_label(),
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
        match recorder.complete("quantifier", &system_prompt, &user_prompt, max_tokens) {
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
    world: &crate::domain::model::agent::WorldView<'_>,
) -> QuantifierResult {
    match result.npcs.confidence {
        QuantifierConfidence::High | QuantifierConfidence::Medium => {
            tracing::info!("[Quantifier] Using dynamic NPCs: {:?}", result.npcs.npc_ids);
            // A Character the World does not know cannot be present.
            let npc_ids = result
                .npcs
                .npc_ids
                .into_iter()
                .filter(|id| world.characters.contains_key(id))
                .collect();
            QuantifierResult {
                npcs: QuantifierParseResult {
                    npc_ids,
                    confidence: result.npcs.confidence,
                },
                movement: result.movement,
            }
        }
        QuantifierConfidence::Low => {
            tracing::info!("[Quantifier] Low confidence, using static NPCs");
            static_npc_result(world, result.movement)
        }
    }
}

pub(crate) fn static_npc_result(
    world: &crate::domain::model::agent::WorldView<'_>,
    movement: MovementParseResult,
) -> QuantifierResult {
    let npc_ids = world.present.iter().map(|n| n.id.clone()).collect();

    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids,
            confidence: QuantifierConfidence::Low,
        },
        movement,
    }
}
