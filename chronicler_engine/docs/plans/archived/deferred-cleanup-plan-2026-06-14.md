# Deferred Cleanup: Askama Templates, Module Reorganization, and Games Migration

## Context

Three items were deferred from the thermo-nuclear code quality review of the Worlds Management Tab:

1. **String-concat HTML in `worlds_fragment/fragments.rs`** — 178 lines of `html.push_str(&format!(...))` when the codebase convention is Askama templates with inline `#[template(source = r#"..."#)]` (see `settings_fragment/template.rs`, `prompt_presets_fragment/template.rs`).
2. **Module organization inconsistency** — Games live in `server/fragments/games.rs` (flat file); worlds live in `server/worlds_fragment/` (sub-module). Two organizational patterns for the same feature type at the same layer.
3. **Games module also uses string-concat HTML** — `games.rs` is 162 lines of `html.push_str`, same tech debt as worlds, just older.

End state: both worlds and games use Askama templates for their main panels and edit forms, and both live as sub-modules under `server/` (matching `settings_fragment/` and `prompt_presets_fragment/`). No string-concat HTML remains in either feature. The `fragments/` module retains only cross-cutting shared code (`renderers.rs`, `endpoints.rs`, `actions.rs`, `history.rs`, `misc.rs`).

---

## Approach

### Step 1: Create Askama template structs for worlds panel and edit form

**Target**: New file `chronicler_engine/src/server/worlds_fragment/template.rs`

Create two template structs following the `SettingsTemplate` / `PromptPresetsTemplate` convention (inline `source`, `ext = "html"`):

```rust
#[derive(Template)]
#[template(source = r#"..."#, ext = "html")]
pub struct WorldsPanelTemplate {
    pub worlds: Vec<WorldRowView>,
}
```

Where `WorldRowView` is a flat view model (also in `template.rs`):
```rust
pub struct WorldRowView {
    pub key: String,
    pub name: String,
    pub description: String,
    pub game_count: usize,
}
```

```rust
#[derive(Template)]
#[template(source = r#"..."#, ext = "html")]
pub struct WorldFormTemplate {
    pub is_edit: bool,
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: String,
    pub starting_room_id: String,
    pub personas: Vec<PersonaOption>,
    pub player_key: String,
    pub default_room_image: String,
    pub map_json: String,
    pub scenarios_json: String,
}
```

Where `PersonaOption` is a simple view-model struct (also in `template.rs`):
```rust
pub struct PersonaOption {
    pub key: String,
    pub name: String,
    pub selected: bool,
}
```

**Template source for `WorldsPanelTemplate`**: port the HTML from current `render_worlds_panel()` in `fragments.rs:10-42`. Askama auto-escapes all `{{ var }}` output — remove all explicit `html_escape()` calls in the template. Use `{% for world in worlds %}` iteration. Game count is accessed as `{{ world.game_count }}` from the flattened `WorldRowView`.

**Template source for `WorldFormTemplate`**: port the HTML from current `render_world_edit_form()` in `fragments.rs:44-178`. All field values rendered with `{{ field }}` are auto-escaped by Askama. The persona dropdown uses `{% for p in personas %}` with `{% if p.selected %}selected{% endif %}`.

**Edge cases**:
- Empty worlds list → `{% if worlds.is_empty() %}` branch in template (same as current logic).
- `default_room_image` is `Option<String>` → flatten to `String` in the view model (empty string if `None`) before passing to template.
- `map_json` / `scenarios_json` are pre-serialized JSON strings → Askama will HTML-escape them via `{{ map_json }}`. Inside `<textarea>`, HTML entities render as literal characters, so this is safe — but a value containing `</textarea>` would be escaped to `&lt;/textarea&gt;` which is the correct safe behavior.

**Dependencies**: None — this step creates a new file only.

### Step 2: Replace string-concat renderers in `worlds_fragment/fragments.rs` with template calls

**Target**: `chronicler_engine/src/server/worlds_fragment/fragments.rs`

Delete the function bodies of `render_worlds_panel` and `render_world_edit_form`. Replace with:

