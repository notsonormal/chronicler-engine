use std::sync::Arc;

use crate::engine::action::Action;
use crate::engine::action_processing::{
    FreeActionContext, TriggerContinuationRequest, TriggerMatch, apply_npc_events,
    commit_trigger_narration, execute_freeaction_impl,
};
use crate::engine::parser::parse_command;

use crate::error::EngineError;
use crate::model::agent::{AgentContext, AgentResult, ExecutionPhase, StatePatch};
use crate::model::character::NpcCard;
use crate::model::quantifier::{
    MovementParseResult, QuantifierConfidence, QuantifierParseResult, QuantifierResult,
    compute_npc_events,
};
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogType};
use crate::narrative::prompt::{PromptBuilder, make_prompt_context};

use super::context::GameServiceContext;
use super::helpers::{load_state, map_llm_error, save_committed_state, save_state};
use super::service::DefaultGameService;

/// [DOC: docs/architecture/system.md]
pub fn execute_action_impl(
    service: &DefaultGameService,
    ctx: GameServiceContext,
    input: String,
    _player_name: String,
) {
    let action = parse_command(&input);

    match action {
        Action::Talk(name, msg) => {
            let mut state = load_state(&ctx);
            let msg_str = msg.unwrap_or_default();
            state.add_log(
                format!("You talk to {name}: {msg_str}"),
                None,
                LogType::System,
            );
            if let Err(e) = finish_action(&ctx, state) {
                log::error!("Failed to persist talk action: {e}");
            }
        }
        Action::FreeAction(text) => {
            let _lock = match ctx.action_lock.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let mut state = load_state(&ctx);
            state.narrative.last_trigger = None;
            execute_freeaction_pipeline(service, &ctx, state, text);
        }
    }
}

fn save_pipeline_error(ctx: &GameServiceContext, error: impl Into<String>) {
    let mut state = load_state(ctx);
    state.narrative.generation.status = GenerationStatus::Error(error.into());
    if let Err(e) = save_state(ctx, &mut state) {
        log::error!("Critical: failed to persist error state: {e}");
    }
}

pub(crate) fn finish_action(
    ctx: &GameServiceContext,
    mut state: GameState,
) -> Result<u64, EngineError> {
    state.narrative.generation.status = GenerationStatus::Idle;
    state.narrative.generation.phase = GenerationPhase::default();
    save_state(ctx, &mut state)
}

pub(crate) fn default_quantifier_result(fallback_npc_ids: &[String]) -> QuantifierResult {
    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: fallback_npc_ids.to_vec(),
            confidence: QuantifierConfidence::Low,
        },
        movement: MovementParseResult::default(),
    }
}

/// Re-runs post-generation quantifier and reconciles NPC presence after trigger continuation.
/// Shared between `execute_freeaction_pipeline` and `retry_event_continuation`.
pub(crate) fn reconcile_post_trigger_npcs(
    service: &DefaultGameService,
    state: GameState,
    player_input: &str,
    continuation_text: &str,
) -> Result<GameState, EngineError> {
    let mut state = state;
    state.narrative.generation.phase = GenerationPhase::Quantifying;

    let previous_ids: Vec<String> = state
        .scene
        .npcs_in_area
        .iter()
        .map(|n| n.id.clone())
        .collect();
    let mut post_trigger_result = default_quantifier_result(&previous_ids);
    run_post_generation_agents(
        service,
        &state,
        player_input,
        continuation_text,
        &mut post_trigger_result,
    );

    let npc_cards: Vec<NpcCard> = post_trigger_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect();
    let new_ids: Vec<String> = npc_cards.iter().map(|n| n.id.clone()).collect();

    state.scene.npcs_in_area = npc_cards;

    let events = compute_npc_events(&previous_ids, &new_ids);
    apply_npc_events(state, &events.events)
}

