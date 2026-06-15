# World Fragment Quality Remediation Plan

**Date:** 2026-06-15
**Status:** IMPLEMENTED

## Context

The uncommitted changes add a worlds panel UI with modal-based CRUD, a global HTMX error handler, and a `delete_world_handler` that returns inline error HTML for validation failures. The thermo-nuclear review flagged three structural regressions: (1) string-matching error branching, (2) HTML built via `format!` in a handler, (3) two competing error display paths. Additionally, the CSS is monolithic at 2120 lines with 81% gradient duplication. This plan fixes the structural issues in the uncommitted code and aligns them with existing codebase patterns.

## Pattern Comparison: What Exists

| Area | Pattern A (settings/presets) | Pattern B (games/worlds) | What should win |
|------|-----|-----|-----|
| Return type | `Html<String>` | `Response<Body>` via `ok()` | **Pattern B** — already used by worlds |
| Error display | Inline `<span class='error'>` in `Html<String>` | Status codes via `app_err_to_response()` | **Neither** — see Approach |
| Validation errors | String check in handler logic | `EngineError::Parse(String)` from storage | **Typed variant** — new `WorldHasGames` |
| CSS | Monolithic single file, per-feature sections | Same | **Decompose** into per-feature files |
| Modal trigger | `onclick` → JS function with `htmx.ajax()` | Declarative `hx-get`/`hx-target` | **JS function** — already chosen, concur |
| Button CSS | Context-scoped selectors (`.panel button`) | Same, 4–6× duplicated gradients | **Utility classes** (`.btn-primary`, `.btn-danger`, `.btn-cyan`) |

## Approach

### Step 1: Add `EngineError::WorldHasGames` variant ✅

Replace the misused `EngineError::Parse("Cannot delete world with N games")` with a typed variant.

**File:** `chronicler_engine/src/error.rs`
- Added variant after `WorldNotFound`:
  ```rust
  #[error("Cannot delete world with {game_count} games")]
  WorldHasGames { game_count: usize },
  ```

**File:** `chronicler_engine/src/storage/backend/worlds.rs`
- Replaced `EngineError::Parse(format!("Cannot delete world with {count} games"))` with `EngineError::WorldHasGames { game_count: count as usize }`
- Replaced `EngineError::Parse(format!("Cannot delete world with {game_count} games"))` with `EngineError::WorldHasGames { game_count }`

### Step 2: Add `is_user_displayable()` to `ApplicationError` ✅

This replaces the string-matching in the handler with type-driven branching.

**File:** `chronicler_engine/src/application/application_service.rs`
- Added method to `ApplicationError`:
  ```rust
  pub fn is_user_displayable(&self) -> bool {
      match self {
          ApplicationError::Validation(_) => true,
          ApplicationError::Engine(EngineError::WorldHasGames { .. }) => true,
          _ => false,
      }
  }
  ```

### Step 3: Simplify `delete_world_handler` — remove string-matching ✅

**File:** `chronicler_engine/src/server/worlds_fragment/handlers.rs`
- Replaced string-matching with type-driven dispatch:
  ```rust
  Err(e) if e.is_user_displayable() => {
      let error_html = render_error(&e.to_string());
      ok(format!(r#"<li class="world-item">{}</li>"#, error_html))
  }
  Err(e) => app_err_to_response(e),
  ```

### Step 4: Extract `render_error` usage to template — NO CHANGE ✅

**Decision:** Keep the `format!` wrapper. The handler must return an `<li class="world-item">` to satisfy the HTMX swap contract. A template for this single element would be over-abstraction. `render_error()` already handles HTML escaping — no XSS risk.

### Step 5: Fix the global `htmx:beforeSwap` handler ✅

**File:** `chronicler_engine/assets/index.html`
- Removed `evt.preventDefault()` — HTMX will still swap the error HTML into the target, AND the notification shows. The notification is additive, not a replacement for the in-context error display.

### Step 6: Extract worlds CSS into its own file ✅

**File:** Created `chronicler_engine/assets/worlds.css`
- Moved worlds-specific selectors from `styles.css` into `worlds.css`
- Kept `.error-message` in `styles.css` (global, used by `render_error()`)

**File:** `chronicler_engine/assets/index.html`
- Added `<link rel="stylesheet" href="/assets/worlds.css?v=1" />` after existing stylesheet link

### Step 7: Add shared button utility classes and deduplicate gradients ✅

**Step 7a:** Added `.btn-primary`, `.btn-cyan`, `.btn-danger` utility classes to `styles.css` after design tokens.

**Step 7b:** Removed ~170 lines of duplicate gradient CSS from:
- `.settings-panel button` blocks → comment marker only
- `.prompt-presets-panel button` blocks → comment marker only
- `.save-load-panel button` / `.btn-switch` / `.btn-delete` / `.btn-new-game` / `.btn-reset` blocks → comment marker only

**Step 7c:** Updated template HTML to use utility classes:
- `settings_fragment/template.rs` + `fragments.rs`: `class="primary"` → `class="btn-primary"`, `class="danger"` → `class="btn-danger"`
- `prompt_presets_fragment/template.rs` + `fragments.rs`: same
- `games_fragment/template.rs`: `class="btn-switch"` → `class="btn-primary"`, `class="btn-delete"` → `class="btn-danger"`, `class="btn-reset"` → `class="btn-danger"`
- `worlds_fragment/template.rs`: `class="danger"` → `class="btn-danger"`, added `btn-cyan` and `btn-primary` classes

**Step 7d:** Simplified `worlds.css` — gradient rules removed, only layout overrides remain.
