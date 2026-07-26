//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Worlds management handlers

use axum::{extract::Path, extract::State, response::Response, Form};
use askama::Template;
use serde::Deserialize;

use crate::domain::model::map::MapDef;
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::world::WorldCard;
use crate::adapters::driving::http::AppState;

use crate::adapters::driving::http::utils::error::render_error;
use crate::adapters::driving::http::utils::response::{bad_request, internal_error, ok};
use crate::adapters::driving::http::utils::view_mappers::games_per_world;
use crate::adapters::driving::http::worlds::templates::worlds::{
    WorldFormTemplate, WorldsPanelTemplate,
};

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

pub async fn list_worlds_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let app = &state.application_service;
    let worlds = match app.list_worlds() {
        Ok(w) => w,
        Err(e) => return internal_error(format!("Failed to load worlds: {e}")),
    };

    let games = app.list_games().unwrap_or_default();
    let games_per_world = games_per_world(&games);

    let html = WorldsPanelTemplate::from_worlds(&worlds, &games_per_world)
        .render()
        .unwrap_or_default();
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
            ok(
                WorldsPanelTemplate::from_worlds(&worlds.unwrap_or_default(), &games_per_world)
                    .render()
                    .unwrap_or_default(),
            )
        }
        Err(e) => bad_request(format!("Failed to create world: {e}")),
    }
}

pub async fn new_world_form_handler(State(_state): State<AppState>) -> Response<axum::body::Body> {
    ok(WorldFormTemplate::from_world_data(None, None, &[])
        .render()
        .unwrap_or_default())
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

    let html = WorldFormTemplate::from_world_data(
        Some(&world_with_map.world_card),
        Some(&world_with_map.map),
        &world_with_map.world_card.scenarios,
    )
    .render()
    .unwrap_or_default();
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
            ok(
                WorldsPanelTemplate::from_worlds(&worlds.unwrap_or_default(), &games_per_world)
                    .render()
                    .unwrap_or_default(),
            )
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
