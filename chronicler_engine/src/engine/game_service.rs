//! [DOC: docs/architecture/system.md]

use std::sync::{Arc, Mutex};

use crate::engine::action::Action;
use crate::engine::action_processing::{execute_freeaction_impl, get_static_npcs};
use crate::engine::logic::{find_room_in_map, get_current_room};
use crate::engine::parser::parse_command;
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::state::{GameState, LogType};
use crate::narrative::prompt::make_prompt_context;
use crate::narrative::quantifier::{QuantifierBackendTrait, determine_npcs_in_room};

pub trait GameService: Send + Sync {
    fn execute_action(&self, state: Arc<Mutex<GameState>>, input: String, player_name: String);

    fn retry_last_response(&self, state: Arc<Mutex<GameState>>);
}

pub struct DefaultGameService {
    llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
    quantifier_backend: Arc<dyn QuantifierBackendTrait>,
}

impl DefaultGameService {
    pub fn new() -> Self {
        Self {
            llm_backend: Arc::from(crate::narrative::llm::get_llm_backend()),
            quantifier_backend: Arc::from(crate::narrative::quantifier::get_quantifier_backend()),
        }
    }

    pub fn with_backends(
        llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
        quantifier_backend: Arc<dyn QuantifierBackendTrait>,
    ) -> Self {
        Self {
            llm_backend,
            quantifier_backend,
        }
    }
}

impl Default for DefaultGameService {
    fn default() -> Self {
        DefaultGameService::new()
    }
}

fn with_state_lock<T>(
    state: &Arc<Mutex<GameState>>,
    f: impl FnOnce(&mut GameState) -> T,
) -> Option<T> {
    state.lock().ok().map(|mut guard| f(&mut guard))
}

fn reset_generating(state: &Arc<Mutex<GameState>>) {
    if let Ok(mut s) = state.lock() {
        s.generation_state.status = crate::model::state::GenerationStatus::Idle;
        s.generation_state.phase = crate::model::state::GenerationPhase::default();
    }
}

fn set_phase(state: &Arc<Mutex<GameState>>, phase: crate::model::state::GenerationPhase) {
    if let Ok(mut s) = state.lock() {
        s.generation_state.status = crate::model::state::GenerationStatus::Generating;
        s.generation_state.phase = phase;
    }
}

fn set_error_and_reset(state: &Arc<Mutex<GameState>>, message: String) {
    if let Ok(mut s) = state.lock() {
        s.generation_state.status = crate::model::state::GenerationStatus::Error(message);
    }
}

fn map_llm_error(e: &EngineError) -> String {
    match e {
        EngineError::Narrative(msg) if msg.contains("timed out") => {
            "LLM Error: request timed out".to_string()
        }
        EngineError::Narrative(msg) if msg.contains("Failed to read response body") => {
            "LLM Error: response incomplete".to_string()
        }
        EngineError::Narrative(msg) if msg.contains("parse") => {
            "LLM Error: unexpected response format".to_string()
        }
        EngineError::LlmEmptyResponse => "LLM Error: empty response".to_string(),
        _ => format!("LLM Error: {e}"),
    }
}