pub(crate) fn run_post_generation_agents(
    service: &DefaultGameService,
    state: &GameState,
    player_input: &str,
    main_response: &str,
    result: &mut QuantifierResult,
) {
    let agent_ctx = AgentContext {
        state,
        main_response: Some(main_response),
        player_input,
        current_room: state.current_room(),
    };

    for agent in service
        .agent_registry
        .agents_for_phase(ExecutionPhase::PostGeneration)
    {
        match agent.execute(&agent_ctx) {
            Ok(AgentResult::StatePatch(StatePatch::Scene {
                npc_ids,
                movement_destination,
                confidence,
            })) => {
                result.npcs.npc_ids = npc_ids;
                result.movement.destination = movement_destination;
                result.npcs.confidence = QuantifierConfidence::from(confidence);
            }
            Ok(AgentResult::NoOp) => {}
            Ok(AgentResult::PromptDirective(_)) => {
                log::warn!("Post-generation agent returned PromptDirective; ignoring");
            }
            Err(e) => {
                log::warn!("Agent {} failed: {e}", agent.name());
            }
        }
    }
}

/// Build a trigger continuation request from raw engine match data.
#[allow(clippy::too_many_arguments)]
fn build_trigger_request(
    state: &GameState,
    narration_text: &str,
    world: &crate::model::world::WorldCard,
    player: &crate::model::character::PlayerCard,
    all_npcs: &[NpcCard],
    response_length: &str,
    max_context_tokens: u32,
    max_tokens: Option<u32>,
    trigger_match: &TriggerMatch,
) -> Option<TriggerContinuationRequest> {
    let continuation_user_msg = format!(
        "Previous narration:\n{}\n\nTrigger event: {}\n\n\
         Continue the scene naturally, incorporating the trigger event into the narrative. \
         Do NOT repeat or contradict what was already described. Build naturally on the existing scene.",
        narration_text, trigger_match.trigger_narration_prompt
    );

    let room_data = state.current_room()?;
    let history = state.narrative.history();
    let trigger_ctx = make_prompt_context(
        world,
        room_data,
        all_npcs,
        &state.scene.npcs_in_area,
        player,
        &continuation_user_msg,
        &history,
    );

    let mut pb = PromptBuilder::from_context(&trigger_ctx);
    pb.max_context_tokens = Some(max_context_tokens);
    pb.requested_max_tokens = max_tokens;
    pb.response_length = Some(response_length);

    let (system_prompt, user_prompt, fitted_max_tokens) = pb.build_split().ok()?;

    Some(TriggerContinuationRequest {
        stored: crate::model::state::StoredTriggerContext {
            npc_id: trigger_match.npc_id.clone(),
            trigger_idx: trigger_match.trigger_idx,
            trigger_name: trigger_match.trigger_name.clone(),
            trigger_repeat: trigger_match.trigger_repeat,
            trigger_narration_prompt: trigger_match.trigger_narration_prompt.clone(),
            system_prompt,
            user_prompt,
            max_tokens: Some(fitted_max_tokens),
        },
    })
}

