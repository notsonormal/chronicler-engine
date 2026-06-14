# Cross-World Game Flow UI

## Context

After Plans 1 and 2, the backend supports multi-world games and a Worlds tab exists for world management. This plan updates the Save/Load game flow to: (1) show games from all worlds with world badges, and (2) add a world picker when creating a new game. This completes the multi-world UX.

## Prerequisites

- Plan 1 (Multi-World Data Foundation) — COMPLETE
- Plan 2 (Worlds Management Tab) — COMPLETE

## Approach

### Step 1: Update Save/Load panel to show all games with world badges

**Edit**: `src/server/fragments/games.rs` — `list_games_fragment()`

Current behavior (lines 37, 63) filters games by `current_world`. Remove that filter:

```rust
// REMOVE: let current_world = &state.world.name;  // No longer in AppState
// REMOVE: filter by current_world

let active_id = state
    .application_service
    .current_game_id(state.as_game_service_context_or_default());

let games = match state.application_service.list_games(state.as_game_service_context_or_default()) {
    Ok(g) => g,
    Err(e) => return internal_error(e.to_string()),
};

let mut html = String::new();
html.push_str("<div class=\"save-load-panel\">");

// Active game section
html.push_str("<div class=\"save-load-section\">");
html.push_str("<h2>Active Game</h2>");
let active_game = games.iter().find(|g| g.id == active_id);
if let Some(game) = active_game {
    html.push_str(&format!(
        "<div class=\"game-card active\"><strong>{}</strong> <span class=\"world-badge\">{}</span>",
        game.name, game.world_name
    ));
    // Continue with existing active game display...
} else {
    html.push_str("<p>No active game.</p>");
}
html.push_str("</div>");

// All games list (no world filter)
html.push_str("<div class=\"save-load-section\">");
html.push_str("<h2>All Games</h2>");
html.push_str("<div class=\"games-list\">");

let other_games: Vec<_> = games.iter().filter(|g| g.id != active_id).collect();

if other_games.is_empty() {
    html.push_str("<div class=\"games-empty\">No saved games.</div>");
} else {
    for game in other_games {
        html.push_str(&format!(
            "<div class=\"game-item\"><strong>{}</strong> <span class=\"world-badge\">{}</span> \
             <button hx-post=\"/games/{}/switch\">Switch</button> \
             <button hx-post=\"/games/{}/delete\">Delete</button></div>",
            game.name, game.world_name, game.id, game.id
        ));
    }
}

html.push_str("</div>"); // games-list
html.push_str("</div>"); // save-load-section

// Actions row
html.push_str("<div class=\"save-load-actions\">");
html.push_str(r#"<button class="btn-new-game" hx-get="/games/new-world-picker">New Game</button>"#);
html.push_str(r#"<button class="btn-reset" hx-post="/reset" hx-confirm="Reset current game?">Reset</button>"#);
html.push_str("</div>");

html.push_str("</div>"); // save-load-panel
ok(html)
```

### Step 2: Add CSS for world badges

**Edit**: `assets/styles.css`

Add near other badge/tag styles:
```css
.world-badge {
    display: inline-block;
    padding: 2px 8px;
    margin-left: 8px;
    font-size: 0.75rem;
    background: var(--accent-secondary, #5a6c7d);
    color: var(--text-inverse, #fff);
    border-radius: 4px;
    font-weight: normal;
}

.games-list .game-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    margin: 4px 0;
    background: var(--bg-tertiary, #2a2f3a);
    border-radius: 4px;
}

.games-list .game-item strong {
    flex: 1;
}

.games-list .game-item .world-badge {
    margin-left: 12px;
}
```

### Step 3: World picker for new game creation

**Edit**: `src/server/fragments/games.rs` — New handler:

