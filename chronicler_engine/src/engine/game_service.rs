//! [DOC: docs/architecture/system.md]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::engine::action::Action;
use crate::engine::action_processing::{
    commit_trigger_narration, execute_freeaction_impl, get_static_npcs,
};
use crate::engine::logic::{find_room_in_map, get_current_room};
use crate::engine::parser::parse_command;
use crate::error::{EngineError, LlmFailure};
use crate::model::agent::{AgentContext, AgentResult, Confidence, ExecutionPhase, StatePatch};
use crate::model::character::NpcCard;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogType};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::world::WorldCard;
use crate::narrative::agents::quantifier::{
    MovementParseResult, QuantifierAgent, QuantifierBackendTrait, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};
use crate::narrative::agents::registry::AgentRegistry;
use crate::narrative::prompt::make_prompt_context;
use crate::storage::snapshot_storage::SnapshotStorage;

/// Context required by [`GameService`] to load and persist game state.
#[derive(Clone)]
pub struct GameServiceContext {
    pub snapshot_storage: Arc<dyn SnapshotStorage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<crate::model::map::MapDef>,
    pub player: Arc<crate::model::character::PlayerCard>,
    pub npcs: Arc<HashMap<String, NpcCard>>,
    pub starting_room_id: String,
    pub cancel_token: CancellationToken,
    /// Serialize async action processing to prevent snapshot race conditions.
    pub action_lock: Arc<Mutex<()>>,
}

impl GameServiceContext {
    /// Load the latest game state from snapshot storage.
    /// Panics if no snapshot exists — use only in tests where a snapshot was pre-seeded.
    #[cfg(test)]
    pub fn load_state(&self) -> GameState {
        let snapshot = match self.snapshot_storage.load_latest(None) {
            Ok(Some(s)) => s,
            Ok(None) => panic!("no snapshots found"),
            Err(e) => panic!("failed to load snapshot: {e}"),
        };
        GameState::from_snapshot(
            &snapshot,
            Arc::clone(&self.world),
            Arc::clone(&self.map),
            Arc::clone(&self.player),
            (*self.npcs).clone(),
        )
    }
}

pub trait GameService: Send + Sync {
    fn execute_action(&self, ctx: GameServiceContext, input: String, player_name: String);

    fn retry_last_response(&self, ctx: GameServiceContext);
}

pub struct DefaultGameService {
    llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
    agent_registry: AgentRegistry,
}

impl DefaultGameService {
    pub fn new() -> Self {
        let settings = crate::settings::load_settings().unwrap_or_default();
        let registry = AgentRegistry::from_configs(&settings.agents).unwrap_or_default();
        Self {
            llm_backend: Arc::from(crate::narrative::llm::get_llm_backend()),
            agent_registry: registry,
        }
    }

    pub fn with_backends(
        llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
        agent_registry: AgentRegistry,
    ) -> Self {
        Self {
            llm_backend,
            agent_registry,
        }
    }

    /// Convenience constructor for tests that only need a mock quantifier.
    pub fn with_mock_quantifier(
        llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
        quantifier_backend: Arc<dyn QuantifierBackendTrait>,
    ) -> Self {
        let agent = QuantifierAgent::with_backend("quantifier".to_string(), quantifier_backend);
        let registry = AgentRegistry::with_agent(Box::new(agent));
        Self {
            llm_backend,
            agent_registry: registry,
        }
    }
}

impl Default for DefaultGameService {
    fn default() -> Self {
        DefaultGameService::new()
    }
}

fn load_state(ctx: &GameServiceContext) -> GameState {
    match ctx.snapshot_storage.load_latest(None) {
        Ok(Some(snapshot)) => GameState::from_snapshot(
            &snapshot,
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).clone(),
        ),
        _ => GameState::new(
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).values().cloned().collect(),
            ctx.starting_room_id.clone(),
        ),
    }
}

fn save_state(ctx: &GameServiceContext, state: &GameState, message_id: String, swipe_index: u32) {
    let snapshot = GameStateSnapshot::from_game_state(state, message_id, swipe_index);
    if let Err(e) = ctx.snapshot_storage.save(&snapshot) {
        log::error!("Failed to save snapshot: {e}");
    }
}

fn map_llm_error(e: &EngineError) -> String {
    match e {
        EngineError::Llm(LlmFailure::Timeout) => "LLM Error: request timed out".to_string(),
        EngineError::Llm(LlmFailure::Network { url, detail }) => {
            format!("LLM Error: network error ({url}) — {detail}")
        }
        EngineError::Llm(LlmFailure::ParseError {
            expected_format, ..
        }) => {
            format!("LLM Error: unexpected response format (expected {expected_format})")
        }
        EngineError::Llm(LlmFailure::EmptyResponse) => "LLM Error: empty response".to_string(),
        EngineError::Llm(LlmFailure::Http { status, body }) => {
            format!("LLM Error: HTTP {status} — {body}")
        }
        EngineError::Narrative(nf) => format!("LLM Error: {nf}"),
        _ => format!("LLM Error: {e}"),
    }
}

