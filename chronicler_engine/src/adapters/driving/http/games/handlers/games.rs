//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Games fragment handlers

use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::Response,
};

use crate::adapters::driving::http::AppState;
use crate::application::application_service::ApplicationError;

use crate::adapters::driving::http::utils::response::{internal_error, ok, ok_refresh};
use crate::adapters::driving::http::utils::view_mappers::game_to_view;
use crate::adapters::driving::http::games::templates::games::{GamesPanelTemplate, PersonaRowView};

pub async fn list_games_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let app = &state.application_service;
    let Ok(games) = app.list_games() else {
        return internal_error("Failed to list games");
    };

    let active_id = app.current_game_id();
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

    let Ok(worlds) = app.list_worlds() else {
        return internal_error("Failed to list worlds");
    };

    let personas: Vec<PersonaRowView> = match app.list_personas() {
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
        .application_service
        .create_game(&form.world_key, &form.persona_key)?;
    Ok(ok_refresh())
}

pub async fn switch_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Response, ApplicationError> {
    state.application_service.switch_game(id)?;
    Ok(ok_refresh())
}

pub async fn delete_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Response, ApplicationError> {
    state.application_service.delete_game(id)?;
    Ok(ok(""))
}
