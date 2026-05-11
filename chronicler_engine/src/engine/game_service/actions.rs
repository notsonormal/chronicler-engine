use std::sync::Arc;

use crate::engine::action::Action;
use crate::engine::action_processing::{
    commit_trigger_narration, execute_freeaction_impl, get_static_npcs,
};
use crate::engine::logic::get_current_room;
use crate::engine::parser::parse_command;
use crate::model::agent::{AgentContext, AgentResult, Confidence, ExecutionPhase, StatePatch};
use crate::model::character::NpcCard;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogType};
use crate::narrative::agents::quantifier::{
    MovementParseResult, QuantifierConfidence, QuantifierParseResult, QuantifierResult,
};
use crate::narrative::prompt::make_prompt_context;

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
        Action::Quit => {
            let mut state = load_state(&ctx);
            state.add_log("Goodbye!".to_string(), None, LogType::System);
            state.narrative.generation.status = GenerationStatus::Idle;
            state.narrative.generation.phase = GenerationPhase::default();
            save_state(&ctx, &state, uuid::Uuid::new_v4().to_string(), 0);
        }
        Action::Look => {
            let mut state = load_state(&ctx);
            let room_name;
            let room_desc;
            {
                let room = get_current_room(&state).ok();
                room_name = room.as_ref().map(|r| r.name.clone());
                room_desc = room.map(|r| r.description.clone());
            }
            if let Some(name) = room_name {
                if let Some(desc) = room_desc {
                    state.add_log(desc, Some(name), LogType::Narration);
                }
            }
            state.narrative.generation.status = GenerationStatus::Idle;
            state.narrative.generation.phase = GenerationPhase::default();
            save_state(&ctx, &state, uuid::Uuid::new_v4().to_string(), 0);
        }
        Action::Talk(name, msg) => {
            let mut state = load_state(&ctx);
            let msg_str = msg.unwrap_or_default();
            state.add_log(
                format!("You talk to {name}: {msg_str}"),
                None,
                LogType::System,
            );
            state.narrative.generation.status = GenerationStatus::Idle;
            state.narrative.generation.phase = GenerationPhase::default();
            save_state(&ctx, &state, uuid::Uuid::new_v4().to_string(), 0);
        }
        Action::Inventory => {
            let mut state = load_state(&ctx);
            state.add_log(
                "Your inventory is empty.".to_string(),
                None,
                LogType::System,
            );
            state.narrative.generation.status = GenerationStatus::Idle;
            state.narrative.generation.phase = GenerationPhase::default();
            save_state(&ctx, &state, uuid::Uuid::new_v4().to_string(), 0);
        }
        Action::FreeAction(text) => {
            let _lock = match ctx.action_lock.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let message_id = uuid::Uuid::new_v4().to_string();
            let mut state = load_state(&ctx);
            state.narrative.last_trigger = None;
            execute_freeaction_pipeline(service, &ctx, state, message_id, text, 0);
        }
    }
}

fn save_pipeline_error(
    ctx: &GameServiceContext,
    message_id: &str,
    swipe_index: u32,
    error: impl Into<String>,
) {
    let mut state = load_state(ctx);
    state.narrative.generation.status = GenerationStatus::Error(error.into());
    save_state(ctx, &state, message_id.to_string(), swipe_index);
}

