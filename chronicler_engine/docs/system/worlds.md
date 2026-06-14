# Worlds Management System

## Overview
The Worlds Management Tab provides a UI for creating, editing, and managing worlds in the Chronicler Engine. Worlds define the setting, lore, global rules, and map structure for games. Multiple games can reference a single world, enabling reuse of world building across multiple playthroughs.

## Location
- **Server module**: `src/server/worlds_fragment/`
- **Domain model**: `src/model/world.rs`
- **Storage**: `src/storage/backend/worlds.rs`
- **UI**: `assets/index.html` (Worlds tab)

## Architecture

### Fragment Module Structure
```
src/server/worlds_fragment/
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
| `starting_room_id` | String | Default room ID for new games |
| `scenarios` | Vec<StartingScenario> | Available starting scenarios |
| `default_scenario_id` | Option<String> | Default scenario if not specified |
| `default_room_image` | Option<String> | Fallback image for rooms without specific images |
| `player_key` | String | Default player persona key |

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

### Modal Form
Create/Edit uses a shared modal with dynamic content:
- **Hidden by default**, shown via JS when "Create New World" or "Edit" clicked
- **Form fields**: key (readonly for edit), name, description, global_rules (textarea), starting_room_id, player_key (dropdown), default_room_image, map_json (textarea), scenarios_json (textarea)
- **Submit**: Form posts to `/worlds` (create) or `/worlds/:key` (update)

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
3. **starting_room_id**: Must reference a room in the map (validated in storage)
4. **player_key**: Must reference an existing persona (validated in storage)
5. **map_json**: Valid JSON matching `MapDef` schema
6. **scenarios_json**: Valid JSON array of `StartingScenario` objects

### Delete Validation
1. **No referencing games**: Storage layer executes SQL `SELECT COUNT(*) FROM games WHERE world_key = ?`
2. **Returns `EngineError::ForeignKeyViolation`** if games reference the world
3. **Handler maps to 400 Bad Request** with user-friendly message
4. **Cascades to map**: Map deleted automatically via FK cascade

## Error Handling

### Handler Layer
- **Form parsing errors**: Return 400 with error message in fragment
- **Validation errors**: Return 400 with descriptive message
- **Database errors**: Return 500 Internal Server Error

### Fragment Renderers
- `render_worlds_panel(worlds, games_per_world) -> String`: HTML for worlds list
- `render_world_edit_form(world, map, scenarios, personas) -> String`: HTML for create/edit form with pre-filled JSON
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
- Persona dropdown populates correctly

### Manual Testing
- Tab navigation works
- Modal opens/closes correctly
- Form validation error messages render
- Worlds list refreshes after CRUD operations

## Performance Considerations

### Caching
- World list not cached (always fresh from DB)
- Persona list fetched on-demand for edit form

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
