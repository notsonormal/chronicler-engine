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
use crate::domain::model::game::{Game, PresetSelection};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use crate::error::EngineError;

use crate::adapters::driving::http::games::templates::games::{
    GamePostureTemplate, GamesPanelTemplate, PersonaRowView,
};
use crate::adapters::driving::http::utils::error::render_error;
use crate::adapters::driving::http::utils::response::{internal_error, ok, ok_refresh};
use crate::adapters::driving::http::utils::view_mappers::game_to_view;
use crate::adapters::driving::http::view_models::SafeHtml;

pub async fn list_games_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let Ok((active_game, saved_games)) = current_and_saved_games(&state) else {
        return internal_error("Failed to list games");
    };

    let active_game = active_game.map(game_to_view);
    let saved_games: Vec<_> = saved_games.into_iter().map(game_to_view).collect();

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
        saved_games_empty: SafeHtml::new(GamesPanelTemplate::saved_games_empty_html()),
    };

    ok(template.render().unwrap_or_default())
}

fn current_and_saved_games(
    state: &AppState,
) -> Result<(Option<Game>, Vec<Game>), ApplicationError> {
    let games = state.game_catalogue.list_games()?;
    let active_id = state.game_catalogue.current_game_id();
    let mut active_game = None;
    let saved_games = games
        .into_iter()
        .filter_map(|game| {
            if game.id == active_id {
                active_game = Some(game);
                None
            } else {
                Some(game)
            }
        })
        .collect();
    Ok((active_game, saved_games))
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
    // The newly current game may carry a stale persisted `Generating` from a panicked turn.
    state.pipeline.heal_stale_status(&state.generation_gate)?;
    Ok(ok_refresh())
}

#[derive(Debug, serde::Deserialize)]
pub struct RenameGameForm {
    pub display_name: String,
}

pub async fn rename_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Form(form): Form<RenameGameForm>,
) -> Result<Response, ApplicationError> {
    state.game_catalogue.rename_game(id, &form.display_name)?;
    Ok(ok_refresh())
}

pub async fn delete_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Response, ApplicationError> {
    state.game_catalogue.delete_game(id)?;
    Ok(ok(remaining_saved_games_empty_state(&state)))
}

/// The list's own empty state only renders with the whole panel, so a delete that
/// empties the list answers with this instead.
fn remaining_saved_games_empty_state(state: &AppState) -> String {
    let Ok((_, saved_games)) = current_and_saved_games(state) else {
        return String::new();
    };
    if saved_games.is_empty() {
        GamesPanelTemplate::saved_games_empty_html()
    } else {
        String::new()
    }
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
    pub options_preset_id: String,
}

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

pub async fn update_game_presets_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Form(form): Form<GamePresetsForm>,
) -> Response {
    let result = state.game_catalogue.set_preset_selection(
        id,
        PresetSelection::new(
            form.system_preset_id,
            form.quantifier_preset_id,
            form.impersonate_preset_id,
            form.options_preset_id,
        ),
    );
    render_posture_result(&state, result)
}

/// Validation failures answer with an error fragment at 200 (house style); storage
/// failures stay 500s so htmx leaves the page alone.
fn render_posture_result(state: &AppState, result: Result<Game, ApplicationError>) -> Response {
    match result {
        Ok(game) => ok(posture_controls_html(state, &game)),
        Err(e) if e.is_user_displayable() => ok(render_error(&e.to_string())),
        Err(e) => internal_error(render_error(&e.to_string())),
    }
}

/// A preset-library load failure degrades to an error fragment in the picker row; the
/// posture controls still render so the auto-saves keep working.
fn posture_controls_html(state: &AppState, game: &Game) -> String {
    let (libraries, preset_load_error) = match load_preset_libraries(state) {
        Ok(libraries) => (libraries, None),
        Err(e) => (Default::default(), Some(e.to_string())),
    };
    let [system, quantifier, impersonate, options] = libraries;

    let mut template =
        GamePostureTemplate::from_game(game, &system, &quantifier, &impersonate, &options);
    template.preset_load_error = preset_load_error;
    template.render().unwrap_or_default()
}

/// The preset libraries the posture picker lists, in
/// [`GamePostureTemplate::from_game`]'s field order.
const PRESET_PICKER_TYPES: [PresetType; 4] = [
    PresetType::System,
    PresetType::Quantifier,
    PresetType::Impersonate,
    PresetType::Options,
];

fn load_preset_libraries(
    state: &AppState,
) -> Result<[Vec<PromptPreset>; PRESET_PICKER_TYPES.len()], EngineError> {
    let mut libraries: [Vec<PromptPreset>; PRESET_PICKER_TYPES.len()] = Default::default();
    for (slot, preset_type) in libraries.iter_mut().zip(PRESET_PICKER_TYPES) {
        *slot = state.prompt_preset_service.list_presets(preset_type)?;
    }
    Ok(libraries)
}