/// [DOC: docs/architecture/system.md]
/// Run the full free-action pipeline from a given starting state.
/// Used by both normal action handling and retry logic.
/// `swipe_index` should be 0 for fresh turns, or incremented for retries.
pub fn execute_freeaction_pipeline(
    service: &DefaultGameService,
    ctx: &GameServiceContext,
    mut state: GameState,
    message_id: String,
    text: String,
    swipe_index: u32,
) {
    let world = Arc::clone(&state.world);
    let map = Arc::clone(&state.map);
    let player = Arc::clone(&state.player);
    let room_id = state.movement.current_room_id.clone();
    let history = state.narrative.history.clone();
    let room_npc_ids = get_current_room(&state)
        .map(|r| r.npcs.clone())
        .unwrap_or_default();
    let nearby_npcs = get_static_npcs(&state, &room_npc_ids);
    let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

    state.narrative.generation.status = GenerationStatus::Generating;
    state.narrative.generation.phase = GenerationPhase::Narrating;
    save_committed_state(ctx, &state, format!("pre-main:{message_id}"), 0);

    let room = map
        .overworld
        .regions
        .iter()
        .flat_map(|r| r.rooms.iter())
        .find(|r| r.id == room_id);

    let Some(room) = room else {
        save_pipeline_error(ctx, &message_id, swipe_index, "Room not found");
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
    let narration_text = match backend.narrate_action(&context) {
        Ok(t) => t,
        Err(e) => {
            save_pipeline_error(ctx, &message_id, swipe_index, map_llm_error(&e));
            return;
        }
    };

    if narration_text.trim().is_empty() {
        save_pipeline_error(ctx, &message_id, swipe_index, "LLM Error: empty response");
        return;
    }

    let mut state = load_state(ctx);
    state.narrative.generation.status = GenerationStatus::Generating;
    state.narrative.generation.phase = GenerationPhase::Quantifying;

    let mut quantifier_result = QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: room.npcs.clone(),
            confidence: QuantifierConfidence::Low,
        },
        movement: MovementParseResult {
            movement_type: None,
            destination: None,
            confidence: QuantifierConfidence::Low,
        },
    };

    let agent_ctx = AgentContext {
        state: &state,
        main_response: Some(&narration_text),
        player_input: &text,
    };

    for agent in service
        .agent_registry
        .agents_for_phase(ExecutionPhase::PostGeneration)
    {
        match agent.execute(&agent_ctx) {
            Ok(AgentResult::StatePatch(patch)) => match patch {
                StatePatch::Scene {
                    npc_ids,
                    movement_destination,
                    confidence,
                } => {
                    quantifier_result.npcs.npc_ids = npc_ids;
                    quantifier_result.movement.destination = movement_destination;
                    quantifier_result.npcs.confidence = match confidence {
                        Confidence::High => QuantifierConfidence::High,
                        Confidence::Medium => QuantifierConfidence::Medium,
                        Confidence::Low => QuantifierConfidence::Low,
                    };
                }
            },
            Ok(AgentResult::NoOp) => {}
            Ok(AgentResult::PromptDirective(_)) => {
                log::warn!("Post-generation agent returned PromptDirective; ignoring");
            }
            Err(e) => {
                log::warn!("Agent {} failed: {e}", agent.name());
            }
        }
    }

    if quantifier_result.npcs.confidence == QuantifierConfidence::Low {
        state.add_log(
            "[System] NPC detection uncertain — using room defaults".to_string(),
            None,
            LogType::System,
        );
    }

    let trigger_request = execute_freeaction_impl(
        &state,
        &crate::engine::action_processing::FreeActionContext {
            narration_text: &narration_text,
            user_input: &text,
            quantifier_result: &quantifier_result,
            world: &world,
            player: &player,
            all_npcs: &all_npcs,
            history: &history,
            llm_backend: backend.as_ref(),
        },
    );

    match trigger_request {
        Ok(turn_result) => {
            let mut next_state = turn_result.next_state;

            if let Some(request) = turn_result.trigger_continuation {
                next_state.narrative.generation.status = GenerationStatus::Generating;
                next_state.narrative.generation.phase = GenerationPhase::GeneratingEvent;
                next_state.narrative.last_trigger = Some(request.stored.clone());
                save_committed_state(ctx, &next_state, format!("pre-event:{message_id}"), 0);

                let continuation_text = match backend.narrate_action_from_prompt(
                    &request.stored.system_prompt,
                    &request.stored.user_prompt,
                    request.stored.max_tokens,
                ) {
                    Ok(t) => t,
                    Err(e) => {
                        log::error!("Trigger narration failed: {e}");
                        next_state.add_log(
                            format!("[Trigger narration failed: {e}]"),
                            None,
                            LogType::System,
                        );
                        next_state.narrative.generation.status =
                            GenerationStatus::Error(format!("Error: {e}"));
                        save_state(ctx, &next_state, message_id.clone(), swipe_index);
                        return;
                    }
                };

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
                }
            }

            next_state.narrative.generation.status = GenerationStatus::Idle;
            next_state.narrative.generation.phase = GenerationPhase::default();
            save_state(ctx, &next_state, message_id.clone(), swipe_index);
        }
        Err(e) => {
            save_pipeline_error(ctx, &message_id, swipe_index, format!("Error: {e}"));
        }
    }
}