/// [DOC: docs/architecture/system.md]
pub fn execute_freeaction_pipeline(
    service: &DefaultGameService,
    ctx: &GameServiceContext,
    mut state: GameState,
    text: String,
) {
    let world = Arc::clone(&state.world);
    let map = Arc::clone(&state.map);
    let player = Arc::clone(&state.player);
    let history = state.narrative.history();
    let nearby_npcs = state.scene.npcs_in_area.clone();
    let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

    state.narrative.generation.status = GenerationStatus::Generating;
    state.narrative.generation.phase = GenerationPhase::Narrating;
    if let Err(e) = save_committed_state(ctx, &mut state) {
        log::error!("Failed to save pre-main snapshot: {e}");
        return;
    }

    let Some(room) = map.get_room_by_id(&state.movement.current_room_id) else {
        save_pipeline_error(ctx, "Room not found");
        return;
    };
    let context = make_prompt_context(
        &world,
        room,
        &all_npcs,
        &nearby_npcs,
        &player,
        &text,
        &history,
    );

    let backend = Arc::clone(&service.llm_backend);
    let narration_result =
        match backend.narrate_action(crate::narrative::llm::backend::AGENT_NARRATOR, &context) {
            Ok(result) => result,
            Err(e) => {
                save_pipeline_error(ctx, map_llm_error(&e));
                return;
            }
        };
    let narration_text = narration_result.text;

    if narration_text.trim().is_empty() {
        save_pipeline_error(ctx, "LLM Error: empty response");
        return;
    }

    state.narrative.generation.status = GenerationStatus::Generating;
    state.narrative.generation.phase = GenerationPhase::Quantifying;

    let mut quantifier_result = default_quantifier_result(&[]);
    run_post_generation_agents(
        service,
        &state,
        &text,
        &narration_text,
        &mut quantifier_result,
    );

    if quantifier_result.npcs.confidence == QuantifierConfidence::Low {
        state.add_log(
            "[System] NPC detection uncertain ÔÇö using room defaults".to_string(),
            None,
            LogType::System,
        );
    }

    let (response_length, max_context_tokens, max_tokens) = ctx.prompt_build_params();

    let turn_result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: &narration_text,
            quantifier_result: &quantifier_result,
        },
    );

    match turn_result {
        Ok(turn_result) => {
            let mut next_state = turn_result.next_state;

            let trigger_request = turn_result.trigger_match.and_then(|m| {
                build_trigger_request(
                    &next_state,
                    &narration_text,
                    &world,
                    &player,
                    &all_npcs,
                    &response_length,
                    max_context_tokens,
                    max_tokens,
                    &m,
                )
            });

            if let Some(request) = trigger_request {
                next_state.narrative.generation.status = GenerationStatus::Generating;
                next_state.narrative.generation.phase = GenerationPhase::GeneratingEvent;
                next_state.narrative.last_trigger = Some(request.stored.clone());

                if let Err(e) = save_committed_state(ctx, &mut next_state) {
                    log::error!("Failed to save pre-event snapshot: {e}");
                    return;
                }

                let continuation_result = match backend.complete(
                    crate::narrative::llm::backend::AGENT_TRIGGER,
                    &request.stored.system_prompt,
                    &request.stored.user_prompt,
                    request.stored.max_tokens,
                ) {
                    Ok(result) => result,
                    Err(e) => {
                        log::error!("Trigger narration failed: {e}");
                        next_state.add_log(
                            format!("[Trigger narration failed: {e}]"),
                            None,
                            LogType::System,
                        );
                        next_state.narrative.generation.status =
                            GenerationStatus::Error(format!("Error: {e}"));
                        if let Err(e) = save_state(ctx, &mut next_state) {
                            log::error!("Critical: failed to persist trigger error state: {e}");
                        }
                        return;
                    }
                };
                let continuation_text = continuation_result.text;

                if !continuation_text.is_empty() {
                    next_state = match commit_trigger_narration(
                        next_state.clone(),
                        &request,
                        &continuation_text,
                    ) {
                        Ok(s) => s,
                        Err(e) => {
                            log::error!("Trigger commit failed: {e}");
                            next_state.narrative.generation.status =
                                GenerationStatus::Error(format!("Trigger error: {e}"));
                            next_state
                        }
                    };

                    match reconcile_post_trigger_npcs(
                        service,
                        next_state.clone(),
                        &text,
                        &continuation_text,
                    ) {
                        Ok(updated) => next_state = updated,
                        Err(e) => {
                            log::error!("Failed to apply post-trigger NPC events: {e}");
                            next_state.narrative.generation.status =
                                GenerationStatus::Error(format!("NPC event error: {e}"));
                            if let Err(e) = save_state(ctx, &mut next_state) {
                                log::error!("Critical: failed to persist NPC error state: {e}");
                            }
                            return;
                        }
                    }
                }
            }
            if let Err(e) = finish_action(ctx, next_state) {
                log::error!("Failed to persist finished action: {e}");
            }
        }
        Err(e) => {
            save_pipeline_error(ctx, format!("Error: {e}"));
        }
    }
}
