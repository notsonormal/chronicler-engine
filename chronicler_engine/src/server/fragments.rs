use std::sync::Arc;
use std::thread;

use axum::{
    extract::{Form, State},
    response::Html,
};
use serde::Deserialize;

use crate::engine::logic::{get_available_exits, get_current_room};
use crate::engine::parser::parse_command;
use crate::error::Result;
use crate::model::state::{GameState, LogEntry, LogType};
use crate::narrative::llm::get_llm_backend;
use crate::server::AppState;
use crate::server::Hub;

const MAX_LOG_DISPLAY: usize = 50;

fn render_header_unlocked(state: &GameState) -> Result<String> {
    let room = get_current_room(state)?;

    Ok(format!(
        "<div class=\"header\">\
            <span class=\"game-title\">Chronicler Engine</span>\
            <span class=\"location\">| {}</span>\
            <span class=\"connection-status connected\" id=\"connection-status\">Connected</span>\
        </div>",
        html_escape(&room.name)
    ))
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

    let logs: String = state_guard
        .narration_history
        .iter()
        .take(MAX_LOG_DISPLAY)
        .map(render_log_entry)
        .collect();

    Ok(logs)
}

fn render_log_entry(entry: &LogEntry) -> String {
    let color_class = match entry.log_type {
        LogType::Narration => "narration",
        LogType::Dialogue => "dialogue",
        LogType::System => "system",
        LogType::Input => "input",
    };

    // Format timestamp as HH:MM
    let timestamp = entry.timestamp.format("%H:%M").to_string();
    let timestamp_html = format!("<span class=\"timestamp\">{timestamp}</span>");

    let sender_html = entry
        .sender
        .as_ref()
        .map(|s| format!("<span class=\"sender\">{s}:</span> "))
        .unwrap_or_default();

    format!(
        "<div class=\"log-entry {}\">{}{}<span class=\"text\">{}</span></div>",
        color_class,
        timestamp_html,
        sender_html,
        html_escape(&entry.text)
    )
}

