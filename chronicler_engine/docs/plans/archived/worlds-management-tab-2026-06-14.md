# Worlds Management Tab UI

## Context

After the data foundation (Plan 1) enables multi-world support, users need a UI to create, edit, and manage worlds. This plan adds a dedicated "Worlds" tab to the dashboard with full CRUD operations for worlds. Requires Plan 1 completion first.

## Approach

### Step 1: Create `worlds_fragment` module

**New directory**: `src/server/worlds_fragment/`

**New file**: `src/server/worlds_fragment/mod.rs`
```rust
mod fragments;
mod handlers;

pub use fragments::*;
pub use handlers::*;
```

**New file**: `src/server/worlds_fragment/handlers.rs`
```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    Form,
};
use serde::Deserialize;

use crate::server::AppState;
use crate::model::world::WorldCard;
use crate::model::map::MapDef;

use super::fragments::{render_worlds_panel, render_error};

#[derive(Deserialize)]
pub struct WorldForm {
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: String,  // One rule per line
    pub starting_room_id: Option<String>,
    pub player_key: String,
    pub default_room_image: Option<String>,
    pub map_json: String,
    pub scenarios_json: String,
}

pub async fn list_worlds_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let worlds = match state.application_service.list_worlds(state.as_game_service_context_or_default()) {
        Ok(w) => w,
        Err(e) => return render_error(&format!("Failed to load worlds: {e}")),
    };
    
    // Count games per world
    let games = state.application_service.list_games(state.as_game_service_context_or_default()).unwrap_or_default();
    
    let mut html = String::new();
    html.push_str("<div class=\"worlds-panel\">");
    html.push_str("<button class=\"btn-new-world\" onclick=\"document.getElementById('world-modal').style.display='block'\">Create New World</button>");
    
    if worlds.is_empty() {
        html.push_str("<p>No worlds defined. Create your first world to get started.</p>");
    } else {
        html.push_str("<ul class=\"worlds-list\">");
        for world in worlds {
            let game_count = games.iter().filter(|g| g.world_key == world.key).count();
            html.push_str(&format!(
                "<li class=\"world-item\"><strong>{}</strong> - {} <em>({} games)</em> \
                 <button hx-post=\"/worlds/{}/edit\">Edit</button> \
                 <button hx-post=\"/worlds/{}/delete\" hx-confirm=\"Delete this world? This cannot be undone.\">Delete</button></li>",
                world.name, world.description, game_count, world.key, world.key
            ));
        }
        html.push_str("</ul>");
    }
    
    html.push_str("</div>");
    crate::server::fragments::renderers::ok(html)
}

pub async fn create_world_handler(
    State(state): State<AppState>,
    Form(form): Form<WorldForm>,
) -> Response<axum::body::Body> {
    // Parse and validate
    let map: MapDef = match serde_json::from_str(&form.map_json) {
        Ok(m) => m,
        Err(e) => return render_error(&format!("Invalid map JSON: {e}")),
    };
    
    let scenarios: Vec<crate::model::scenario::StartingScenario> = match serde_json::from_str(&form.scenarios_json) {
        Ok(s) => s,
        Err(e) => return render_error(&format!("Invalid scenarios JSON: {e}")),
    };
    
    let global_rules: Vec<String> = form.global_rules.lines().filter(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()).collect();
    
    let world_card = WorldCard {
        key: form.key,
        name: form.name,
        description: form.description,
        global_rules,
        starting_room_id: form.starting_room_id.unwrap_or_else(|| "start".to_string()),
        scenarios,
        default_scenario_id: None,
        default_room_image: form.default_room_image.filter(|s| !s.is_empty()),
        player_key: form.player_key,
    };
    
    match state.application_service.create_world(world_card, map) {
        Ok(_) => crate::server::fragments::renderers::ok_refresh(),
        Err(e) => render_error(&format!("Failed to create world: {e}")),
    }
}

pub async fn edit_world_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Response<axum::body::Body> {
    // Load world and show edit form
    // Implementation similar to create but pre-populates form
    todo!("Implement edit form rendering")
}

pub async fn update_world_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Form(form): Form<WorldForm>,
) -> Response<axum::body::Body> {
    // Similar to create but calls update_world
    todo!("Implement update")
}

pub async fn delete_world_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> (StatusCode, String) {
    // Check for games referencing this world
    let games = state.application_service.list_games(state.as_game_service_context_or_default()).unwrap_or_default();
    let game_count = games.iter().filter(|g| g.world_key == key).count();
    
    if game_count > 0 {
        return (StatusCode::BAD_REQUEST, format!("Cannot delete world '{key}' - {} games reference it. Delete those games first.", game_count));
    }
    
    match state.application_service.delete_world(&key) {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete world: {e}")),
    }
}
```

