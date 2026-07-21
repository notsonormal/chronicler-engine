//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Worlds management handlers

use axum::{extract::Path, extract::State, response::Response, Form};
use serde::Deserialize;

use crate::domain::model::map::MapDef;
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::world::WorldCard;
use crate::adapters::driving::http::AppState;

use super::fragments::{render_world_edit_form, render_worlds_panel};
use crate::adapters::driving::http::fragments::{bad_request, internal_error, ok, render_error};

#[derive(Debug, Deserialize)]
pub struct WorldForm {
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: String,
    pub default_room_image: Option<String>,
    pub map_json: String,
    pub scenarios_json: String,
}

impl WorldForm {
    fn into_world_card(self) -> Result<(WorldCard, MapDef), String> {
        let map: MapDef =
            serde_json::from_str(&self.map_json).map_err(|e| format!("Invalid map JSON: {e}"))?;

        let scenarios: Vec<StartingScenario> = serde_json::from_str(&self.scenarios_json)
            .map_err(|e| format!("Invalid scenarios JSON: {e}"))?;

        let global_rules: Vec<String> = self
            .global_rules
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .collect();

        let world_card = WorldCard {
            key: self.key,
            name: self.name,
            description: self.description,
            global_rules,
            scenarios,
            default_scenario_id: None,
            default_room_image: self.default_room_image.filter(|s| !s.is_empty()),
        };

        Ok((world_card, map))
    }
}

fn games_per_world(
    games: &[crate::domain::model::game::Game],
) -> std::collections::HashMap<String, usize> {
    let mut map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for game in games {
        *map.entry(game.world_key.clone()).or_insert(0) += 1;
    }
    map
}

pub async fn list_worlds_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let app = &state.application_service;
    let worlds = match app.list_worlds() {
        Ok(w) => w,
        Err(e) => return internal_error(format!("Failed to load worlds: {e}")),
    };

    let games = app.list_games().unwrap_or_default();
    let games_per_world = games_per_world(&games);

    let html = render_worlds_panel(&worlds, &games_per_world);
    ok(html)
}

pub async fn create_world_handler(
    State(state): State<AppState>,
    Form(form): Form<WorldForm>,
) -> Response<axum::body::Body> {
    let app = &state.application_service;
    let (world_card, map) = match form.into_world_card() {
        Ok(w) => w,
        Err(e) => return bad_request(e),
    };

    match app.create_world(world_card, map) {
        Ok(_) => {
            let worlds = app.list_worlds();
            let games = app.list_games().unwrap_or_default();
            let games_per_world = games_per_world(&games);
            ok(render_worlds_panel(
                &worlds.unwrap_or_default(),
                &games_per_world,
            ))
        }
        Err(e) => bad_request(format!("Failed to create world: {e}")),
    }
}

pub async fn new_world_form_handler(State(_state): State<AppState>) -> Response<axum::body::Body> {
    ok(render_world_edit_form(None, None, &[]))
}

pub async fn edit_world_form_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Response<axum::body::Body> {
    let world_with_map = match state.application_service.get_world(&key) {
        Ok(Some(w)) => w,
        Ok(None) => return bad_request(format!("World '{key}' not found")),
        Err(e) => return internal_error(format!("Failed to load world: {e}")),
    };

    let html = render_world_edit_form(
        Some(&world_with_map.world_card),
        Some(&world_with_map.map),
        &world_with_map.world_card.scenarios,
    );
    ok(html)
}

pub async fn update_world_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Form(form): Form<WorldForm>,
) -> Response<axum::body::Body> {
    let app = &state.application_service;
    let world_with_map = match app.get_world(&key) {
        Ok(Some(w)) => w,
        Ok(None) => return bad_request(format!("World '{key}' not found")),
        Err(e) => return internal_error(format!("Failed to load world: {e}")),
    };

    let (mut world_card, map) = match form.into_world_card() {
        Ok(w) => w,
        Err(e) => return bad_request(e),
    };

    world_card.key = key;

    match app.update_world(world_with_map.world_id, world_card, map) {
        Ok(()) => {
            let worlds = app.list_worlds();
            let games = app.list_games().unwrap_or_default();
            let games_per_world = games_per_world(&games);
            ok(render_worlds_panel(
                &worlds.unwrap_or_default(),
                &games_per_world,
            ))
        }
        Err(e) => internal_error(format!("Failed to update world: {e}")),
    }
}

pub async fn delete_world_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Response<axum::body::Body> {
    match state.application_service.delete_world(&key) {
        Ok(()) => ok(""),
        Err(e) if e.is_user_displayable() => {
            let error_html = render_error(&e.to_string());
            ok(format!(r#"<li class="world-item">{error_html}</li>"#))
        }
        Err(e) => internal_error(render_error(&e.to_string())),
    }
}