fn render_visual_sidebar_unlocked(state: &GameState) -> Result<String> {
    let room = get_current_room(state)?;

    let room_image = if let Some(path) = &room.image_path {
        format!(
            "<div class=\"image-container location-image\">\
                <img src=\"{}\" alt=\"{}\" />\
                <div class=\"image-label\">Location</div>\
            </div>",
            path,
            html_escape(&room.name)
        )
    } else {
        "<div class=\"image-container no-image\">\
            <div class=\"placeholder\">No Location Image</div>\
        </div>"
            .to_string()
    };

    let npc_images: String = room
        .npcs
        .iter()
        .filter_map(|npc_id| {
            let npc = state.npcs.get(npc_id)?;
            let image_path = npc.sheet.image_path.as_ref()?;
            let name = html_escape(&npc.sheet.name);
            Some(format!(
                "<div class=\"image-container npc-portrait\">\
                    <img src=\"{image_path}\" alt=\"{name}\" />\
                    <div class=\"image-label\">{name}</div>\
                </div>"
            ))
        })
        .collect();

    Ok(format!(
        "<div class=\"visual-sidebar\" id=\"visual-sidebar\">\
            {room_image}\
            <div class=\"npc-portraits\">{npc_images}</div>\
        </div>"
    ))
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
    let _input_text = if is_generating {
        "...The Game Master is thinking...".to_string()
    } else {
        state_guard.tui_state.input.clone()
    };

    // Get available exits for action hints
    let exits = get_available_exits(&state_guard);
    let available_actions = if exits.is_empty() {
        String::from("<span class=\"action-hint\">[Look] [Inventory]</span>")
    } else {
        let exit_hints: String = exits
            .iter()
            .map(|e| format!("[{e}]"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("<span class=\"action-hint\">[Look] [Inventory] {exit_hints}</span>")
    };
    drop(state_guard);

    let status_class = if is_generating {
        "status thinking"
    } else {
        "status ready"
    };
    let status_text = if is_generating {
        "Thinking..."
    } else {
        "Ready"
    };
    let disabled_attr = if is_generating { "disabled" } else { "" };

    Ok(format!(
        "<div class=\"action-area\" id=\"action-area\">\
            <form hx-post=\"/action\" hx-target=\"#action-area\" hx-swap=\"outerHTML\" class=\"command-form\">\
                <input type=\"text\" name=\"command\" placeholder=\"Enter command...\" value=\"\" {disabled_attr} autocomplete=\"off\" />\
                <button type=\"submit\" {disabled_attr}>Send</button>\
            </form>\
            <div class=\"action-hints\">{available_actions}</div>\
            <div class=\"{status_class}\">{status_text}</div>\
        </div>"
    ))
}

fn render_action_area_unlocked(state: &GameState) -> Result<String> {
    let is_generating = state.tui_state.is_generating;

    // Get available exits for action hints
    let exits = get_available_exits(state);
    let available_actions = if exits.is_empty() {
        String::from("<span class=\"action-hint\">[Look] [Inventory]</span>")
    } else {
        let exit_hints: String = exits
            .iter()
            .map(|e| format!("[{e}]"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("<span class=\"action-hint\">[Look] [Inventory] {exit_hints}</span>")
    };

    let status_class = if is_generating {
        "status thinking"
    } else {
        "status ready"
    };
    let status_text = if is_generating {
        "Thinking..."
    } else {
        "Ready"
    };
    let disabled_attr = if is_generating { "disabled" } else { "" };

    Ok(format!(
        "<div class=\"action-area\" id=\"action-area\">\
            <form hx-post=\"/action\" hx-target=\"#action-area\" hx-swap=\"outerHTML\" class=\"command-form\">\
                <input type=\"text\" name=\"command\" placeholder=\"Enter command...\" value=\"\" {disabled_attr} autocomplete=\"off\" />\
                <button type=\"submit\" {disabled_attr}>Send</button>\
            </form>\
            <div class=\"action-hints\">{available_actions}</div>\
            <div class=\"{status_class}\">{status_text}</div>\
        </div>"
    ))
}

pub async fn header_fragment(State(state): State<AppState>) -> Html<String> {
    Html(render_header(&state).unwrap_or_else(|_| String::new()))
}

pub async fn story_log_fragment(State(state): State<AppState>) -> Html<String> {
    Html(render_story_log(&state).unwrap_or_else(|_| String::new()))
}

pub async fn visual_sidebar_fragment(State(state): State<AppState>) -> Html<String> {
    Html(render_visual_sidebar(&state).unwrap_or_else(|_| String::new()))
}

pub async fn action_area_fragment(State(state): State<AppState>) -> Html<String> {
    Html(render_action_area(&state).unwrap_or_else(|_| String::new()))
}

/// Handler for action hints - returns just the hints HTML
pub async fn hints_handler(State(state): State<AppState>) -> Html<String> {
    let hints = render_action_hints(&state).unwrap_or_else(|_| String::new());
    Html(hints)
}

/// Handler for status ready reset
pub async fn status_ready_handler(State(_state): State<AppState>) -> Html<String> {
    Html("<span class=\"status ready\">Ready</span>".to_string())
}

/// Handler for generating status - returns whether LLM is currently generating
pub async fn generating_status_handler(State(state): State<AppState>) -> Html<String> {
    let is_generating = state
        .state
        .lock()
        .map(|guard| guard.tui_state.is_generating)
        .unwrap_or(false);

    if is_generating {
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
    let player_name = {
        let mut state_guard = match state.state.lock() {
            Ok(g) => g,
            Err(_) => return Html(String::new()),
        };

        let name = state_guard.player.sheet.name.clone();
        state_guard.add_log(command.clone(), Some(name.clone()), LogType::Input);
        state_guard.tui_state.is_generating = true;
        name
    };

    // Broadcast the new input
    if let Ok(html) = render_story_log(&state) {
        state.hub.broadcast(
            serde_json::json!({
                "type": "update",
                "event": "story-log",
                "fragment": "story-log",
                "html": html
            })
            .to_string(),
        );
    }

    // Process the action asynchronously
    let state_clone = state.state.clone();
    let hub_clone = state.hub.clone();
    let cmd = command;
    let pname = player_name;

    std::thread::spawn(move || {
        process_action(state_clone, hub_clone, cmd, pname);
    });

    // Return thinking status - goes into #status-display div
    Html("<span class=\"status thinking\">Thinking...</span>".to_string())
}

fn process_action(
    state: Arc<std::sync::Mutex<GameState>>,
    hub: Hub,
    input: String,
    _player_name: String,
) {
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
            drop(state_guard);
            broadcast_state(&state, &hub);
        }
        crate::engine::action::Action::WalkTo(target) => {
            let result = crate::engine::logic::attempt_walk(&mut state_guard, &target);
            if let Err(e) = result {
                state_guard.add_log(e.to_string(), None, LogType::System);
                state_guard.tui_state.is_generating = false;
                drop(state_guard);
                broadcast_state(&state, &hub);
                return;
            }

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

            // Generate LLM narration for arrival
            let world = Arc::clone(&state_guard.world);
            let map = Arc::clone(&state_guard.map);
            let player = Arc::clone(&state_guard.player);
            let room_id = state_guard.current_room_id.clone();
            drop(state_guard);

            let state_for_thread = state.clone();
            let hub_for_thread = hub.clone();
            thread::spawn(move || {
                let room = map
                    .overworld
                    .regions
                    .iter()
                    .flat_map(|r| r.rooms.iter())
                    .find(|r| r.id == room_id);

                if let Some(room) = room {
                    let backend = get_llm_backend();
                    let narration = backend.narrate_arrival(&world, room, &[], &player);
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
                            broadcast_state(&state_for_thread, &hub_for_thread);
                        }
                        Err(e) => {
                            if let Ok(mut state) = state_for_thread.lock() {
                                state.add_log(format!("Error: {e}"), None, LogType::System);
                                state.tui_state.is_generating = false;
                            }
                            broadcast_state(&state_for_thread, &hub_for_thread);
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
            drop(state_guard);
            broadcast_state(&state, &hub);
        }
        crate::engine::action::Action::Inventory => {
            state_guard.add_log(
                "Your inventory is empty.".to_string(),
                None,
                LogType::System,
            );
            state_guard.tui_state.is_generating = false;
            drop(state_guard);
            broadcast_state(&state, &hub);
        }
        crate::engine::action::Action::FreeAction(text) => {
            // Generate LLM response for free action
            let world = Arc::clone(&state_guard.world);
            let map = Arc::clone(&state_guard.map);
            let player = Arc::clone(&state_guard.player);
            let room_id = state_guard.current_room_id.clone();
            drop(state_guard);

            let state_for_thread = state.clone();
            let hub_for_thread = hub.clone();
            thread::spawn(move || {
                let room = map
                    .overworld
                    .regions
                    .iter()
                    .flat_map(|r| r.rooms.iter())
                    .find(|r| r.id == room_id);

                if let Some(room) = room {
                    let backend = get_llm_backend();
                    let narration = backend.narrate_action(&world, room, &[], &player, &text);
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
                            broadcast_state(&state_for_thread, &hub_for_thread);
                        }
                        Err(e) => {
                            if let Ok(mut state) = state_for_thread.lock() {
                                state.add_log(format!("Error: {e}"), None, LogType::System);
                                state.tui_state.is_generating = false;
                            }
                            broadcast_state(&state_for_thread, &hub_for_thread);
                        }
                    }
                }
            });
        }
    }

    // Initial broadcast after action processing
    broadcast_state(&state, &hub);
}

fn broadcast_state(state: &Arc<std::sync::Mutex<GameState>>, hub: &Hub) {
    // Get a snapshot of state for rendering
    let (logs_html, header_html, action_html, sidebar_html) = {
        let state_guard = match state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        let logs: String = state_guard
            .narration_history
            .iter()
            .take(MAX_LOG_DISPLAY)
            .map(render_log_entry)
            .collect();
        let logs_html = format!("<div class=\"story-log\" id=\"story-log\">{logs}</div>");

        // Render header
        let header_html = render_header_unlocked(&state_guard).unwrap_or_default();

        // Render action area
        let action_html = render_action_area_unlocked(&state_guard).unwrap_or_default();

        // Render sidebar (needs state but we've released the lock)
        let sidebar_html = render_visual_sidebar_unlocked(&state_guard).unwrap_or_default();

        (logs_html, header_html, action_html, sidebar_html)
    };

    // Broadcast story log update
    hub.broadcast(
        serde_json::json!({
            "type": "update",
            "event": "story-log",
            "fragment": "story-log",
            "html": logs_html
        })
        .to_string(),
    );

    // Broadcast header update
    hub.broadcast(
        serde_json::json!({
            "type": "update",
            "event": "header",
            "fragment": "header",
            "html": header_html
        })
        .to_string(),
    );

    // Broadcast action area update
    hub.broadcast(
        serde_json::json!({
            "type": "update",
            "event": "action-area",
            "fragment": "action-area",
            "html": action_html
        })
        .to_string(),
    );

    // Broadcast sidebar update
    hub.broadcast(
        serde_json::json!({
            "type": "update",
            "event": "visual-sidebar",
            "fragment": "visual-sidebar",
            "html": sidebar_html
        })
        .to_string(),
    );
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
