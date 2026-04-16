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
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, StoryLogTemplate, VisualSidebarTemplate,
};

const MAX_LOG_DISPLAY: usize = 50;

/// Render an error message for the UI
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

    // Use Askama template for compile-time validation
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
    let npc_data: Vec<(String, String)> = room
        .npcs
        .iter()
        .filter_map(|npc_id| {
            let npc = state.npcs.get(npc_id)?;
            let image_path = npc.sheet.image_path.as_ref()?.clone();
            let name = npc.sheet.name.clone();
            Some((image_path, name))
        })
        .collect();

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
    let exits = get_available_exits(&state_guard);
    drop(state_guard);

    // Use Askama template for compile-time validation
    let template = ActionAreaTemplate::new(is_generating, &exits);
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

/// Handler for action hints - returns just the hints HTML
pub async fn hints_handler(State(state): State<AppState>) -> Html<String> {
    match render_action_hints(&state) {
        Ok(hints) => Html(hints),
        Err(e) => {
            log::error!("hints_handler failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}

/// Handler for status ready reset
pub async fn status_ready_handler(State(_state): State<AppState>) -> Html<String> {
    Html("<span class=\"status ready\">Ready</span>".to_string())
}

/// Handler for generating status - returns whether LLM is currently generating
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

/// Render just the action hints div content
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
    let action = parse_command(&input);

    let mut state_guard = match state.lock() {
        Ok(g) => g,
        Err(_) => return,
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
            if let Some(name) = room_name {
                state_guard.add_log(String::new(), Some(name), LogType::Narration);
            }

            // Fetch NPCs from room's NPC IDs via state.npcs HashMap
            let mut nearby_npcs: Vec<NpcCard> = Vec::new();
            for npc_id in &room_npc_ids {
                if let Some(npc) = state_guard.npcs.get(npc_id) {
                    nearby_npcs.push(npc.clone());
                }
            }

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
                    let narration =
                        backend.narrate_arrival(&world, room, &nearby_npcs, &player, &history);
                    match narration {
                        Ok(text) => {
                            if let Ok(mut state) = state_for_thread.lock() {
                                // Location entry already added above, just add narration text
                                state.add_log(text, None, LogType::Narration);
                                state.tui_state.is_generating = false;
                            }
                        }
                        Err(e) => {
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
            // Get nearby NPCs (empty for now - in full implementation would fetch from room)
            let nearby_npcs: Vec<NpcCard> = vec![];
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
                    let narration = backend.narrate_action(
                        &world,
                        room,
                        &nearby_npcs,
                        &player,
                        &text,
                        &history,
                    );
                    match narration {
                        Ok(text) => {
                            if let Ok(mut state) = state_for_thread.lock() {
                                state.add_log(
                                    text,
                                    Some("Game Master".to_string()),
                                    LogType::Narration,
                                );
                                state.tui_state.is_generating = false;
                            }
                        }
                        Err(e) => {
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
