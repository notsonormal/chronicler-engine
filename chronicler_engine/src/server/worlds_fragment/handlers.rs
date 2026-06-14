//! [DOC: docs/system/worlds.md]
//! Worlds management handlers

use axum::{extract::Path, extract::State, http::StatusCode, response::Response, Form};
use serde::Deserialize;

use crate::model::map::MapDef;
use crate::model::scenario::StartingScenario;
use crate::model::world::WorldCard;
use crate::server::AppState;

use super::fragments::{render_world_edit_form, render_worlds_panel};
use crate::server::fragments::{bad_request, internal_error, ok, ok_refresh};

/// Form data for creating or updating a world.
#[derive(Debug, Deserialize)]
pub struct WorldForm {
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: String, // One rule per line
    pub starting_room_id: Option<String>,
    pub player_key: String,
    pub default_room_image: Option<String>,
    pub map_json: String,
    pub scenarios_json: String,
}

impl WorldForm {
    /// Convert form data into a WorldCard, parsing and validating JSON fields.
    /// Returns an error message string if validation fails.
    fn into_world_card(self) -> Result<(WorldCard, MapDef), String> {
        // Parse and validate map JSON
        let map: MapDef =
            serde_json::from_str(&self.map_json).map_err(|e| format!("Invalid map JSON: {e}"))?;

        // Parse and validate scenarios JSON
        let scenarios: Vec<StartingScenario> = serde_json::from_str(&self.scenarios_json)
            .map_err(|e| format!("Invalid scenarios JSON: {e}"))?;

        // Parse global rules (one per line)
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
            starting_room_id: self.starting_room_id.unwrap_or_else(|| "start".to_string()),
            scenarios,
            default_scenario_id: None,
            default_room_image: self.default_room_image.filter(|s| !s.is_empty()),
            player_key: self.player_key,
        };

        Ok((world_card, map))
    }
}

/// List all worlds in a fragment for HTMX swap.
pub async fn list_worlds_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let ctx = match state.as_game_service_context() {
        Ok(c) => c,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };

    let worlds = match state.application_service.list_worlds(ctx.clone()) {
        Ok(w) => w,
        Err(e) => return internal_error(format!("Failed to load worlds: {e}")),
    };

    // Count games per world
    let games = state
        .application_service
        .list_games(ctx)
        .unwrap_or_default();

    let mut games_per_world: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for game in &games {
        *games_per_world.entry(game.world_key.clone()).or_insert(0) += 1;
    }

    let html = render_worlds_panel(&worlds, &games_per_world);
    ok(html)
}

/// Create a new world.
pub async fn create_world_handler(
    State(state): State<AppState>,
    Form(form): Form<WorldForm>,
) -> Response<axum::body::Body> {
    let ctx = match state.as_game_service_context() {
        Ok(c) => c,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };

    let (world_card, map) = match form.into_world_card() {
        Ok(w) => w,
        Err(e) => return bad_request(e),
    };

    match state
        .application_service
        .create_world(ctx.clone(), world_card, map)
    {
        Ok(_) => ok_refresh(),
        Err(e) => bad_request(format!("Failed to create world: {e}")),
    }
}

/// Edit world - show the edit form.
pub async fn new_world_form_handler(State(state): State<AppState>) -> Response<axum::body::Body> {
    let ctx = match state.as_game_service_context() {
        Ok(c) => c,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };

    // Load personas for the dropdown
    let personas = match state.application_service.list_personas(ctx) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Failed to load personas: {e}");
            Vec::new()
        }
    };

    let html = render_world_edit_form(None, None, &[], &personas);
    ok(html)
}

/// Edit world - show the edit form.
pub async fn edit_world_form_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Response<axum::body::Body> {
    let ctx = match state.as_game_service_context() {
        Ok(c) => c,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };

    // Load the world WITH its map data
    let world_with_map = match state.application_service.get_world(ctx.clone(), &key) {
        Ok(Some(w)) => w,
        Ok(None) => return bad_request(format!("World '{key}' not found")),
        Err(e) => return internal_error(format!("Failed to load world: {e}")),
    };

    // Load personas for the dropdown
    let personas = match state.application_service.list_personas(ctx) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Failed to load personas: {e}");
            Vec::new()
        }
    };

    let html = render_world_edit_form(
        Some(&world_with_map.world_card),
        Some(&world_with_map.map),
        &world_with_map.world_card.scenarios,
        &personas,
    );
    ok(html)
}

/// Update an existing world.
pub async fn update_world_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Form(form): Form<WorldForm>,
) -> Response<axum::body::Body> {
    let ctx = match state.as_game_service_context() {
        Ok(c) => c,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };

    // Get the world by key to retrieve its ID
    let world_with_map = match state.application_service.get_world(ctx.clone(), &key) {
        Ok(Some(w)) => w,
        Ok(None) => return bad_request(format!("World '{key}' not found")),
        Err(e) => return internal_error(format!("Failed to load world: {e}")),
    };

    let (mut world_card, map) = match form.into_world_card() {
        Ok(w) => w,
        Err(e) => return bad_request(e),
    };

    // Use path key as canonical source, ignore form.key
    world_card.key = key;

    // Update using the world_id
    match state
        .application_service
        .update_world(ctx, world_with_map.world_id, world_card, map)
    {
        Ok(()) => ok_refresh(),
        Err(e) => internal_error(format!("Failed to update world: {e}")),
    }
}

/// Delete a world.
pub async fn delete_world_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> (StatusCode, String) {
    let ctx = match state.as_game_service_context() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load context: {e}"),
            );
        }
    };

    // Call application_service.delete_world directly - storage layer handles the foreign key check
    match state.application_service.delete_world(ctx, &key) {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => {
            // Map EngineError::ForeignKeyViolation to BAD_REQUEST
            let err_str = e.to_string();
            if err_str.contains("referenced") || err_str.contains("foreign key") {
                (
                    StatusCode::BAD_REQUEST,
                    format!(
                        "Cannot delete world '{key}' - games reference it. Delete those games first."
                    ),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to delete world: {e}"),
                )
            }
        }
    }
}
