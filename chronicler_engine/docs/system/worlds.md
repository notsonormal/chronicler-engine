# Worlds Management System

## Overview

The Worlds Management Tab provides a UI for creating, editing, and managing worlds in the Chronicler Engine. Worlds define the setting, lore, global rules, and map structure for games. Multiple games can reference a single world, enabling reuse of world building across multiple playthroughs.

## Location

- **Server module**: `src/adapters/driving/http/worlds_fragment/`
- **Domain model**: `src/domain/model/world.rs`
- **Storage**: `src/adapters/driven/storage/backend/worlds.rs`
- **CSS**: Scoped `.worlds-panel` rules under `assets/worlds.css` (extracted from `styles.css`)

## Architecture

### Fragment Module Structure

```
src/adapters/driving/http/worlds_fragment/
├── mod.rs          # Module exports
├── fragments.rs    # HTML fragment renderers
├── handlers.rs     # HTTP request handlers
└── tests/          # Unit + integration tests
    ├── fragments_tests.rs
    └── handlers_tests.rs
```

### Data Flow

```
Browser (Worlds Tab)
    │
    ├── hx-get /fragment/worlds → list_worlds_fragment()
    │       └── ApplicationService::list_worlds() → Storage::list_worlds()
    │
    ├── GET /worlds/:key/edit → edit_world_form_handler()
    │       └── ApplicationService::get_world(key) → loads WorldCard + MapDef
    │       └── Pre-fills form with map/scenarios JSON
    │
    ├── Form POST /worlds → create_world_handler()
    │       └── WorldForm::into_world_card() → parses/validates JSON
    │       └── ApplicationService::create_world() → Storage::create_world()
    │
    ├── Form POST /worlds/:key → update_world_handler()
    │       └── Path key used as canonical (ignores form.key)
    │       └── WorldForm::into_world_card() → parses/validates JSON
    │       └── ApplicationService::update_world() → Storage::update_world()
    │
    └── POST /worlds/:key/delete → delete_world_handler()
            └── ApplicationService::delete_world() → Storage::delete_world()
            └── Storage checks FK constraint (games referencing world)
```

## World Model

### WorldCard Fields

| Field | Type | Description |
|-------|------|-------------|
| `key` | String | Unique identifier (immutable after creation) |
| `name` | String | Display name |
| `description` | String | Short description |
| `global_rules` | Vec<String> | One rule per line, applied to all games in this world |
| `scenarios` | Vec<StartingScenario> | Available starting scenarios. Each scenario declares its own `starting_room_id` (default `"start"`). |
| `default_scenario_id` | Option<String> | Default scenario if not specified |
| `default_room_image` | Option<String> | Fallback image for rooms without specific images |

### World-Game Relationship

- **One-to-Many**: One world can have multiple games referencing it
- **Referential Integrity**: Cannot delete a world with referencing games
- **Checked At**: Storage layer (`Storage::delete_world()`) and handler layer (returns 400 with game count)

## API Endpoints

### Fragment Endpoints (HTMX)

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/fragment/worlds` | `list_worlds_fragment()` | Render worlds list panel |
| GET | `/fragment/worlds/new` | `new_world_form_handler()` | Render empty world creation form |
| GET | `/worlds/:key/edit` | `edit_world_form_handler()` | Render edit form with world data pre-filled |

### CRUD Handlers

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| POST | `/worlds` | `create_world_handler()` | Create new world from form (validates JSON via `WorldForm::into_world_card()`) |
| GET | `/worlds/:key/edit` | `edit_world_form_handler()` | Render edit form (pre-fills map/scenarios JSON) |
| POST | `/worlds/:key` | `update_world_handler()` | Update world from form (uses path key, validates JSON) |
| POST | `/worlds/:key/delete` | `delete_world_handler()` | Delete world (FK check at storage layer only) |

## UI Components

### Tab Structure

The Worlds tab is the third tab in the dashboard, positioned after "Prompt Presets":

```html
<button class="tab" data-tab="worlds">Worlds</button>
```

### Panel Layout

```html
<div class="tab-content" id="worlds-tab">
  <div class="worlds-panel" hx-get="/fragment/worlds" hx-trigger="load"></div>