**New file**: `src/server/worlds_fragment/fragments.rs`
```rust
use crate::model::world::WorldCard;

pub fn render_worlds_panel(worlds: &[WorldCard], games_per_world: std::collections::HashMap<&str, usize>) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"worlds-panel\">");
    html.push_str("<button class=\"btn-new-world\">Create New World</button>");
    
    if worlds.is_empty() {
        html.push_str("<p>No worlds defined.</p>");
    } else {
        html.push_str("<ul class=\"worlds-list\">");
        for world in worlds {
            let count = games_per_world.get(world.key.as_str()).copied().unwrap_or(0);
            html.push_str(&format!(
                "<li><strong>{}</strong> - {} <em>({} games)</em></li>",
                world.name, world.description, count
            ));
        }
        html.push_str("</ul>");
    }
    
    html.push_str("</div>");
    html
}

pub fn render_world_edit_form(world: Option<&WorldCard>) -> String {
    let is_edit = world.is_some();
    let w = world.unwrap_or(&WorldCard::default());
    
    format!(
        r#"<form hx-post="/worlds{}" class="world-form">
            <label>Key (immutable): <input type="text" name="key" value="{}" {} /></label>
            <label>Name: <input type="text" name="name" value="{}" required /></label>
            <label>Description: <textarea name="description">{}</textarea></label>
            <label>Global Rules (one per line): <textarea name="global_rules">{}</textarea></label>
            <label>Starting Room ID: <input type="text" name="starting_room_id" value="{}" /></label>
            <label>Player Persona: <select name="player_key">...</select></label>
            <label>Default Room Image: <input type="text" name="default_room_image" value="{}" /></label>
            <label>Map JSON: <textarea name="map_json" class="json-editor">{}</textarea></label>
            <label>Scenarios JSON: <textarea name="scenarios_json" class="json-editor">{}</textarea></label>
            <button type="submit">Save World</button>
        </form>"#,
        if is_edit { format!("/{}", w.key) } else { String::new() },
        w.key, if is_edit { "readonly" } else { "" },
        w.name, w.description,
        w.global_rules.join("\n"),
        w.starting_room_id,
        w.default_room_image.clone().unwrap_or_default(),
        "{}", "{}"  // map and scenarios JSON would be serialized here
    )
}

pub fn render_error(msg: &str) -> axum::response::Response<axum::body::Body> {
    use axum::{http::StatusCode, response::Response};
    (StatusCode::BAD_REQUEST, format!("<div class=\"error\">{msg}</div>")).into_response()
}
```

### Step 2: Add routes to router

**Edit**: `src/server/router.rs`

Add module declaration at top:
```rust
mod worlds_fragment;
```

Add routes in the router chain:
```rust
.route("/fragment/worlds", get(worlds_fragment::list_worlds_fragment))
.route("/worlds", post(worlds_fragment::create_world_handler))
.route("/worlds/:key", post(worlds_fragment::update_world_handler))
.route("/worlds/:key/edit", get(worlds_fragment::edit_world_handler))
.route("/worlds/:key/delete", post(worlds_fragment::delete_world_handler))
```

### Step 3: Add Worlds tab to index.html

**Edit**: `assets/index.html`

Add tab button after Settings (line 23):
```html
<button class="tab" data-tab="worlds">Worlds</button>
```

