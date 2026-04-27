use std::sync::Arc;
use std::thread;

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
};
use serde::Deserialize;

use crate::engine::action_processing::apply_npc_events;
use crate::engine::logic::{get_available_exits, get_current_room};
use crate::engine::parser::parse_command;
use crate::error::Result;
use crate::model::character::NpcCard;
use crate::model::state::{GameState, LogType};
use crate::narrative::llm::get_llm_backend;
use crate::narrative::prompt::PromptContext;
use crate::narrative::quantifier::{
    MockQuantifierBackend, QuantifierBackendTrait, QuantifierConfidence, QuantifierPromptContext,
    QuantifierResult, RealQuantifierBackend, RoomInfo, compute_npc_events,
};
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, StoryLogTemplate, VisualSidebarTemplate,
};

use crate::engine::action_processing::{
    evaluate_and_narrate_triggers, get_static_npcs, handle_movement,
};

const MAX_LOG_DISPLAY: usize = 50;

/// [DOC: docs/reference/quantifier_prompt.md]
fn determine_npcs_in_room(
    state: &GameState,
    room_npc_ids: &[String],
    previous_room_npcs: &[NpcCard],
    player_action: &str,
) -> QuantifierResult {
    let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

    let room = match get_current_room(state) {
        Ok(r) => r,
        Err(_) => {
            log::warn!("[Quantifier] Cannot get current room, using static NPCs");
            return QuantifierResult {
                npcs: crate::narrative::quantifier::QuantifierParseResult {
                    npc_ids: get_static_npcs(state, room_npc_ids)
                        .iter()
                        .map(|n| n.id.clone())
                        .collect(),
                    confidence: QuantifierConfidence::Low,
                },
                movement: crate::narrative::quantifier::MovementParseResult {
                    movement_type: None,
                    destination: None,
                    confidence: QuantifierConfidence::Low,
                },
            };
        }
    };

    let recent_history: Vec<_> = state
        .narration_history
        .iter()
        .rev()
        .take(4)
        .rev()
        .cloned()
        .collect();

    let all_rooms: Vec<RoomInfo> = state
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
        room,
        previous_room_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &all_rooms,
        player_name: &state.player.sheet.name,
        recent_history: &recent_history,
        player_action,
    };

    // Use mock backend when LLM_BACKEND=mock, otherwise use real backend
    let use_mock = std::env::var("LLM_BACKEND").as_deref() == Ok("mock");
    let backend: Box<dyn QuantifierBackendTrait> = if use_mock {
        Box::new(MockQuantifierBackend::default())
    } else {
        Box::new(RealQuantifierBackend)
    };

    match backend.quantify_room(&context, room_npc_ids) {
        Ok(result) => match result.npcs.confidence {
            QuantifierConfidence::High | QuantifierConfidence::Medium => {
                log::info!("[Quantifier] Using dynamic NPCs: {:?}", result.npcs.npc_ids);
                let npc_cards: Vec<NpcCard> = result
                    .npcs
                    .npc_ids
                    .iter()
                    .filter_map(|id| state.npcs.get(id).cloned())
                    .collect();
                QuantifierResult {
                    npcs: crate::narrative::quantifier::QuantifierParseResult {
                        npc_ids: npc_cards.iter().map(|n| n.id.clone()).collect(),
                        confidence: result.npcs.confidence,
                    },
                    movement: result.movement,
                }
            }
            QuantifierConfidence::Low => {
                log::info!("[Quantifier] Low confidence, using static NPCs");
                QuantifierResult {
                    npcs: crate::narrative::quantifier::QuantifierParseResult {
                        npc_ids: get_static_npcs(state, room_npc_ids)
                            .iter()
                            .map(|n| n.id.clone())
                            .collect(),
                        confidence: QuantifierConfidence::Low,
                    },
                    movement: result.movement,
                }
            }
        },
        Err(e) => {
            log::warn!("[Quantifier] Failed: {e}, using static NPCs");
            QuantifierResult {
                npcs: crate::narrative::quantifier::QuantifierParseResult {
                    npc_ids: get_static_npcs(state, room_npc_ids)
                        .iter()
                        .map(|n| n.id.clone())
                        .collect(),
                    confidence: QuantifierConfidence::Low,
                },
                movement: crate::narrative::quantifier::MovementParseResult {
                    movement_type: None,
                    destination: None,
                    confidence: QuantifierConfidence::Low,
                },
            }
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

    let image_path = room
        .image_path
        .clone()
        .or_else(|| state.world.default_room_image.clone());

    let npc_data: Vec<(String, String)> = if !state.npcs_in_area.is_empty() {
        state
            .npcs_in_area
            .iter()
            .filter_map(|npc| {
                // Defensive: only include NPCs that exist in state.npcs
                let npc = state.npcs.get(&npc.id)?;
                let image_path = npc.sheet.preferred_image()?.to_string();
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
                let image_path = npc.sheet.preferred_image()?.to_string();
                let name = npc.sheet.name.clone();
                Some((image_path, name))
            })
            .collect()
    };

    let template = VisualSidebarTemplate::new(image_path, room.name.clone(), npc_data);
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

    let is_generating = state_guard.generation_state.is_generating;
    let error_message = state_guard.generation_state.error_message.clone();
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

    let npc_data: Vec<(String, String)> = state_guard
        .npcs
        .iter()
        .filter_map(|(_npc_id, npc)| {
            let image = npc.sheet.preferred_image()?;
            let name = npc.sheet.name.clone();
            Some((image.to_string(), name))
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
                guard.generation_state.is_generating,
                guard.generation_state.error_message.clone(),
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

pub async fn reset_generating_handler(State(state): State<AppState>) -> Html<String> {
    let result = state
        .state
        .lock()
        .map(|mut guard| {
            guard.generation_state.is_generating = false;
            guard.generation_state.error_message = None;
            true
        })
        .unwrap_or(false);

    if result {
        Html("reset".to_string())
    } else {
        Html("failed".to_string())
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

/// [DOC: docs/system/game_flow.md]
pub async fn action_handler(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();
    if command.is_empty() {
        // Browser should catch invalid actions, but return error just in case
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "<span class=\"status error\">Enter a command</span>",
            ))
            .unwrap();
    }

    let (player_name, is_sync) = {
        // [DOC: docs/system/game_flow.md]
        let mut state_guard = match state.state.lock() {
            Ok(g) => g,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::new(String::new()))
                    .unwrap();
            }
        };

        let name = state_guard.player.sheet.name.clone();
        state_guard.add_log(command.clone(), Some(name.clone()), LogType::Input);

        let action = parse_command(&command);
        let is_sync = matches!(
            action,
            crate::engine::action::Action::Look
                | crate::engine::action::Action::Inventory
                | crate::engine::action::Action::Quit
        );

        if is_sync {
            process_sync_action(&mut state_guard, &action);
            state_guard.generation_state.is_generating = false;
        } else {
            state_guard.generation_state.is_generating = true;
        }
        state_guard.generation_state.error_message = None;

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

    // Return the current status immediately.
    // For sync actions, include HX-Trigger to also refresh the story log immediately.
    if is_sync {
        let mut headers = HeaderMap::new();
        headers.insert("HX-Trigger", "sync-action-complete".parse().unwrap());
        Response::builder()
            .status(StatusCode::OK)
            .header("HX-Trigger", "sync-action-complete")
            .body(Body::from("<span class=\"status ready\">Ready</span>"))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                "<span class=\"status thinking\">Thinking...</span>",
            ))
            .unwrap()
    }
}

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
        _ => {}
    }
}

