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
    app_err_to_response, ctx_or_error, internal_error, ok, ok_refresh,
};
use crate::server::games_fragment::template::{GameRowView, GamesPanelTemplate};

fn game_to_view(g: Game) -> GameRowView {
    GameRowView {
        id: g.id,
        name: g.name.clone(),
        world_name: g.world_name.clone(),
    }
}

pub async fn list_games_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let Ok(ctx) = ctx_or_error(&state) else {
        return match ctx_or_error(&state) {
            Ok(_) => unreachable!(),
            Err(e) => *e,
        };
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

    let template = GamesPanelTemplate {
        active_game,
        saved_games,
        worlds,
    };

    ok(template.render().unwrap_or_default())
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateGameForm {
    pub world_key: String,
}

pub async fn create_game_handler(
    State(state): State<AppState>,
    Form(form): Form<CreateGameForm>,
) -> Response<axum::body::Body> {
    match state.context_for_world(&form.world_key) {
        Ok(ctx) => match state.application_service.create_game(ctx) {
            Ok(_) => ok_refresh(),
            Err(e) => app_err_to_response(e),
        },
        Err(e) => internal_error(format!(
            "Failed to build context for world '{}': {e}",
            form.world_key
        )),
    }
}

pub async fn switch_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Response<axum::body::Body> {
    let Ok(ctx) = ctx_or_error(&state) else {
        return match ctx_or_error(&state) {
            Ok(_) => unreachable!(),
            Err(e) => *e,
        };
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
    let Ok(ctx) = ctx_or_error(&state) else {
        return match ctx_or_error(&state) {
            Ok(_) => unreachable!(),
            Err(e) => *e,
        };
    };

    match state.application_service.delete_game(ctx, id) {
        Ok(()) => ok(""),
        Err(e) => app_err_to_response(e),
    }
}
