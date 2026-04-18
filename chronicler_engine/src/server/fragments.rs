use std::sync::Arc;
use std::thread;

use askama::Template;
use axum::{
    extract::{Form, State},
    response::Html,
};
use serde::Deserialize;

use crate::engine::logic::{get_available_exits, get_current_room};
use crate::engine::parser::parse_command;
use crate::error::Result;
use crate::model::character::NpcCard;
use crate::model::state::{GameState, LogType};
use crate::narrative::llm::get_llm_backend;
use crate::narrative::prompt::PromptContext;
use crate::narrative::quantifier::{
    QuantifierBackend, QuantifierConfidence, QuantifierPromptContext,
};
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, StoryLogTemplate, VisualSidebarTemplate,
};

const MAX_LOG_DISPLAY: usize = 50;

/// Get NPCs from static room.npcs list in map.json.
fn get_static_npcs(state: &GameState, room_npc_ids: &[String]) -> Vec<NpcCard> {
    room_npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect()
}

/// Determine which NPCs are in the current room using the quantifier LLM.
/// Falls back to static room.npcs from map.json if the quantifier fails
/// or returns Low confidence.
fn determine_npcs_in_room(
    state: &GameState,
    room_npc_ids: &[String],
    previous_room_npcs: &[NpcCard],
    player_action: &str,
) -> Vec<NpcCard> {
    let api_key = match std::env::var("OPENROUTER_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            log::debug!("[Quantifier] No API key, using static NPCs");
            return get_static_npcs(state, room_npc_ids);
        }
    };

    let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

    // Get the current room
    let room = match get_current_room(state) {
        Ok(r) => r,
        Err(_) => {
            log::warn!("[Quantifier] Cannot get current room, using static NPCs");
            return get_static_npcs(state, room_npc_ids);
        }
    };

    // Get last 4 history entries
    let recent_history: Vec<_> = state
        .narration_history
        .iter()
        .rev()
        .take(4)
        .rev()
        .cloned()
        .collect();

    let context = QuantifierPromptContext {
        room,
        previous_room_npcs,
        all_known_npcs: &all_npcs,
        player_name: &state.player.sheet.name,
        recent_history: &recent_history,
        player_action,
    };

    let backend = QuantifierBackend;
    match backend.quantify_room(&api_key, &context, room_npc_ids) {
        Ok(result) => match result.confidence {
            QuantifierConfidence::High | QuantifierConfidence::Medium => {
                log::info!("[Quantifier] Using dynamic NPCs: {:?}", result.npc_ids);
                result
                    .npc_ids
                    .iter()
                    .filter_map(|id| state.npcs.get(id).cloned())
                    .collect()
            }
            QuantifierConfidence::Low => {
                log::info!("[Quantifier] Low confidence, using static NPCs");
                get_static_npcs(state, room_npc_ids)
            }
        },
        Err(e) => {
            log::warn!("[Quantifier] Failed: {e}, using static NPCs");
            get_static_npcs(state, room_npc_ids)
        }
    }
}

fn render_error(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}

