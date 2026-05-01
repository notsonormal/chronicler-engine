use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
};
use serde::{Deserialize, Serialize};

use crate::engine::logic::{get_available_exits, get_current_room};
use crate::engine::parser::parse_command;
use crate::error::Result;
use crate::model::state::{GameState, LogType};
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, StoryLogTemplate, VisualSidebarTemplate,
};

const MAX_LOG_DISPLAY: usize = 50;

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

#[derive(Deserialize, Serialize)]
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
        let game_service = state.game_service.clone();

        std::thread::spawn(move || {
            // Small delay to let inner threads start their guards first
            std::thread::sleep(std::time::Duration::from_millis(50));

            game_service.execute_action(state_clone, cmd, pname);
        });
    }

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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape_basic() {
        assert_eq!(html_escape("<test>"), "&lt;test&gt;");
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("foo & bar"), "foo &amp; bar");
    }

    #[test]
    fn test_html_escape_quotes() {
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_html_escape_all() {
        assert_eq!(
            html_escape("<foo & \"bar\">"),
            "&lt;foo &amp; &quot;bar&quot;&gt;"
        );
    }

    #[test]
    fn test_html_escape_empty() {
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn test_render_error_basic() {
        let result = render_error("Test error message");
        assert!(result.contains("error-message"));
        assert!(result.contains("Test error message"));
    }

    #[test]
    fn test_render_error_html_escaped() {
        let result = render_error("<script>alert('xss')</script>");
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_action_form_deserialization() {
        let form: ActionForm = serde_json::from_str(r#"{"command": "look"}"#).unwrap();
        assert_eq!(form.command, "look");
    }

    #[test]
    fn test_action_form_empty_command() {
        let form: ActionForm = serde_json::from_str(r#"{"command": ""}"#).unwrap();
        assert!(form.command.is_empty());
    }

    #[test]
    fn test_edit_history_form_deserialization() {
        let form: EditHistoryForm = serde_json::from_str(r#"{"text": "Modified text"}"#).unwrap();
        assert_eq!(form.text, "Modified text");
    }

    #[test]
    fn test_render_error_empty_message() {
        let result = render_error("");
        assert!(result.contains("error-message"));
        assert!(result.contains("Error:"));
    }

    #[test]
    fn test_html_escape_newline() {
        // [DOC: docs/reference/testing.md]
        // Newlines should be preserved (not converted to &lt;br&gt;)
        assert_eq!(html_escape("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn test_html_escape_backtick() {
        // [DOC: docs/reference/testing.md]
        // Backticks should be preserved
        assert_eq!(html_escape("`code`"), "`code`");
    }

    #[test]
    fn test_html_escape_unicode_characters() {
        // [DOC: docs/reference/testing.md]
        // Unicode should be preserved
        assert_eq!(html_escape("日本語"), "日本語");
    }

    #[test]
    fn test_html_escape_multiple_special_chars() {
        assert_eq!(
            html_escape("<div class=\"test\">Hello & \"World\"</div>"),
            "&lt;div class=&quot;test&quot;&gt;Hello &amp; &quot;World&quot;&lt;/div&gt;"
        );
    }

    #[test]
    fn test_html_escape_repeated_escaping() {
        // [DOC: docs/reference/testing.md]
        // html_escape is NOT idempotent - running it twice double-encodes
        let escaped = html_escape("<&>");
        assert_eq!(escaped, "&lt;&amp;&gt;");
        assert_eq!(html_escape(&escaped), "&amp;lt;&amp;amp;&amp;gt;");
    }

    #[test]
    fn test_action_form_with_whitespace_command() {
        let form: ActionForm = serde_json::from_str(r#"{"command": "  look  "}"#).unwrap();
        assert_eq!(form.command, "  look  ");
    }

    #[test]
    fn test_action_form_with_special_characters() {
        let form: ActionForm =
            serde_json::from_str(r#"{"command": "go north & talk to guard"}"#).unwrap();
        assert_eq!(form.command, "go north & talk to guard");
    }

    #[test]
    fn test_edit_history_form_empty_text() {
        let form: EditHistoryForm = serde_json::from_str(r#"{"text": ""}"#).unwrap();
        assert!(form.text.is_empty());
    }

    #[test]
    fn test_edit_history_form_with_newlines() {
        let form: EditHistoryForm =
            serde_json::from_str(r#"{"text": "Line1\nLine2\nLine3"}"#).unwrap();
        assert!(form.text.contains('\n'));
    }

    #[test]
    fn test_action_form_deserialize_unicode() {
        let form: ActionForm = serde_json::from_str(r#"{"command": "こんにちは"}"#).unwrap();
        assert_eq!(form.command, "こんにちは");
    }

    #[test]
    fn test_render_error_long_message() {
        let long_msg = "x".repeat(10000).to_string();
        let result = render_error(&long_msg);
        assert!(result.len() > 10000);
        assert!(result.contains(&long_msg[..100]));
    }

    #[test]
    fn test_html_escape_only_ampersand() {
        assert_eq!(html_escape("&"), "&amp;");
    }

    #[test]
    fn test_html_escape_only_lt() {
        assert_eq!(html_escape("<"), "&lt;");
    }

    #[test]
    fn test_html_escape_only_gt() {
        assert_eq!(html_escape(">"), "&gt;");
    }

    #[test]
    fn test_action_form_roundtrip() {
        let original = ActionForm {
            command: "test command".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: ActionForm = serde_json::from_str(&json).unwrap();
        assert_eq!(original.command, parsed.command);
    }

    #[test]
    fn test_edit_history_form_roundtrip() {
        let original = EditHistoryForm {
            text: "new text".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: EditHistoryForm = serde_json::from_str(&json).unwrap();
        assert_eq!(original.text, parsed.text);
    }

}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    text: String,
}

/// Edit a history entry by ID
pub async fn edit_history_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    Form(form): Form<EditHistoryForm>,
) -> (StatusCode, String) {
    let result = state
        .state
        .lock()
        .map(|mut guard| guard.edit_log(id, form.text));

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            "<span class=\"status ready\">Edited</span>".to_string(),
        ),
        Ok(Err(e)) => (StatusCode::NOT_FOUND, render_error(&e.to_string())),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            render_error("Failed to lock state"),
        ),
    }
}

/// Retry the last AI response
pub async fn retry_handler(State(state): State<AppState>) -> (StatusCode, String) {
    let has_input = state
        .state
        .lock()
        .map(|g| g.get_last_input_text().is_some())
        .unwrap_or(false);
    if !has_input {
        return (StatusCode::BAD_REQUEST, render_error("No input to retry"));
    }

    let state_clone = state.state.clone();
    let game_service = state.game_service.clone();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        game_service.retry_last_response(state_clone);
    });

    (
        StatusCode::OK,
        "<span class=\"status ready\">Retrying...</span>".to_string(),
    )
}