impl GameService for DefaultGameService {
    fn execute_action(&self, ctx: GameServiceContext, input: String, _player_name: String) {
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
                save_state(&ctx, &state, message_id.clone(), 0);

                let room = map
                    .overworld
                    .regions
                    .iter()
                    .flat_map(|r| r.rooms.iter())
                    .find(|r| r.id == room_id);

                let Some(room) = room else {
                    let mut state = load_state(&ctx);
                    state.narrative.generation.status = GenerationStatus::Idle;
                    state.narrative.generation.phase = GenerationPhase::default();
                    save_state(&ctx, &state, message_id.clone(), 0);
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

                let backend = Arc::clone(&self.llm_backend);
                let narration_text = match backend.narrate_action(&context) {
                    Ok(t) => t,
                    Err(e) => {
                        let mut state = load_state(&ctx);
                        state.narrative.generation.status =
                            GenerationStatus::Error(map_llm_error(&e));
                        save_state(&ctx, &state, message_id.clone(), 0);
                        return;
                    }
                };

                if narration_text.trim().is_empty() {
                    let mut state = load_state(&ctx);
                    state.narrative.generation.status =
                        GenerationStatus::Error("LLM Error: empty response".to_string());
                    save_state(&ctx, &state, message_id.clone(), 0);
                    return;
                }

                let mut state = load_state(&ctx);
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

                for agent in self
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
                            next_state.narrative.generation.phase =
                                GenerationPhase::GeneratingEvent;
                            save_state(&ctx, &next_state, message_id.clone(), 0);

                            let continuation_text = match backend.narrate_action_from_prompt(
                                &request.system_prompt,
                                &request.user_prompt,
                                request.max_tokens,
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
                                    save_state(&ctx, &next_state, message_id.clone(), 0);
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
                        save_state(&ctx, &next_state, message_id.clone(), 0);
                    }
                    Err(e) => {
                        let mut state = load_state(&ctx);
                        state.narrative.generation.status =
                            GenerationStatus::Error(format!("Error: {e}"));
                        save_state(&ctx, &state, message_id.clone(), 0);
                    }
                }
            }
        }
    }

    fn retry_last_response(&self, ctx: GameServiceContext) {
        let (input_text, snapshot_message_id, snapshot_swipe_index) = {
            let snapshot = match ctx.snapshot_storage.load_latest(None) {
                Ok(Some(s)) => s,
                _ => {
                    log::error!("No snapshot to retry");
                    return;
                }
            };

            let state = GameState::from_snapshot(
                &snapshot,
                Arc::clone(&ctx.world),
                Arc::clone(&ctx.map),
                Arc::clone(&ctx.player),
                (*ctx.npcs).clone(),
            );

            match state.get_last_input_text() {
                Some((_sender, text)) => (text, snapshot.message_id, snapshot.swipe_index),
                None => {
                    log::error!("No input to retry");
                    return;
                }
            }
        };

        let (world, map, player, all_npcs, room_npc_ids, history_for_retry, current_room_id) = {
            let snapshot = match ctx.snapshot_storage.load_latest(None) {
                Ok(Some(s)) => s,
                _ => {
                    log::error!("No snapshot to retry");
                    return;
                }
            };

            let guard = GameState::from_snapshot(
                &snapshot,
                Arc::clone(&ctx.world),
                Arc::clone(&ctx.map),
                Arc::clone(&ctx.player),
                (*ctx.npcs).clone(),
            );

            let room_npc_ids = match get_current_room(&guard) {
                Ok(room) => room.npcs.clone(),
                Err(_) => vec![],
            };

            (
                Arc::clone(&guard.world),
                Arc::clone(&guard.map),
                Arc::clone(&guard.player),
                guard.npcs.values().cloned().collect::<Vec<_>>(),
                room_npc_ids,
                guard.get_history_context_for_retry(),
                guard.movement.current_room_id.clone(),
            )
        };

        let backend = Arc::clone(&self.llm_backend);

        let Some(room) = find_room_in_map(&map, &current_room_id) else {
            let mut state = load_state(&ctx);
            state.narrative.generation.status =
                GenerationStatus::Error("Retry failed: room not found".to_string());
            save_state(
                &ctx,
                &state,
                snapshot_message_id.clone(),
                snapshot_swipe_index + 1,
            );
            return;
        };

        let nearby_npcs: Vec<NpcCard> = all_npcs
            .iter()
            .filter(|npc| room_npc_ids.contains(&npc.id))
            .cloned()
            .collect();
        let context = make_prompt_context(
            &world,
            room,
            &all_npcs,
            &nearby_npcs,
            &player,
            &input_text,
            &history_for_retry,
        );

        let new_narration = match backend.narrate_action(&context) {
            Ok(t) => t,
            Err(e) => {
                let mut state = load_state(&ctx);
                state.narrative.generation.status = GenerationStatus::Error(map_llm_error(&e));
                save_state(
                    &ctx,
                    &state,
                    snapshot_message_id.clone(),
                    snapshot_swipe_index + 1,
                );
                return;
            }
        };

        if new_narration.trim().is_empty() {
            let mut state = load_state(&ctx);
            state.narrative.generation.status =
                GenerationStatus::Error("LLM Error: empty response".to_string());
            save_state(
                &ctx,
                &state,
                snapshot_message_id.clone(),
                snapshot_swipe_index + 1,
            );
            return;
        }

        let mut state = load_state(&ctx);
        if let Err(e) = state.replace_last_ai_response(new_narration) {
            state.narrative.generation.status =
                GenerationStatus::Error(format!("Retry failed: {e}"));
        } else {
            state.narrative.generation.status = GenerationStatus::Idle;
            state.narrative.generation.phase = GenerationPhase::default();
        }
        save_state(
            &ctx,
            &state,
            snapshot_message_id.clone(),
            snapshot_swipe_index + 1,
        );
    }
}