Add tab content panel after settings-tab (around line 90):
```html
<div class="tab-content" id="worlds-tab">
  <div class="worlds-panel" hx-get="/fragment/worlds" hx-trigger="load"></div>
</div>
```

### Step 4: Application service methods

**Edit**: `src/application/application_service.rs`

Add delegation methods:
```rust
pub fn list_worlds(&self, ctx: GameServiceContext) -> Result<Vec<WorldCard>, ApplicationError> {
    ctx.storage.list_worlds().map_err(Into::into)
}

pub fn create_world(&self, world_card: WorldCard, map: MapDef) -> Result<i64, ApplicationError> {
    self.storage.create_world(&world_card, &map).map_err(Into::into)
}

pub fn update_world(&self, id: i64, world_card: WorldCard, map: MapDef) -> Result<(), ApplicationError> {
    self.storage.update_world(id, &world_card, &map).map_err(Into::into)
}

pub fn delete_world(&self, key: &str) -> Result<(), ApplicationError> {
    self.storage.delete_world(key).map_err(Into::into)
}
```

**Edit**: `src/storage/backend/worlds.rs` — Add `Storage::delete_world(key: &str)`:
```rust
pub fn delete_world(&self, key: &str) -> Result<(), EngineError> {
    self.with_backend_mut(Operation::DeleteWorld, |backend, _game_id| match backend {
        Backend::Sqlite { pool } => {
            let conn = pool.conn();
            // First check for referencing games
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM games WHERE world_key = ?", [key])?;
            if count > 0 {
                return Err(EngineError::Validation(format!("Cannot delete world with {} games", count)));
            }
            conn.execute("DELETE FROM worlds WHERE key = ?", [key])?;
            // Map deletion cascades via FK
            Ok(())
        }
        _ => unreachable!(),
    })
}
```

**Edit**: `src/storage/backend/core.rs` — Add `Operation::DeleteWorld`.

### Step 5: Persona dropdown population

The world edit form needs a `<select>` for `player_key`. Add endpoint:

**Edit**: `src/server/worlds_fragment/handlers.rs`
```rust
pub async fn get_persona_list(State(state): State<AppState>) -> Response<axum::body::Body> {
    let personas = state.application_service.list_personas(state.as_game_service_context_or_default()).unwrap_or_default();
    let mut options = String::new();
    for p in personas {
        options.push_str(&format!("<option value=\"{}\">{}</option>", p.key, p.sheet.name));
    }
    crate::server::fragments::renderers::ok(format!("<select name=\"player_key\">{options}</select>"))
}
```

**Edit**: `src/application/application_service.rs`:
```rust
pub fn list_personas(&self, ctx: GameServiceContext) -> Result<Vec<crate::model::character::PlayerCard>, ApplicationError> {
    ctx.storage.list_personas().map_err(Into::into)
}
```

## Critical Files & Anchors

- `src/server/worlds_fragment/` — New module (4 files: mod.rs, handlers.rs, fragments.rs, template.rs if needed)
- `src/server/router.rs` — Route registration
- `assets/index.html:23, 90` — Tab button and content panel
- `src/application/application_service.rs` — Delegate world CRUD methods
- `src/storage/backend/worlds.rs` — Implement delete_world()

## Verification

1. **Worlds tab renders**: Start server, navigate to Worlds tab via browser. Panel shows existing worlds (`redmist_estate`, `test`).
2. **Create world via UI**: Fill form with valid data (e.g., key="cyberpunk_city", minimal map JSON with one room). Submit. Verify world appears in list and in DB.
3. **Edit world**: Click Edit on a world. Form populates. Change name. Save. Verify name updated.
4. **Delete blocked**: Try to delete `redmist_estate` (has games). Error: "Cannot delete world with N games".
5. **Delete succeeds**: Create test world with no games. Delete it. World removed from list.
6. **Persona dropdown**: Edit form shows dropdown with available personas (`julian`, `test_player`).
7. **Build**: `python build.py` passes.