fn render_header_unlocked(state: &GameState) -> Result<String> {
    let room = get_current_room(state)?;
    let template = HeaderTemplate {
        room_name: room.name.clone(),
    };
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_header(state: &AppState) -> Result<String> {
    let state_guard = state
        .state
        .lock()
        .map_err(|_| crate::error::EngineError::Config("Lock poisoned".into()))?;
    render_header_unlocked(&state_guard)
}

pub fn render_story_log(state: &AppState) -> Result<String> {
    let state_guard = state
        .state
        .lock()
        .map_err(|_| crate::error::EngineError::Config("Lock poisoned".into()))?;

    let entries: Vec<_> = state_guard
        .narration_history
        .iter()
        .take(MAX_LOG_DISPLAY)
        .cloned()
        .collect();
    let template = StoryLogTemplate::new(&entries);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

fn render_visual_sidebar_unlocked(state: &GameState) -> Result<String> {
    let room = get_current_room(state)?;

    // Collect NPC data: (image_path, name) pairs
    // Use npcs_in_area from state if available, otherwise fallback to room.npcs
    let npc_data: Vec<(String, String)> = if !state.npcs_in_area.is_empty() {
        state
            .npcs_in_area
            .iter()
            .filter_map(|npc| {
                // Defensive: only include NPCs that exist in state.npcs
                let npc = state.npcs.get(&npc.id)?;
                // Use headshot_image with fallback to image_path
                let image_path = npc
                    .sheet
                    .headshot_image
                    .as_ref()
                    .or(npc.sheet.image_path.as_ref())?
                    .clone();
                let name = npc.sheet.name.clone();
                Some((image_path, name))
            })
            .collect()
    } else {
        // Fallback to static room.npcs
        room.npcs
            .iter()
            .filter_map(|npc_id| {
                let npc = state.npcs.get(npc_id)?;
                // Use headshot_image with fallback to image_path
                let image_path = npc
                    .sheet
                    .headshot_image
                    .as_ref()
                    .or(npc.sheet.image_path.as_ref())?
                    .clone();
                let name = npc.sheet.name.clone();
                Some((image_path, name))
            })
            .collect()
    };

    let template = VisualSidebarTemplate::new(room.image_path.clone(), room.name.clone(), npc_data);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_visual_sidebar(state: &AppState) -> Result<String> {
    let state_guard = state
        .state
        .lock()
        .map_err(|_| crate::error::EngineError::Config("Lock poisoned".into()))?;
    render_visual_sidebar_unlocked(&state_guard)
}

pub fn render_action_area(state: &AppState) -> Result<String> {
    let state_guard = state
        .state
        .lock()
        .map_err(|_| crate::error::EngineError::Config("Lock poisoned".into()))?;

    let is_generating = state_guard.tui_state.is_generating;
    let error_message = state_guard.tui_state.error_message.clone();
    let exits = get_available_exits(&state_guard);
    drop(state_guard);

    let template = ActionAreaTemplate::new_with_error(is_generating, &exits, error_message);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub async fn header_fragment(State(state): State<AppState>) -> Html<String> {
    match render_header(&state) {
        Ok(html) => Html(html),
        Err(e) => {
            log::error!("header_fragment failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}

pub async fn story_log_fragment(State(state): State<AppState>) -> Html<String> {
    match render_story_log(&state) {
        Ok(html) => Html(html),
        Err(e) => {
            log::error!("story_log_fragment failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}

pub async fn visual_sidebar_fragment(State(state): State<AppState>) -> Html<String> {
    match render_visual_sidebar(&state) {
        Ok(html) => Html(html),
        Err(e) => {
            log::error!("visual_sidebar_fragment failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}

pub async fn action_area_fragment(State(state): State<AppState>) -> Html<String> {
    match render_action_area(&state) {
        Ok(html) => Html(html),
        Err(e) => {
            log::error!("action_area_fragment failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}

fn render_character_headshots(state: &AppState) -> Result<String> {
    use crate::server::templates::CharacterHeadshotsTemplate;
    use askama::Template;

    let state_guard = state
        .state
        .lock()
        .map_err(|_| crate::error::EngineError::Config("Lock poisoned".into()))?;

    // Collect NPC image data from game state
    let npc_data: Vec<(String, String)> = state_guard
        .npcs
        .iter()
        .filter_map(|(_npc_id, npc)| {
            // Use headshot_image with fallback to image_path
            let image = npc
                .sheet
                .headshot_image
                .as_ref()
                .or(npc.sheet.image_path.as_ref())?;
            let name = npc.sheet.name.clone();
            Some((image.clone(), name))
        })
        .collect();

    let template = CharacterHeadshotsTemplate::new(npc_data);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub async fn character_headshots_fragment(State(state): State<AppState>) -> Html<String> {
    match render_character_headshots(&state) {
        Ok(html) => Html(html),
        Err(e) => {
            log::error!("character_headshots_fragment failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}

pub async fn hints_handler(State(state): State<AppState>) -> Html<String> {
    match render_action_hints(&state) {
        Ok(hints) => Html(hints),
        Err(e) => {
            log::error!("hints_handler failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}

pub async fn status_ready_handler(State(_state): State<AppState>) -> Html<String> {
    Html("<span class=\"status ready\">Ready</span>".to_string())
}

pub async fn generating_status_handler(State(state): State<AppState>) -> Html<String> {
    let (is_generating, error_message) = state
        .state
        .lock()
        .map(|guard| {
            (
                guard.tui_state.is_generating,
                guard.tui_state.error_message.clone(),
            )
        })
        .unwrap_or((false, None));

    if let Some(err) = error_message {
        Html(format!("<span class=\"status error\">Error: {err}</span>"))
    } else if is_generating {
        Html("generating".to_string())
    } else {
        Html("idle".to_string())
    }
}

fn render_action_hints(state: &AppState) -> Result<String> {
    let state_guard = state
        .state
        .lock()
        .map_err(|_| crate::error::EngineError::Config("Lock poisoned".into()))?;

    let exits = get_available_exits(&state_guard);
    let available_actions = if exits.is_empty() {
        String::from(
            "<span class=\"action-hint\">[Look]</span> <span class=\"action-hint\">[Inventory]</span>",
        )
    } else {
        let exit_hints: String = exits
            .iter()
            .map(|e| format!("<span class=\"action-hint\">[{e}]</span>"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "<span class=\"action-hint\">[Look]</span> <span class=\"action-hint\">[Inventory]</span> {exit_hints}"
        )
    };

    Ok(available_actions)
}

#[derive(Deserialize)]
pub struct ActionForm {
    command: String,
}

pub async fn action_handler(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Html<String> {
    let command = form.command.trim().to_string();
    if command.is_empty() {
        // Return error status - browser should have caught this, but just in case
        return Html("<span class=\"status error\">Enter a command</span>".to_string());
    }

    // Get mutable state and add the input to the log
    let (player_name, is_sync) = {
        let mut state_guard = match state.state.lock() {
            Ok(g) => g,
            Err(_) => return Html(String::new()),
        };

        let name = state_guard.player.sheet.name.clone();
        state_guard.add_log(command.clone(), Some(name.clone()), LogType::Input);

        // For synchronous commands (look, inventory, quit), set generating=false immediately
        // For async commands (free actions), keep generating=true until LLM completes
        let action = parse_command(&command);
        let is_sync = matches!(
            action,
            crate::engine::action::Action::Look
                | crate::engine::action::Action::Inventory
                | crate::engine::action::Action::Quit
        );

        if is_sync {
            // Add the output immediately for sync commands
            process_sync_action(&mut state_guard, &action);
            state_guard.tui_state.is_generating = false;
        } else {
            state_guard.tui_state.is_generating = true;
        }
        state_guard.tui_state.error_message = None;

        (name, is_sync)
    };

    // For async actions, spawn a thread to process them
    if !is_sync {
        let state_clone = state.state.clone();
        let cmd = command;
        let pname = player_name;

        std::thread::spawn(move || {
            // Small delay to let inner threads start their guards first
            std::thread::sleep(std::time::Duration::from_millis(50));

            process_action(state_clone, cmd, pname);
        });
    }

    // Return the current status immediately
    if is_sync {
        Html("<span class=\"status ready\">Ready</span>".to_string())
    } else {
        Html("<span class=\"status thinking\">Thinking...</span>".to_string())
    }
}

/// Process synchronous actions (Look, Inventory, Quit) immediately
fn process_sync_action(state: &mut GameState, action: &crate::engine::action::Action) {
    match action {
        crate::engine::action::Action::Look => {
            if let Ok(room) = get_current_room(state) {
                state.add_log(
                    room.description.clone(),
                    Some(room.name.clone()),
                    LogType::Narration,
                );
            }
        }
        crate::engine::action::Action::Inventory => {
            state.add_log(
                "Your inventory is empty.".to_string(),
                None,
                LogType::System,
            );
        }
        crate::engine::action::Action::Quit => {
            state.add_log("Goodbye!".to_string(), None, LogType::System);
        }
        _ => {} // Other actions handled by process_action
    }
}

fn process_action(state: Arc<std::sync::Mutex<GameState>>, input: String, _player_name: String) {
    // Note: We don't use GeneratingGuard here because async actions (WalkTo, FreeAction)
    // spawn inner threads that need to manage the is_generating flag themselves.
    // The outer spawn (line 272-279) now uses a guard to ensure cleanup.

    let action = parse_command(&input);

    let mut state_guard = match state.lock() {
        Ok(g) => g,
        Err(_) => return, // Guard will still reset on drop
    };

    match action {
        crate::engine::action::Action::Quit => {
            state_guard.add_log("Goodbye!".to_string(), None, LogType::System);
            state_guard.tui_state.is_generating = false;
        }
        crate::engine::action::Action::Look => {
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
            state_guard.tui_state.is_generating = false;
        }
        crate::engine::action::Action::WalkTo(target) => {
            let result = crate::engine::logic::attempt_walk(&mut state_guard, &target);
            if let Err(e) = result {
                state_guard.add_log(e.to_string(), None, LogType::System);
                state_guard.tui_state.is_generating = false;
                return;
            }

            let (room_name, room_npc_ids);
            {
                let room = get_current_room(&state_guard).ok();
                room_name = room.as_ref().map(|r| r.name.clone());
                room_npc_ids = room.map(|r| r.npcs.clone()).unwrap_or_default();
            }

            // Add location entry (sender + empty text for is_location detection)
            if let Some(name) = room_name.clone() {
                state_guard.add_log(String::new(), Some(name), LogType::Narration);
            }

            // Determine NPCs in room using quantifier (with static fallback)
            // Note: previous_room_npcs is empty for now; future work can track
            // which NPCs were in the previous room for follow detection
            let player_action = format!(
                "{} enters the {}.",
                state_guard.player.sheet.name,
                room_name.as_deref().unwrap_or("room")
            );
            let nearby_npcs = determine_npcs_in_room(
                &state_guard,
                &room_npc_ids,
                &[], // No previous room tracking yet
                &player_action,
            );

            // Store quantifier result in game state for persistence
            state_guard.npcs_in_area = nearby_npcs.clone();

            // Get ALL NPCs from game state for prompt context
            let all_npcs: Vec<NpcCard> = state_guard.npcs.values().cloned().collect();

            // Generate LLM narration for arrival
            let world = Arc::clone(&state_guard.world);
            let map = Arc::clone(&state_guard.map);
            let player = Arc::clone(&state_guard.player);
            let room_id = state_guard.current_room_id.clone();
            let history = state_guard.narration_history.clone();
            drop(state_guard);

            let state_for_thread = state.clone();
            thread::spawn(move || {
                let room = map
                    .overworld
                    .regions
                    .iter()
                    .flat_map(|r| r.rooms.iter())
                    .find(|r| r.id == room_id);

                if let Some(room) = room {
                    let backend = get_llm_backend();
                    let context = PromptContext {
                        world: &world,
                        room,
                        all_npcs: &all_npcs,
                        npcs_in_area: &nearby_npcs,
                        player: &player,
                        user_message: "", // narrate_arrival creates its own message
                        history: &history,
                    };
                    let narration = backend.narrate_arrival(&context);
                    match narration {
                        Ok(text) => {
                            if let Ok(mut state) = state_for_thread.lock() {
                                // Location entry already added above, just add narration text
                                state.add_log(text.clone(), None, LogType::Narration);
                                state.tui_state.is_generating = false;

                                // Re-quantify NPCs after EVERY LLM generation
                                // The LLM decides who should be in the room based on narrative context
                                let room_npc_ids = get_current_room(&state)
                                    .map(|r| r.npcs.clone())
                                    .unwrap_or_default();
                                let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();
                                let new_npcs = determine_npcs_in_room(
                                    &state,
                                    &room_npc_ids,
                                    &previous_room_npcs,
                                    "re-quantify after narration",
                                );
                                state.npcs_in_area = new_npcs;
                            }
                        }
                        Err(e) => {
                            log::error!("LLM arrival failed: {e}");
                            if let Ok(mut state) = state_for_thread.lock() {
                                state.tui_state.error_message = Some(format!("LLM Error: {e}"));
                                state.tui_state.is_generating = false;
                            }
                        }
                    }
                }
            });
        }
        crate::engine::action::Action::Talk(name, msg) => {
            // Simplified - in full implementation, would generate dialogue via LLM
            let msg_str = msg.unwrap_or_default();
            state_guard.add_log(
                format!("You talk to {name}: {msg_str}"),
                None,
                LogType::System,
            );
            state_guard.tui_state.is_generating = false;
        }
        crate::engine::action::Action::Inventory => {
            state_guard.add_log(
                "Your inventory is empty.".to_string(),
                None,
                LogType::System,
            );
            state_guard.tui_state.is_generating = false;
        }
        crate::engine::action::Action::FreeAction(text) => {
            // Generate LLM response for free action
            let world = Arc::clone(&state_guard.world);
            let map = Arc::clone(&state_guard.map);
            let player = Arc::clone(&state_guard.player);
            let room_id = state_guard.current_room_id.clone();
            let history = state_guard.narration_history.clone();
            // Get nearby NPCs from current room (static lookup for free actions)
            let room_npc_ids = get_current_room(&state_guard)
                .map(|r| r.npcs.clone())
                .unwrap_or_default();
            let nearby_npcs = get_static_npcs(&state_guard, &room_npc_ids);
            // Get ALL NPCs from game state for prompt context
            let all_npcs: Vec<NpcCard> = state_guard.npcs.values().cloned().collect();
            let text = text.clone();
            drop(state_guard);

            let state_for_thread = state.clone();
            thread::spawn(move || {
                let room = map
                    .overworld
                    .regions
                    .iter()
                    .flat_map(|r| r.rooms.iter())
                    .find(|r| r.id == room_id);

                if let Some(room) = room {
                    let backend = get_llm_backend();
                    let context = PromptContext {
                        world: &world,
                        room,
                        all_npcs: &all_npcs,
                        npcs_in_area: &nearby_npcs,
                        player: &player,
                        user_message: &text,
                        history: &history,
                    };
                    let narration = backend.narrate_action(&context);
                    match narration {
                        Ok(text) => {
                            if let Ok(mut state) = state_for_thread.lock() {
                                state.add_log(text, None, LogType::Narration);
                                state.tui_state.is_generating = false;

                                // Re-quantify NPCs after EVERY LLM generation
                                // The LLM decides who should be in the room based on narrative context
                                let room_npc_ids = get_current_room(&state)
                                    .map(|r| r.npcs.clone())
                                    .unwrap_or_default();
                                let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();
                                let new_npcs = determine_npcs_in_room(
                                    &state,
                                    &room_npc_ids,
                                    &previous_room_npcs,
                                    "re-quantify after narration",
                                );
                                state.npcs_in_area = new_npcs;
                            }
                        }
                        Err(e) => {
                            log::error!("LLM narration failed: {e}");
                            if let Ok(mut state) = state_for_thread.lock() {
                                // Set error message for UI instead of adding to chat log
                                state.tui_state.error_message = Some(format!("LLM Error: {e}"));
                                state.tui_state.is_generating = false;
                            }
                        }
                    }
                }
            });
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