```rust
pub fn render_worlds_panel(worlds: &[WorldCard], games_per_world: &HashMap<String, usize>) -> String {
    let rows: Vec<WorldRowView> = worlds.iter().map(|w| {
        let game_count = games_per_world.get(&w.key).copied().unwrap_or(0);
        WorldRowView { key: w.key.clone(), name: w.name.clone(), description: w.description.clone(), game_count }
    }).collect();
    WorldsPanelTemplate { worlds: rows }.render().unwrap_or_default()
}

pub fn render_world_edit_form(world: Option<&WorldCard>, map: Option<&MapDef>, scenarios: &[StartingScenario], personas: &[PlayerCard]) -> String {
    let is_edit = world.is_some();
    let default_world = WorldCard::default();
    let w = world.unwrap_or(&default_world);
    let persona_options: Vec<PersonaOption> = personas.iter().map(|p| PersonaOption {
        key: p.key.clone(), name: p.sheet.name.clone(), selected: p.key == w.player_key,
    }).collect();
    WorldFormTemplate {
        is_edit,
        key: w.key.clone(),
        name: w.name.clone(),
        description: w.description.clone(),
        global_rules: w.global_rules.join("\n"),
        starting_room_id: w.starting_room_id.clone(),
        personas: persona_options,
        player_key: w.player_key.clone(),
        default_room_image: w.default_room_image.clone().unwrap_or_default(),
        map_json: map.map(|m| serde_json::to_string_pretty(m).unwrap_or_default()).unwrap_or_default(),
        scenarios_json: if scenarios.is_empty() { String::new() } else { serde_json::to_string_pretty(scenarios).unwrap_or_default() },
    }.render().unwrap_or_default()
}
```

Remove all inline `crate::server::fragments::html_escape(...)` calls — Askama auto-escapes. Remove unused `use crate::model::*` imports that were only needed by the old template bodies.

Update `src/server/worlds_fragment/mod.rs` to add `mod template;` and export the view model types if they're needed by tests:
```rust
mod fragments;
mod handlers;
mod template;

pub use fragments::{render_world_edit_form, render_worlds_panel};
pub use handlers::{
    create_world_handler, delete_world_handler, edit_world_form_handler, list_worlds_fragment,
    new_world_form_handler, update_world_handler,
};
```

**Dependencies**: Step 1 must be complete.

### Step 3: Move `games.rs` into `games_fragment/` sub-module

**Target**: Create `chronicler_engine/src/server/games_fragment/` as a new sub-module.

**3a.** Create directory `src/server/games_fragment/`.

**3b.** Create `src/server/games_fragment/mod.rs`:
```rust
mod handlers;

pub use handlers::{create_game_handler, delete_game_handler, list_games_fragment, switch_game_handler};
```

**3c.** Move `src/server/fragments/games.rs` → `src/server/games_fragment/handlers.rs` (verbatim copy — no logic changes). Update the `use super::renderers::{...}` import to `use crate::server::fragments::renderers::{...}` since the module parent changes.

**3d.** Remove `mod games;` from `src/server/fragments/mod.rs`. Remove `pub use games::{...};` from the same file. Add `pub use crate::server::games_fragment;` re-export if needed, or update the router import.

**3e.** Update `src/server/mod.rs`: add `pub mod games_fragment;`

**3f.** Update `src/server/router.rs`: change `fragments::list_games_fragment` → `games_fragment::list_games_fragment`, `fragments::create_game_handler` → `games_fragment::create_game_handler`, `fragments::switch_game_handler` → `games_fragment::switch_game_handler`, `fragments::delete_game_handler` → `games_fragment::delete_game_handler`.

**3g.** Move `src/server/fragments/games_tests.rs` → `src/server/games_fragment/handlers_tests.rs`. Update the import in the test file from `use crate::server::fragments::games::{...}` to `use crate::server::games_fragment::handlers::{...}`. Update `src/server/fragments/mod.rs` to remove `mod games_tests;` from the `#[cfg(test)]` section. Add `#[cfg(test)] mod handlers_tests;` to `src/server/games_fragment/mod.rs`.

**3h.** Search for any other references to `fragments::games::` or `fragments::list_games_fragment` etc. across the codebase and update them. Run: `search pattern="fragments::games|fragments::list_games|fragments::create_game|fragments::switch_game|fragments::delete_game"` — update every hit.

**Dependencies**: None — independent of Steps 1-2.

### Step 4: Create Askama template structs for games panel

**Target**: New file `chronicler_engine/src/server/games_fragment/template.rs`

Port the HTML from `games.rs` handler's inline string-concat into a template struct:

```rust
#[derive(Template)]
#[template(source = r#"..."#, ext = "html")]
pub struct GamesPanelTemplate {
    pub active_game: Option<GameRowView>,
    pub saved_games: Vec<GameRowView>,
    pub current_game_id: Option<u64>,
}

pub struct GameRowView {
    pub id: u64,
    pub name: String,
    pub world_key: String,
}
```

The template covers: active game section, saved games list, and action buttons (New Game, Reset).

**Dependencies**: Step 3 (directory must exist).

### Step 5: Replace string-concat HTML in `games_fragment/handlers.rs` with template calls

**Target**: `chronicler_engine/src/server/games_fragment/handlers.rs`

Refactor `list_games_fragment` to construct `GamesPanelTemplate` and call `.render()`. Remove all `html.push_str(...)` lines. The handler logic for loading context/games stays the same — only the HTML assembly changes.

Other handlers in the file (`create_game_handler`, `switch_game_handler`, `delete_game_handler`) don't render HTML (they return status codes or redirects) — no changes to those.

