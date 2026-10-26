//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Games fragment handlers

use std::str::FromStr;

use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Response},
};

use crate::adapters::driving::http::AppState;
use crate::application::errors::ApplicationError;
use crate::domain::model::game::{Game, GameId, PresetSelection};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use crate::error::EngineError;

use crate::adapters::driving::http::games::templates::games::{
    GamePostureTemplate, GameRenameFormTemplate, GamesPanelTemplate, NewGameFormTemplate,
    PersonaRowView,
};
use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::fragment::{
    failed_fragment_response, render_failure_response, render_response,
};
use crate::adapters::driving::http::utils::error::{
    action_error_response, action_failure_response, action_refusal_response,
};
use crate::adapters::driving::http::utils::response::ok;
use crate::adapters::driving::http::utils::view_mappers::game_to_view;

pub async fn list_games_fragment(
    State(state): State<AppState>,
    current: Option<CurrentGame>,
) -> Response<axum::body::Body> {
    let (active_game, saved_games) = match current_and_saved_games(&state, current) {
        Ok(games) => games,
        Err(e) => return failed_fragment_response(&format!("Failed to list games: {e}")),
    };

    let posture_html = match &active_game {
        Some(game) => match posture_controls_html(&state, game) {
            Ok(html) => html,
            Err(e) => return render_failure_response(&e),
        },
        None => String::new(),
    };

    let view_of = |game: Game| {
        let run_held = state.game_view_query.autoplay_holds(GameId(game.id));
        game_to_view(game, run_held)
    };
    let active_game = active_game.map(view_of);
    let saved_games: Vec<_> = saved_games.into_iter().map(view_of).collect();

    let worlds = match state.world_catalogue.list_worlds() {
        Ok(worlds) => worlds,
        Err(e) => return failed_fragment_response(&format!("Failed to list worlds: {e}")),
    };

    let personas = persona_rows(&state);

    let template = GamesPanelTemplate {
        active_game,
        saved_games,
        worlds,
        personas,
        posture_html,
    };

    render_response(&template)
}

fn persona_rows(state: &AppState) -> Vec<PersonaRowView> {
    match state.persona_catalogue.list_personas() {
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
    .collect()
}

/// The Rename form opens in the dialog. A Game that is gone, or a list that
/// cannot be read, opens no dialog: the Rename control reports it in its own slot.
pub async fn rename_game_form_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Response<axum::body::Body> {
    let games = match state.game_catalogue.list_games() {
        Ok(games) => games,
        Err(e) => return action_failure_response(format!("Failed to list games: {e}")),
    };
    match games.into_iter().find(|game| game.id == id) {
        Some(game) => render_response(&GameRenameFormTemplate {
            id: game.id,
            display_name: game.display_name,
        }),
        None => action_refusal_response("Game not found"),
    }
}

pub async fn new_game_form_handler(State(state): State<AppState>) -> Response<axum::body::Body> {
    let worlds = match state.world_catalogue.list_worlds() {
        Ok(worlds) => worlds,
        Err(e) => return action_failure_response(format!("Failed to list worlds: {e}")),
    };
    render_response(&NewGameFormTemplate {
        worlds,
        personas: persona_rows(&state),
    })
}

fn current_and_saved_games(
    state: &AppState,
    current: Option<CurrentGame>,
) -> Result<(Option<Game>, Vec<Game>), ApplicationError> {
    let games = state.game_catalogue.list_games()?;
    let active_id = current.map(|CurrentGame(game_id)| game_id);
    let mut active_game = None;
    let saved_games = games
        .into_iter()
        .filter_map(|game| {
            if Some(GameId(game.id)) == active_id {
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
    Ok(ok(""))
}

pub async fn switch_game_handler(State(state): State<AppState>, Path(id): Path<u64>) -> Response {
    match state.game_catalogue.switch_game(id) {
        Ok(()) => ok(""),
        Err(e) => action_error_response(e, "Failed to switch game"),
    }
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
    Ok(ok(""))
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
    config_save_result(game)
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
    config_save_result(result)
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
    config_save_result(result)
}

/// A saved config answers nothing: the Games panel loads the new value. A refusal
/// or a failure answers a non-2xx error fragment, which the control reports in its
/// own slot.
fn config_save_result(result: Result<Game, ApplicationError>) -> Response {
    match result {
        Ok(_) => ok(""),
        Err(ApplicationError::Refused(refusal)) => refusal.into_response(),
        Err(e) => action_error_response(e, "Failed to save the setting"),
    }
}

/// The posture controls still render on a preset-load failure, because the
/// auto-saves depend on them.
fn posture_controls_html(state: &AppState, game: &Game) -> askama::Result<String> {
    let (libraries, preset_load_error) = match load_preset_libraries(state) {
        Ok(libraries) => (libraries, None),
        Err(e) => (Default::default(), Some(e.to_string())),
    };
    let [system, quantifier, impersonate, options] = libraries;

    let view_only = state.game_view_query.autoplay_holds(GameId(game.id));
    let mut template = GamePostureTemplate::from_game(
        game,
        &system,
        &quantifier,
        &impersonate,
        &options,
        view_only,
    );
    template.preset_load_error = preset_load_error;
    template.render()
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