```rust
pub async fn new_game_world_picker(State(state): State<AppState>) -> Response<axum::body::Body> {
    let worlds = match state.application_service.list_worlds(state.as_game_service_context_or_default()) {
        Ok(w) => w,
        Err(e) => return render_error(&format!("Failed to load worlds: {e}")),
    };
    
    if worlds.is_empty() {
        return render_error("No worlds available. Create a world first in the Worlds tab.");
    }
    
    let mut html = String::new();
    html.push_str("<div class=\"world-picker\">");
    html.push_str("<h3>Select a World</h3>");
    html.push_str("<form hx-post=\"/games\" hx-swap=\"none\">");
    html.push_str("<select name=\"world_key\" required>");
    for world in &worlds {
        html.push_str(&format!(
            "<option value=\"{}\" title=\"{}\">{}</option>",
            world.key, world.description, world.name
        ));
    }
    html.push_str("</select>");
    html.push_str("<button type=\"submit\" class=\"btn-primary\">Create Game</button>");
    html.push_str("</form>");
    html.push_str("</div>");
    
    ok(html)
}
```

**Edit**: `src/server/fragments/games.rs` — Update `create_game_handler()`:

```rust
pub async fn create_game_handler(
    State(state): State<AppState>,
    Form(params): Form<std::collections::HashMap<String, String>>,
) -> Response<axum::body::Body> {
    let world_key = params.get("world_key").cloned().unwrap_or_else(|| {
        // Fallback to current game's world for backward compatibility
        let ctx = state.as_game_service_context_or_default();
        ctx.world.key.clone()
    });
    
    if state.is_generating.load(std::sync::atomic::Ordering::SeqCst) {
        return service_unavailable_generating();
    }
    
    match state.application_service.create_game(state.as_game_service_context_or_default(), &world_key) {
        Ok(_) => ok_refresh(),
        Err(e) => app_err_to_response(e),
    }
}
```

**Edit**: `src/server/router.rs` — Add route:
```rust
.route("/games/new-world-picker", get(fragments::new_game_world_picker))
```

Update `/games` route to accept POST with form data (already does, just noting it's now world-key-aware).

### Step 4: Inline world picker via HTMX (optional enhancement)

Instead of replacing the "New Game" button with a GET to a picker page, use HTMX inline reveal:

**Edit**: `src/server/fragments/games.rs` — `list_games_fragment()`:

Change the New Game button to:
```html
<button class="btn-new-game" hx-get="/games/new-world-picker" hx-target="#new-game-container" hx-swap="innerHTML">New Game</button>
<div id="new-game-container"></div>
```

This reveals the world picker inline when clicked, replacing the button text with the form.

### Step 5: Update documentation

**Edit**: `docs/system/dashboard.md`

Add section:
```markdown
## Save/Load Tab

The Save/Load tab shows all games across all worlds. Each game displays:
- Game name
- World name (badge)
- Switch button (activates that game)
- Delete button (removes game and all history)

Creating a new game opens a world picker dropdown. Select a world to create a new game under that world. Games are named `{WorldName}_{Date}_{N}`.
```

**Edit**: `docs/reference/data_schemas.md`

Games table now includes `world_key TEXT` column. Update schema documentation.

## Critical Files & Anchors

- `src/server/fragments/games.rs:18-98` — `list_games_fragment()`. Remove world filter, add world badges.
- `src/server/fragments/games.rs:100-115` — `create_game_handler()`. Accept `world_key` form param.
- `src/server/fragments/games.rs` — Add `new_game_world_picker()` handler.
- `assets/styles.css` — Add `.world-badge` styles.
- `assets/index.html` — Tab switching (already handles Worlds tab via data-tab, no changes needed here).

## Verification

1. **Cross-world games visible**: Create games under `redmist_estate` and `test` worlds. Open Save/Load tab. Both games visible with world badges.
2. **Switch between worlds**: Click Switch on a test-world game. Dashboard reloads with that game active. Verify correct world name in header.
3. **New game world picker**: Click "New Game". World picker dropdown appears with both worlds listed.
4. **Create game in test world**: Select `test` world, submit. New game created under test world. Appears in list with badge.
5. **Browser smoke test**: Run server, open in browser. Navigate tabs. Create game. Switch games. No JS console errors.
6. **Build**: `python build.py` passes.
