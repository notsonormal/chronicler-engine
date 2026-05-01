//! [DOC: docs/architecture/system.md]

use std::sync::{Arc, Mutex};
use std::thread;

use crate::engine::action::Action;
use crate::engine::action_processing::{execute_freeaction_impl, get_static_npcs};
use crate::engine::logic::{find_room_in_map, get_current_room};
use crate::engine::parser::parse_command;
use crate::model::character::NpcCard;
use crate::model::state::{GameState, LogType};
use crate::narrative::llm::get_llm_backend;
use crate::narrative::prompt::make_prompt_context;
use crate::narrative::quantifier::{
    MockQuantifierBackend, QuantifierBackendTrait, RealQuantifierBackend, determine_npcs_in_room,
};

/// Trait for game service that handles game orchestration logic.
pub trait GameService: Send + Sync {
    /// Execute a player action, spawning threads for async processing.
    fn execute_action(&self, state: Arc<Mutex<GameState>>, input: String, player_name: String);

    /// Retry the last AI response with a new LLM call.
    fn retry_last_response(&self, state: Arc<Mutex<GameState>>);
}

/// Default implementation of the GameService trait.
pub struct DefaultGameService;

impl DefaultGameService {
    pub fn new() -> Self {
        DefaultGameService
    }
}

impl Default for DefaultGameService {
    fn default() -> Self {
        DefaultGameService::new()
    }
}

impl GameService for DefaultGameService {
    fn execute_action(&self, state: Arc<Mutex<GameState>>, input: String, _player_name: String) {
        // Note: We don't use GeneratingGuard here because async actions (WalkTo, FreeAction)
        // spawn inner threads that need to manage the is_generating flag themselves.

        let action = parse_command(&input);

        let mut state_guard = match state.lock() {
            Ok(g) => g,
            Err(_) => return, // Guard will still reset on drop
        };

        match action {
            Action::Quit => {
                state_guard.add_log("Goodbye!".to_string(), None, LogType::System);
                state_guard.generation_state.is_generating = false;
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
                state_guard.generation_state.is_generating = false;
            }
            Action::Talk(name, msg) => {
                let msg_str = msg.unwrap_or_default();
                state_guard.add_log(
                    format!("You talk to {name}: {msg_str}"),
                    None,
                    LogType::System,
                );
                state_guard.generation_state.is_generating = false;
            }
            Action::Inventory => {
                state_guard.add_log(
                    "Your inventory is empty.".to_string(),
                    None,
                    LogType::System,
                );
                state_guard.generation_state.is_generating = false;
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
                thread::spawn(move || {
                    let room = map
                        .overworld
                        .regions
                        .iter()
                        .flat_map(|r| r.rooms.iter())
                        .find(|r| r.id == room_id);

                    let Some(room) = room else {
                        if let Ok(mut s) = state_for_thread.lock() {
                            s.generation_state.is_generating = false;
                        }
                        return;
                    };

                    let backend = get_llm_backend();
                    let context = make_prompt_context(
                        &world,
                        room,
                        &all_npcs,
                        &nearby_npcs,
                        &player,
                        &text,
                        &history,
                    );

                    let Ok(narration_text) = backend.narrate_action(&context) else {
                        if let Ok(mut s) = state_for_thread.lock() {
                            s.generation_state.error_message =
                                Some("LLM Error: narration failed".to_string());
                            s.generation_state.is_generating = false;
                        }
                        return;
                    };

                    // Determine NPCs via quantifier
                    let use_mock = std::env::var("LLM_BACKEND").as_deref() == Ok("mock");
                    let quantifier_backend: Box<dyn QuantifierBackendTrait> = if use_mock {
                        Box::new(MockQuantifierBackend::default())
                    } else {
                        Box::new(RealQuantifierBackend)
                    };

                    let quantifier_result = {
                        let state = match state_for_thread.lock() {
                            Ok(s) => s,
                            Err(_) => {
                                if let Ok(mut s) = state_for_thread.lock() {
                                    s.generation_state.is_generating = false;
                                }
                                return;
                            }
                        };
                        let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();
                        determine_npcs_in_room(
                            &state,
                            &room.npcs,
                            &previous_room_npcs,
                            &narration_text,
                            quantifier_backend.as_ref(),
                        )
                    };

                    // Call synchronous impl — handles all state mutations
                    let result = {
                        let mut state = match state_for_thread.lock() {
                            Ok(s) => s,
                            Err(_) => {
                                if let Ok(mut s) = state_for_thread.lock() {
                                    s.generation_state.is_generating = false;
                                }
                                return;
                            }
                        };
                        execute_freeaction_impl(
                            &mut state,
                            &narration_text,
                            &text,
                            &quantifier_result,
                            &world,
                            &map,
                            &player,
                            &all_npcs,
                            &room.npcs,
                            &history,
                        )
                    };

                    if let Err(e) = result {
                        if let Ok(mut s) = state_for_thread.lock() {
                            s.generation_state.error_message = Some(format!("Error: {e}"));
                        }
                    }

                    if let Ok(mut s) = state_for_thread.lock() {
                        s.generation_state.is_generating = false;
                    }
                });
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

        // Spawn background thread to call LLM and replace the response
        thread::spawn(move || {
            // Small delay to let any inner threads start their guards first
            std::thread::sleep(std::time::Duration::from_millis(50));

            let room = find_room_in_map(&map, &current_room_id);

            let Some(room) = room else {
                if let Ok(mut state) = state_clone.lock() {
                    state.generation_state.error_message =
                        Some("Retry failed: room not found".to_string());
                    state.generation_state.is_generating = false;
                }
                return;
            };

            let nearby_npcs: Vec<NpcCard> = all_npcs
                .iter()
                .filter(|npc| room_npc_ids.contains(&npc.id))
                .cloned()
                .collect();

            let backend = get_llm_backend();
            let context = make_prompt_context(
                &world,
                room,
                &all_npcs,
                &nearby_npcs,
                &player,
                &input_text,
                &history_for_retry, // History excludes the AI response being retried
            );

            let new_narration = match backend.narrate_action(&context) {
                Ok(text) => text,
                Err(e) => {
                    if let Ok(mut state) = state_clone.lock() {
                        state.generation_state.error_message = Some(format!("LLM Error: {e}"));
                        state.generation_state.is_generating = false;
                    }
                    return;
                }
            };

            // Replace the last AI response with the new narration
            if let Ok(mut state) = state_clone.lock() {
                if let Err(e) = state.replace_last_ai_response(new_narration) {
                    state.generation_state.error_message = Some(format!("Retry failed: {e}"));
                }
                state.generation_state.is_generating = false;
            }
        });
    }
}