fn process_action(state: Arc<std::sync::Mutex<GameState>>, input: String, _player_name: String) {
    // Note: We don't use GeneratingGuard here because async actions (WalkTo, FreeAction)
    // spawn inner threads that need to manage the is_generating flag themselves.

    let action = parse_command(&input);

    let mut state_guard = match state.lock() {
        Ok(g) => g,
        Err(_) => return, // Guard will still reset on drop
    };

    match action {
        crate::engine::action::Action::Quit => {
            state_guard.add_log("Goodbye!".to_string(), None, LogType::System);
            state_guard.generation_state.is_generating = false;
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
            state_guard.generation_state.is_generating = false;
        }
        crate::engine::action::Action::Talk(name, msg) => {
            let msg_str = msg.unwrap_or_default();
            state_guard.add_log(
                format!("You talk to {name}: {msg_str}"),
                None,
                LogType::System,
            );
            state_guard.generation_state.is_generating = false;
        }
        crate::engine::action::Action::Inventory => {
            state_guard.add_log(
                "Your inventory is empty.".to_string(),
                None,
                LogType::System,
            );
            state_guard.generation_state.is_generating = false;
        }
        crate::engine::action::Action::FreeAction(text) => {
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

                let Some(room) = room else {
                    if let Ok(mut state) = state_for_thread.lock() {
                        state.generation_state.is_generating = false;
                    }
                    return;
                };

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

                // First quantifier: detect NPCs in player action text and handle movement
                if let Ok(mut state) = state_for_thread.lock() {
                    let room_npc_ids = get_current_room(&state)
                        .map(|r| r.npcs.clone())
                        .unwrap_or_default();
                    let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();

                    let pre_narration_quantifier =
                        determine_npcs_in_room(&state, &room_npc_ids, &previous_room_npcs, &text);

                    handle_movement(
                        &mut state,
                        pre_narration_quantifier.movement.destination.as_deref(),
                        &pre_narration_quantifier.npcs.npc_ids,
                    );
                }

                // Generate main narration
                let Ok(narration_text) = backend.narrate_action(&context) else {
                    if let Ok(mut state) = state_for_thread.lock() {
                        state.generation_state.error_message =
                            Some("LLM Error: narration failed".to_string());
                    }
                    if let Ok(mut state) = state_for_thread.lock() {
                        state.generation_state.is_generating = false;
                    }
                    return;
                };

                // Second quantifier: detect NPCs that appeared in the narration
                if let Ok(mut state) = state_for_thread.lock() {
                    let room_npc_ids = get_current_room(&state)
                        .map(|r| r.npcs.clone())
                        .unwrap_or_default();
                    let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();
                    let previous_npc_ids: Vec<String> =
                        previous_room_npcs.iter().map(|n| n.id.clone()).collect();

                    let quantifier_result = determine_npcs_in_room(
                        &state,
                        &room_npc_ids,
                        &previous_room_npcs,
                        &narration_text,
                    );

                    let current_npcs: Vec<NpcCard> = quantifier_result
                        .npcs
                        .npc_ids
                        .iter()
                        .filter_map(|id| state.npcs.get(id).cloned())
                        .collect();
                    let current_npc_ids: Vec<String> =
                        current_npcs.iter().map(|n| n.id.clone()).collect();
                    let npcs_for_context = current_npcs.clone();
                    let trigger_context = PromptContext {
                        world: &world,
                        room,
                        all_npcs: &all_npcs,
                        npcs_in_area: &npcs_for_context,
                        player: &player,
                        user_message: &text,
                        history: &history,
                    };
                    state.add_log(narration_text.clone(), None, LogType::Narration);
                    state.npcs_in_area = current_npcs;

                    // Evaluate triggers BEFORE incrementing times_met
                    // (so TimesMet Eq 0 can fire on first detection)
                    evaluate_and_narrate_triggers(&mut state, &narration_text, &trigger_context, 3);

                    // Apply NPC events using the reusable function
                    let events = compute_npc_events(&previous_npc_ids, &current_npc_ids);
                    apply_npc_events(&mut state, &events.events);
                }

                if let Ok(mut state) = state_for_thread.lock() {
                    state.generation_state.is_generating = false;
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
