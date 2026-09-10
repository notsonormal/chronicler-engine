//! [DOC: docs/diataxis/reference/game_flow.md]
//! Worlds management handlers

use std::str::FromStr;

use axum::{extract::Path, extract::State, response::Response, Form};
use askama::Template;
use serde::Deserialize;

use crate::domain::model::map::MapDef;
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
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
    pub narrator_mode: Option<String>,
    pub narrative_perspective: Option<String>,
    pub narrative_tense: Option<String>,
}

impl WorldForm {
    fn parse_map(&self) -> Result<MapDef, String> {
        serde_json::from_str(&self.map_json).map_err(|e| format!("Invalid map JSON: {e}"))
    }

    fn parse_scenarios(&self) -> Result<Vec<StartingScenario>, String> {
        serde_json::from_str(&self.scenarios_json)
            .map_err(|e| format!("Invalid scenarios JSON: {e}"))
    }

    fn global_rules_list(&self) -> Vec<String> {
        self.global_rules
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .collect()
    }

    fn into_world_card(self) -> Result<(WorldCard, MapDef), String> {
        let map = self.parse_map()?;

        let scenarios = self.parse_scenarios()?;

        let global_rules = self.global_rules_list();

        let world_card = WorldCard {
            key: self.key,
            name: self.name,
            description: self.description,
            global_rules,
            scenarios,
            default_scenario_id: None,
            default_room_image: self.default_room_image.filter(|s| !s.is_empty()),
            narrator_mode: self
                .narrator_mode
                .as_deref()
                .map(NarratorMode::parse_or_default)
                .unwrap_or_default(),
            narrative_perspective: self
                .narrative_perspective
                .as_deref()
                .map(NarrativePerspective::parse_or_default)
                .unwrap_or_default(),
            narrative_tense: self
                .narrative_tense
                .as_deref()
                .map(NarrativeTense::parse_or_default)
                .unwrap_or_default(),
            options_always_on: false,
        };

        Ok((world_card, map))
    }
}

pub async fn list_worlds_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let worlds = match state.world_catalogue.list_worlds() {
        Ok(w) => w,
        Err(e) => return internal_error(format!("Failed to load worlds: {e}")),
    };

    let games = state.game_catalogue.list_games().unwrap_or_default();
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
    let (world_card, map) = match form.into_world_card() {
        Ok(w) => w,
        Err(e) => return bad_request(e),
    };

    match state.world_catalogue.create_world(world_card, map) {
        Ok(_) => {
            let worlds = state.world_catalogue.list_worlds();
            let games = state.game_catalogue.list_games().unwrap_or_default();
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
    let (_, world_card, map) = match state.world_catalogue.get_world(&key) {
        Ok(Some(w)) => w,
        Ok(None) => return bad_request(format!("World '{key}' not found")),
        Err(e) => return internal_error(format!("Failed to load world: {e}")),
    };

    let html =
        WorldFormTemplate::from_world_data(Some(&world_card), Some(&map), &world_card.scenarios)
            .render()
            .unwrap_or_default();
    ok(html)
}

pub async fn update_world_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Form(form): Form<WorldForm>,
) -> Response<axum::body::Body> {
    let (world_id, mut world_card, _) = match state.world_catalogue.get_world(&key) {
        Ok(Some(w)) => w,
        Ok(None) => return bad_request(format!("World '{key}' not found")),
        Err(e) => return internal_error(format!("Failed to load world: {e}")),
    };

    let map = match form.parse_map() {
        Ok(m) => m,
        Err(e) => return bad_request(e),
    };
    let scenarios = match form.parse_scenarios() {
        Ok(s) => s,
        Err(e) => return bad_request(e),
    };


    world_card.default_scenario_id = world_card
        .default_scenario_id
        .filter(|id| scenarios.iter().any(|s| &s.id == id));
    world_card.scenarios = scenarios;
    world_card.global_rules = form.global_rules_list();
    world_card.name = form.name;
    world_card.description = form.description;
    world_card.default_room_image = form.default_room_image.filter(|s| !s.is_empty());


    if let Some(mode) = form.narrator_mode {
        world_card.narrator_mode = NarratorMode::parse_or_default(&mode);
    }
    if let Some(perspective) = form.narrative_perspective {
        world_card.narrative_perspective = NarrativePerspective::parse_or_default(&perspective);
    }
    if let Some(tense) = form.narrative_tense {
        world_card.narrative_tense = NarrativeTense::parse_or_default(&tense);
    }

    match state
        .world_catalogue
        .update_world(world_id, world_card, map)
    {
        Ok(()) => {
            let worlds = state.world_catalogue.list_worlds();
            let games = state.game_catalogue.list_games().unwrap_or_default();
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
    match state.world_catalogue.delete_world(&key) {
        Ok(()) => ok(""),
        Err(e) if e.is_user_displayable() => {
            let error_html = render_error(&e.to_string());
            ok(format!(r#"<li class="world-item">{error_html}</li>"#))
        }
        Err(e) => internal_error(render_error(&e.to_string())),
    }
}

#[derive(Debug, Deserialize)]
pub struct WorldPostureForm {
    pub narrator_mode: Option<String>,
    pub narrative_perspective: Option<String>,
    pub narrative_tense: Option<String>,
}

/// Auto-save the world's narrative posture. Patches only the posture
/// fields; invalid values render an error span into `#world-posture-status`
/// and mutate nothing.
pub async fn update_world_posture_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Form(form): Form<WorldPostureForm>,
) -> Response<axum::body::Body> {
    let parsed = (
        form.narrator_mode
            .as_deref()
            .map(NarratorMode::from_str)
            .transpose(),
        form.narrative_perspective
            .as_deref()
            .map(NarrativePerspective::from_str)
            .transpose(),
        form.narrative_tense
            .as_deref()
            .map(NarrativeTense::from_str)
            .transpose(),
    );
    let (mode, perspective, tense) = match parsed {
        (Ok(mode), Ok(perspective), Ok(tense)) => (mode, perspective, tense),
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            return ok(render_error(&e));
        }
    };

    let (world_id, mut world_card, map) = match state.world_catalogue.get_world(&key) {
        Ok(Some(w)) => w,
        Ok(None) => return bad_request(format!("World '{key}' not found")),
        Err(e) => return internal_error(format!("Failed to load world: {e}")),
    };

    if let Some(mode) = mode {
        world_card.narrator_mode = mode;
    }
    if let Some(perspective) = perspective {
        world_card.narrative_perspective = perspective;
    }
    if let Some(tense) = tense {
        world_card.narrative_tense = tense;
    }

    match state
        .world_catalogue
        .update_world(world_id, world_card, map)
    {
        Ok(()) => ok(r#"<span class="posture-status">Saved</span>"#),
        Err(e) => internal_error(format!("Failed to update world posture: {e}")),
    }
}
