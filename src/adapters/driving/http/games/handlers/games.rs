//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Games fragment handlers

use std::str::FromStr;

use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::Response,
};

use crate::adapters::driving::http::AppState;
use crate::application::errors::ApplicationError;
use crate::domain::model::game::Game;
use crate::domain::model::prompt_preset::PresetType;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};

use crate::adapters::driving::http::games::templates::games::{
    GamePostureTemplate, GamesPanelTemplate, PersonaRowView,
};
use crate::adapters::driving::http::utils::error::render_error;
use crate::adapters::driving::http::utils::response::{internal_error, ok, ok_refresh};
use crate::adapters::driving::http::utils::view_mappers::game_to_view;

pub async fn list_games_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let Ok(games) = state.game_catalogue.list_games() else {
        return internal_error("Failed to list games");
    };

    let active_id = state.game_catalogue.current_game_id();
    let mut active_game = None;
    let saved_games: Vec<_> = games
        .into_iter()
        .filter_map(|g| {
            if g.id == active_id {
                active_game = Some(game_to_view(g));
                None
            } else {
                Some(game_to_view(g))
            }
        })
        .collect();

    let Ok(worlds) = state.world_catalogue.list_worlds() else {
        return internal_error("Failed to list worlds");
    };

    let personas: Vec<PersonaRowView> = match state.persona_catalogue.list_personas() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Failed to load personas: {e}");
            Vec::new()
        }
    }
    .into_iter()
    .map(|p| PersonaRowView {
        key: p.key,
        name: p.sheet.name,
    })
    .collect();

    let template = GamesPanelTemplate {
        active_game,
        saved_games,
        worlds,
        personas,
        posture_html: match state.game_catalogue.current_game() {
            Ok(Some(game)) => posture_controls_html(&state, &game),
            Ok(None) => String::new(),
            Err(e) => return internal_error(format!("Failed to load active game: {e}")),
        },
    };

    ok(template.render().unwrap_or_default())
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateGameForm {
    pub world_key: String,
    pub persona_key: String,
}

pub async fn create_game_handler(
    State(state): State<AppState>,
    Form(form): Form<CreateGameForm>,
) -> Result<Response, ApplicationError> {
    state
        .game_catalogue
        .create_game(&form.world_key, &form.persona_key)?;
    Ok(ok_refresh())
}

pub async fn switch_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Response, ApplicationError> {
    state.game_catalogue.switch_game(id)?;
    Ok(ok_refresh())
}

pub async fn delete_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Response, ApplicationError> {
    state.game_catalogue.delete_game(id)?;
    Ok(ok(""))
}

#[derive(Debug, serde::Deserialize)]
pub struct GameModeForm {
    pub narrator_mode: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GamePostureForm {
    pub narrative_perspective: String,
    pub narrative_tense: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GamePresetsForm {
    pub system_preset_id: String,
    pub quantifier_preset_id: String,
    pub impersonate_preset_id: String,
}

/// Per-game mode-switch action: set mode, retarget presets to the new
/// mode's registry bundle, and nudge perspective. Deliberately not folded
/// into the generic posture save.
pub async fn switch_game_mode_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Form(form): Form<GameModeForm>,
) -> Response {
    let mode = NarratorMode::from_str(&form.narrator_mode);
    let game = match mode {
        Ok(mode) => state.game_catalogue.switch_mode(id, mode),
        Err(e) => Err(ApplicationError::validation(e)),
    };
    render_posture_result(&state, game)
}

/// Auto-save a game's perspective and tense (plain posture override).
pub async fn update_game_posture_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Form(form): Form<GamePostureForm>,
) -> Response {
    let perspective = NarrativePerspective::from_str(&form.narrative_perspective);
    let tense = NarrativeTense::from_str(&form.narrative_tense);
    let result = match (perspective, tense) {
        (Ok(perspective), Ok(tense)) => state.game_catalogue.set_posture(id, perspective, tense),
        (Err(e), _) | (_, Err(e)) => Err(ApplicationError::validation(e)),
    };
    render_posture_result(&state, result)
}

/// Auto-save the game's per-game preset selection (the picker).
pub async fn update_game_presets_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Form(form): Form<GamePresetsForm>,
) -> Response {
    let result = state.game_catalogue.set_preset_selection(
        id,
        &form.system_preset_id,
        &form.quantifier_preset_id,
        &form.impersonate_preset_id,
    );
    render_posture_result(&state, result)
}

/// Re-render the override fragment after a posture action. Validation
/// failures surface as an error span in place of the fragment (house style);
/// storage failures stay 500s so htmx leaves the page alone.
fn render_posture_result(state: &AppState, result: Result<Game, ApplicationError>) -> Response {
    match result {
        Ok(game) => ok(posture_controls_html(state, &game)),
        Err(e) if e.is_user_displayable() => ok(render_error(&e.to_string())),
        Err(e) => internal_error(render_error(&e.to_string())),
    }
}

/// Renders the posture-override fragment for one game. A preset-library
/// load failure degrades to an error span in the picker row; the posture
/// controls still render so the auto-saves keep working.
fn posture_controls_html(state: &AppState, game: &Game) -> String {
    let presets = (
        state.prompt_preset_service.list_presets(PresetType::System),
        state
            .prompt_preset_service
            .list_presets(PresetType::Quantifier),
        state
            .prompt_preset_service
            .list_presets(PresetType::Impersonate),
    );
    let (system, quantifier, impersonate, preset_load_error) = match presets {
        (Ok(system), Ok(quantifier), Ok(impersonate)) => (system, quantifier, impersonate, None),
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            (Vec::new(), Vec::new(), Vec::new(), Some(e.to_string()))
        }
    };

    let mut template = GamePostureTemplate::from_game(game, &system, &quantifier, &impersonate);
    template.preset_load_error = preset_load_error;
    template.render().unwrap_or_default()
}
