//! [DOC: docs/system/dashboard.md]
//! Games fragment handlers

use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::Response,
};

use crate::model::game::Game;
use crate::server::AppState;

use crate::server::fragments::renderers::{
    app_err_to_response, bad_request, ctx_or_error, internal_error, ok, ok_refresh,
};
use crate::server::games_fragment::template::{GameRowView, GamesPanelTemplate, PersonaRowView};

fn game_to_view(g: Game) -> GameRowView {
    GameRowView {
        id: g.id,
        name: g.name.clone(),
        world_name: g.world_name.clone(),
        persona_name: g.persona_name.clone(),
    }
}

pub async fn list_games_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let ctx = match ctx_or_error(&state) {
        Ok(ctx) => ctx,
        Err(e) => return *e,
    };

    let Ok(games) = state.application_service.list_games(ctx.clone()) else {
        return internal_error("Failed to list games");
    };

    let active_id = state.application_service.current_game_id(ctx.clone());
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

    let Ok(worlds) = state.application_service.list_worlds(ctx.clone()) else {
        return internal_error("Failed to list worlds");
    };

    let personas: Vec<PersonaRowView> = match ctx.storage.list_personas() {
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
) -> Response<axum::body::Body> {
    match state.context_for_world(&form.world_key, &form.persona_key) {
        Ok(ctx) => match state.application_service.create_game(ctx) {
            Ok(_) => ok_refresh(),
            Err(e) => app_err_to_response(e),
        },
        Err(e) => bad_request(format!(
            "Failed to build context for world '{}' / persona '{}': {e}",
            form.world_key, form.persona_key
        )),
    }
}

pub async fn switch_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Response<axum::body::Body> {
    let ctx = match ctx_or_error(&state) {
        Ok(ctx) => ctx,
        Err(e) => return *e,
    };

    match state.application_service.switch_game(ctx, id) {
        Ok(()) => ok_refresh(),
        Err(e) => app_err_to_response(e),
    }
}

pub async fn delete_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Response<axum::body::Body> {
    let ctx = match ctx_or_error(&state) {
        Ok(ctx) => ctx,
        Err(e) => return *e,
    };

    match state.application_service.delete_game(ctx, id) {
        Ok(()) => ok(""),
        Err(e) => app_err_to_response(e),
    }
}
