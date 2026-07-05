# Worlds Management System

## Overview

The Worlds Management Tab provides a UI for creating, editing, and managing worlds in the Chronicler Engine. Worlds define the setting, lore, global rules, and map structure for games. Multiple games can reference a single world, enabling reuse of world building across multiple playthroughs.

## Architecture

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

- **Referential Integrity**: A world with referencing games cannot be deleted; the storage layer (`Storage::delete_world()`) and handler layer (returns 400 with game count) both enforce this.

## Worlds Tab UI

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