</div>
```

### Inline Form (HTMX Swap)

Create/Edit uses inline HTMX swaps with `hx-target=".worlds-panel" hx-swap="outerHTML"` — no modal overlay:

- **Hidden by default**, shown when "Create New World" or "Edit" clicked (replaces the worlds list)
- **Form fields**: key (readonly for edit), name, description, global_rules (textarea), default_room_image, map_json (textarea), scenarios_json (textarea). `starting_room_id` lives inside each scenario object, not at the world level.
- **Submit**: Form posts to `/worlds` (create) or `/worlds/:key` (update)
- **Cancel**: Returns to worlds list via `hx-get="/fragment/worlds"` targeting `.worlds-panel`

### Worlds List Panel

Rendered by `render_worlds_panel()`:

- "Create New World" button at top
- Empty state message if no worlds
- List of worlds with name, description, game count, Edit/Delete buttons
- Delete button shows confirmation dialog via `hx-confirm`

## Validation Rules

### Create/Update Validation

1. **key**: Required, unique, alphanumeric + underscore only
2. **name**: Required, non-empty
3. **scenarios_json**: Valid JSON array of `StartingScenario` objects (each scenario's `starting_room_id` must reference a room in the map; validated in storage)
4. **map_json**: Valid JSON matching `MapDef` schema

### Delete Validation

1. **No referencing games**: Storage layer executes SQL `SELECT COUNT(*) FROM games WHERE world_key = ?`
2. **Returns `EngineError::WorldHasGames`** if games reference the world (typed variant, not string-matched)
3. **Handler uses `is_user_displayable()`** for type-driven branching — displayable errors render inline; others return error status
4. **Cascades to map**: Map deleted automatically via FK cascade

## Error Handling

### Handler Layer

- **Form parsing errors**: Return 400 with error message in fragment
- **Validation errors**: Return 400 with descriptive message
- **Database errors**: Return 500 Internal Server Error

### Fragment Renderers

- `render_worlds_panel(worlds, games_per_world) -> String`: HTML for worlds list
- `render_world_edit_form(world, map, scenarios) -> String`: HTML for create/edit form with pre-filled JSON
- `ok(html: String) -> Response`: Returns 200 OK with HTML fragment
- `ok_refresh() -> Response`: Returns 200 with HTMX refresh header
- `bad_request(msg: String) -> Response`: Returns 400 with error message
- `internal_error(msg: String) -> Response`: Returns 500 with error message

## Testing Strategy

### Unit Tests

- Fragment renderers (HTML generation)
- Form parsing and validation
- Error message formatting

### Integration Tests

- Create world via HTTP POST
- Edit world loads correct data
- Update persists changes
- Delete blocked by referencing games
- Delete succeeds when no games reference

### Manual Testing

- Tab navigation works
- Inline form swap works (no modal)
- Form validation error messages render
- Worlds list refreshes after CRUD operations
- Cancel button returns to worlds list

## Performance Considerations

### Caching

- World list not cached (always fresh from DB)
- Persona list managed on Games tab (per ADR-026); worlds form no longer loads personas

### Query Optimization

- Single query for world list
- JOIN used to count games per world in list endpoint
- Index on `games.world_key` for fast delete checks

## Future Enhancements

### Planned

- World export/import as JSON
- World duplication (copy as new)
- Map visualizer integration
- Scenario management UI

### Backlog

- World templates (pre-configured worlds)
- Bulk operations (delete multiple)
- World versioning/history

## Related Documents

- [`dashboard.md`](dashboard.md) - Dashboard layout and tabs
- [`storage.md`](storage.md) - Storage backend design
- [`../architecture/system.md`](../architecture/system.md) - Core architecture
- [`../plans/worlds-management-tab-implementation.md`](../plans/worlds-management-tab-implementation.md) - Implementation plan