Remove `use crate::server::fragments::renderers::html_escape;` if the only usage was in the string-concat HTML.

**Dependencies**: Steps 3 and 4.

### Step 6: Update documentation

**Target**: `chronicler_engine/docs/system/worlds.md`, `docs/architecture/system.md`, any file referencing `fragments/games.rs`

- `worlds.md`: no changes needed (already updated in prior review).
- `system.md`: update module listing from `games.rs` inside `fragments/` to `games_fragment/` sub-module.
- Any doc referencing `server/fragments/games.rs` path → update to `server/games_fragment/`.

**Dependencies**: Steps 3-5.

### Step 7: Build and test

```bash
cd chronicler_engine && python build.py
```

This runs `cargo fmt`, `cargo clippy`, `cargo nextest run`. All 1186+ tests must pass.

Specific new tests to verify:
- `test_list_worlds_fragment_returns_html` — should still find `worlds-panel` in output
- `test_new_world_form_handler_returns_form` — should still find `world-form-container`
- `test_create_world_handler_invalid_json` — should still get "Invalid map JSON" error
- `test_delete_world_handler_exists` — should still respond

No new test files needed — the HTTP integration tests in `tests/http/worlds_fragment.rs` validate the templates render correctly end-to-end. The template compilation itself is verified by `cargo check` (Askama macros fail at compile time on syntax errors).

**Dependencies**: All previous steps.

---

## Critical files & anchors

| File | Anchor | Why |
|------|--------|-----|
| `src/server/worlds_fragment/fragments.rs` | `render_worlds_panel`, `render_world_edit_form` | String-concat HTML to replace with Askama template calls |
| `src/server/fragments/games.rs` | `list_games_fragment` (lines 18-91) | String-concat HTML to migrate and replace |
| `src/server/settings_fragment/template.rs` | `SettingsTemplate` | Pattern to copy: inline Askama `#[template(source = r#"..."#)]` |
| `src/server/fragments/mod.rs` | Lines 6, 20 | Remove `mod games;` and `pub use games::...;` |
| `src/server/router.rs` | Lines 62-84 | Update import paths from `fragments::` to `games_fragment::` |

---

## Verification

### Askama templates compile and render
```bash
cd chronicler_engine && cargo check
```
Fail → template syntax error in `#[template(source = ...)]`.

### Worlds HTTP tests pass
```bash
cd chronicler_engine && cargo nextest run worlds_fragment
```
19 tests must pass — validates template rendering end-to-end through real HTTP handlers.

### All games tests pass
```bash
cd chronicler_engine && cargo nextest run games
```
Validates the games sub-module migration didn't break anything.

### Full build
```bash
cd chronicler_engine && python build.py
```
1186+ tests pass, clippy clean, fmt clean.

### Manual smoke test
```bash
cd chronicler_engine && cargo run
```
Open `http://localhost:3000` in browser. Verify:
1. "Save/Load" tab shows games list (not empty/broken)
2. "Worlds" tab shows worlds list
3. Clicking "Create New World" opens form with all fields
4. Clicking "Edit" on a world pre-fills map/scenarios JSON in textareas

---

## Assumptions & contingencies

- **Askama auto-escaping is sufficient** for all template fields. `<textarea>` content rendered via `{{ value }}` will HTML-escape, which prevents `</textarea>` injection. Inside a textarea, `&lt;/textarea&gt;` renders literally — correct and safe. If a user reports that escaped entities appear literally in the textarea (e.g., `&amp;` instead of `&`), the fix is to use `{{ value|safe }}` for textarea content specifically — but this must be audited per-field. Default to escaped (`{{ value }}`) and only unescape if proven necessary.
- **`HashMap` in Askama templates**: Askama supports `.get()` on `HashMap` but not indexing. The plan avoids this by flattening game counts into `WorldRowView`. If Askama can do `{{ games_per_world.get(world.key) }}` directly, the flattening is unnecessary — confirm by trying `.get()` in the template source; if it compiles, keep the simpler struct.
- **Game handler tests location**: `fragments/games_tests.rs` uses `use crate::server::fragments::games::{...}` fully-qualified imports (not `use super::*`). When moving to `games_fragment/handlers_tests.rs`, update to `use crate::server::games_fragment::handlers::{...}`.
- **`fragments/` module still needs `renderers.rs`**: The shared renderers (`html_escape`, `ok`, `bad_request`, `internal_error`, etc.) remain in `fragments/renderers.rs` — they're imported by many modules. Only `games.rs` is extracted. No other file names in `fragments/` change.

## Implementation Status

**COMPLETED** 2026-06-14

All steps implemented successfully:
- ✅ Worlds Askama templates created and integrated
- ✅ Games module migrated to `games_fragment/` sub-module
- ✅ Games Askama templates created and integrated
- ✅ All documentation updated
- ✅ Full test suite passes (1186 tests)
- ✅ Clippy clean, fmt clean