impl GameService for DefaultGameService {
    fn execute_action(&self, state: Arc<Mutex<GameState>>, input: String, _player_name: String) {
        // NOTE: async actions manage is_generating themselves.

        let action = parse_command(&input);

        let mut state_guard = match state.lock() {
            Ok(g) => g,
            Err(_) => return, // Guard will still reset on drop
        };

        match action {
            Action::Quit => {
                state_guard.add_log("Goodbye!".to_string(), None, LogType::System);
                state_guard.generation_state.status = crate::model::state::GenerationStatus::Idle;
            }
            Action::Look => {
                let room_name;
                let room_desc;
                {
                    let room = get_current_room(&state_guard).ok();
                    room_name = room.as_ref().map(|r| r.name.clone());
                    room_desc = room.map(|r| r.description.clone());
                }
                if let Some(name) = room_name {
                    if let Some(desc) = room_desc {
                        state_guard.add_log(desc, Some(name), LogType::Narration);
                    }
                }
                state_guard.generation_state.status = crate::model::state::GenerationStatus::Idle;
            }
            Action::Talk(name, msg) => {
                let msg_str = msg.unwrap_or_default();
                state_guard.add_log(
                    format!("You talk to {name}: {msg_str}"),
                    None,
                    LogType::System,
                );
                state_guard.generation_state.status = crate::model::state::GenerationStatus::Idle;
            }
            Action::Inventory => {
                state_guard.add_log(
                    "Your inventory is empty.".to_string(),
                    None,
                    LogType::System,
                );
                state_guard.generation_state.status = crate::model::state::GenerationStatus::Idle;
            }
            Action::FreeAction(text) => {
                let world = Arc::clone(&state_guard.world);
                let map = Arc::clone(&state_guard.map);
                let player = Arc::clone(&state_guard.player);
                let room_id = state_guard.current_room_id.clone();
                let history = state_guard.narration_history.clone();
                let room_npc_ids = get_current_room(&state_guard)
                    .map(|r| r.npcs.clone())
                    .unwrap_or_default();
                let nearby_npcs = get_static_npcs(&state_guard, &room_npc_ids);
                let all_npcs: Vec<NpcCard> = state_guard.npcs.values().cloned().collect();

                drop(state_guard);

                let state_for_thread = state.clone();
                let backend = Arc::clone(&self.llm_backend);
                let quantifier_backend = Arc::clone(&self.quantifier_backend);

                let room = map
                    .overworld
                    .regions
                    .iter()
                    .flat_map(|r| r.rooms.iter())
                    .find(|r| r.id == room_id);

                let Some(room) = room else {
                    reset_generating(&state_for_thread);
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

                set_phase(
                    &state_for_thread,
                    crate::model::state::GenerationPhase::Narrating,
                );

                let narration_text = match backend.narrate_action(&context) {
                    Ok(t) => t,
                    Err(e) => {
                        set_error_and_reset(&state_for_thread, map_llm_error(&e));
                        return;
                    }
                };

                let quantifier_backend: Arc<dyn QuantifierBackendTrait> = quantifier_backend;

                set_phase(
                    &state_for_thread,
                    crate::model::state::GenerationPhase::Quantifying,
                );

                let quantifier_result = with_state_lock(&state_for_thread, |state| {
                    let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();
                    determine_npcs_in_room(
                        state,
                        &room.npcs,
                        &previous_room_npcs,
                        &narration_text,
                        quantifier_backend.as_ref(),
                    )
                });

                let Some(quantifier_result) = quantifier_result else {
                    reset_generating(&state_for_thread);
                    return;
                };

                let result = with_state_lock(&state_for_thread, |state| {
                    execute_freeaction_impl(
                        state,
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
                    )
                });

                if let Some(Err(e)) = result {
                    if let Ok(mut s) = state_for_thread.lock() {
                        s.generation_state.status =
                            crate::model::state::GenerationStatus::Error(format!("Error: {e}"));
                    }
                }

                reset_generating(&state_for_thread);
            }
        }
    }

    fn retry_last_response(&self, state: Arc<Mutex<GameState>>) {
        let (input_text, _player_name) = match state.lock() {
            Ok(guard) => match guard.get_last_input_text() {
                Some((sender, text)) => (text, sender),
                None => {
                    log::error!("No input to retry");
                    return;
                }
            },
            Err(_) => {
                log::error!("Failed to lock state");
                return;
            }
        };

        let (world, map, player, all_npcs, room_npc_ids, history_for_retry, current_room_id) = {
            let guard = match state.lock() {
                Ok(g) => g,
                Err(_) => {
                    log::error!("Failed to lock state");
                    return;
                }
            };

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
                guard.get_history_context_for_retry(), // Excludes the AI response being retried
                guard.current_room_id.clone(),
            )
        };

        let state_clone = state.clone();
        let backend = Arc::clone(&self.llm_backend);

        let Some(room) = find_room_in_map(&map, &current_room_id) else {
            set_error_and_reset(&state_clone, "Retry failed: room not found".to_string());
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
                set_error_and_reset(&state_clone, map_llm_error(&e));
                return;
            }
        };

        if let Ok(mut state) = state_clone.lock() {
            if let Err(e) = state.replace_last_ai_response(new_narration) {
                state.generation_state.status =
                    crate::model::state::GenerationStatus::Error(format!("Retry failed: {e}"));
            } else {
                state.generation_state.status = crate::model::state::GenerationStatus::Idle;
            }
        }
    }
}
