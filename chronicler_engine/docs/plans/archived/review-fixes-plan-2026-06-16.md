# Review Fixes Plan

## Context

Five findings from the thermo-nuclear code quality review of uncommitted changes. The diff renames "Save / Load" → "Games" and removes the world modal, but leaves the old `save-load` naming throughout the DOM/CSS, duplicates the global `.btn-primary` via a scoped override, leaves `styles.css` at 1755 lines when a natural extraction exists, makes world form submit do a full page reload while Cancel does a smooth inline swap, and has two deleted plan files that need staging. This plan fixes all five.

## Approach

### Step 1 — Rename `save-load` → `games` in DOM, CSS, templates, and docs

Every occurrence of the `save-load` identifier in active code gets renamed to `games`. This is a straight find-and-replace; no structural change.

**Replacements (all case-insensitive where applicable):**

| File | Old | New |
|------|-----|-----|
| `chronicler_engine/assets/index.html:26` | `data-tab="save-load"` | `data-tab="games"` |
| `chronicler_engine/assets/index.html:106` | `id="save-load-tab"` | `id="games-tab"` |
| `chronicler_engine/assets/index.html:108` | `class="save-load-panel"` | `class="games-panel"` |
| `chronicler_engine/assets/styles.css:1462-1463` | `/* Games Tab */\n#save-load-tab` | `/* Games Tab */\n#games-tab` |
| `chronicler_engine/assets/styles.css:1467` | `.save-load-panel` | `.games-panel` |
| `chronicler_engine/assets/styles.css:1478` | `.save-load-section` | `.games-section` |
| `chronicler_engine/assets/styles.css:1640` | `/* Save/Load Buttons` | `/* Games Buttons` |
| `chronicler_engine/src/server/games_fragment/template.rs:16` | `<div class="save-load-panel">` | `<div class="games-panel">` |
| `chronicler_engine/src/server/games_fragment/template.rs:17` | `<div class="save-load-section">` | `<div class="games-section">` |
| `chronicler_engine/src/server/games_fragment/template.rs:34` | `<div class="save-load-section new-game-section">` | `<div class="games-section new-game-section">` |
| `chronicler_engine/src/server/games_fragment/template.rs:52` | `<div class="save-load-section">` | `<div class="games-section">` |
| `chronicler_engine/docs/system/ui_design.md:335` | `.save-load-panel button` | `.games-panel button` |
| `chronicler_engine/docs/CHANGELOG.md:8` | `data-tab="save-load"` | `data-tab="games"` — update the parenthetical |

**Docs/archive files — leave as-is.** Archived plans (`docs/plans/archived/`) are historical records; do not rewrite them. The active `docs/plans/games-tab-restructure-plan.md` is about to be archived itself, so leave it too. The only changelog edit is line 8's parenthetical.

**Edge:** The tab switching JS in `index.html` reads `data-tab` to construct the ID lookup (`id = "${tab}-tab"`). Since we're renaming both `data-tab` and the corresponding `id` consistently (`games` → `games-tab`), the JS still works. Verify by reading the tab-switching code.

### Step 2 — Delete `.new-game-form .btn-primary` scoped override, adjust global `.btn-primary` padding

The scoped override at `styles.css:1621-1638` duplicates the global `.btn-primary` (lines 65–78) with identical gradient, border, and hover — then bumps padding from `8px 16px` to `8px 20px` and font-size from `--font-size-small` to `--font-size-base`. Delete the scoped override entirely.

The real difference: form buttons need slightly more padding and a standard font size. Adjust the global `.btn-primary`:
- Change `padding: 8px 16px` → `padding: 8px 20px` (the form button size, which looks better in all current usage)
- Change `font-size: var(--font-size-small)` → `font-size: var(--font-size-base)` (consistent with the engine's other buttons)

Delete these CSS blocks entirely:
- `.new-game-form .btn-primary { ... }` (lines 1621–1632)
- `.new-game-form .btn-primary:hover { ... }` (lines 1635–1638)

Also delete the comment on line 1640 (`/* Save/Load Buttons — ... */`) — it's orphaned and will be renamed per Step 1 anyway.

**Callers of `.btn-primary`:** search confirms it's used in `templates.rs` (TextCheckPreview Send), prompt_presets_fragment forms, settings_fragment, worlds_fragment, and the games panel. All of these look fine with slightly larger padding and base font size. The sole reason the original had `--font-size-small` was the action-bar Send button, but that has its own `#submit-btn` styles that override sizing.

No risk to the action bar. The `#command-form button` (line 506) has its own full styling — gradient, padding, font-size — it does NOT use `.btn-primary`. Only `.btn-primary` users (Games, Settings, Prompt Presets, Worlds, TextCheckPreview) are affected.

### Step 3 — Extract `games.css` from `styles.css`

Follow the exact pattern of `worlds.css`: a separate CSS file loaded via `<link>` in `index.html`, containing all Games-panel rules.

**Extract these rules from `styles.css` into a new `chronicler_engine/assets/games.css`:**
- `/* Games Tab */` comment + `#games-tab { overflow: hidden; }` (line ~1462 after Step 1)
- `.games-panel { ... }` and all descendant selectors through `.game-actions { ... }` (lines ~1467–1556)
- `.active-game-info { ... }` and `.active-game-info .game-name { ... }` (lines ~1558–1570)
- `.btn-reset-small { ... }` and `:hover` (lines ~1572–1588)
- `.new-game-form { ... }` and all descendants: `.form-row`, `select`, `select:focus`, (lines ~1590–1638 — minus the deleted `.btn-primary` override from Step 2) 
- `/* Games Buttons */` comment (line ~1640 after edits)

**In `index.html`:** Add `<link rel="stylesheet" href="/assets/games.css?v=1" />` after the `worlds.css` link (line 7). Bump `worlds.css` cache-buster from `v=3` to `v=4` since `worlds.css` is also modified in this commit.

**Estimated lines removed from `styles.css`:** ~120 lines. This brings it from 1755 down to ~1635.

### Step 4 — World form submit: return inline HTML instead of full page reload

Two handlers call `ok_refresh()` on success:

1. `create_world_handler` — `chronicler_engine/src/server/worlds_fragment/handlers.rs:105`
2. `update_world_handler` — `chronicler_engine/src/server/worlds_fragment/handlers.rs:187`

**Change both to return the re-rendered worlds panel HTML** (same content Cancel returns to), so the HTMX inline swap replaces `.worlds-panel` with the updated list.

For `create_world_handler` (line 105), replace:
```rust
Ok(_) => ok_refresh(),
```
with:
```rust
Ok(_) => {
    let worlds = state.application_service.list_worlds(ctx.clone());
    let games = state.application_service.list_games(ctx).unwrap_or_default();
    let mut games_per_world: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for game in &games {
        *games_per_world.entry(game.world_key.clone()).or_insert(0) += 1;
    }
    ok(render_worlds_panel(
        &worlds.unwrap_or_default(),
        &games_per_world,
    ))
}
```

For `update_world_handler` (line 187), replace:
```rust
Ok(()) => ok_refresh(),
```
with the same pattern. Additionally, `ctx` is consumed by `update_world` on line 185 — change `update_world(ctx, ...)` to `update_world(ctx.clone(), ...)` so `ctx` remains for `list_games`.
```rust
Ok(()) => {
    let worlds = state.application_service.list_worlds(ctx.clone());
    let games = state.application_service.list_games(ctx).unwrap_or_default();
    let mut games_per_world: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for game in &games {
        *games_per_world.entry(game.world_key.clone()).or_insert(0) += 1;
    }
    ok(render_worlds_panel(
        &worlds.unwrap_or_default(),
        &games_per_world,
    ))
}
```

**Reuse:** `render_worlds_panel` at `chronicler_engine/src/server/worlds_fragment/fragments.rs:14-33` and the exact computation from `list_worlds_fragment` at `handlers.rs:67-81` (there is no `count_games_per_world()` method — `games_per_world` is computed manually via `list_games` + loop).

**Verify:** `render_worlds_panel` is already imported in handlers.rs via `use super::fragments::{render_world_edit_form, render_worlds_panel};` (line 12). No new import needed.

**Error path unchanged.** Only the `Ok` branch changes. Error branches still return `bad_request()` / `internal_error()`.

**The form already has `hx-target=".worlds-panel" hx-swap="outerHTML"`** (set in the template, `WorldsPanelTemplate` wraps content in `<div class="worlds-panel">`). Since the submit/response now returns that same `<div class="worlds-panel">` HTML, HTMX replaces the form with the worlds list just like Cancel. No template changes needed.

**`ctx` ownership:** In `update_world_handler`, `ctx` is moved into `update_world()` at line 185. The final `ctx` on that line must become `ctx.clone()` so `ctx` survives for the `list_worlds`/`list_games` calls. In `create_world_handler`, `ctx.clone()` is already used on line 103, so just add another clone for the new calls.

### Step 5 — Stage deleted plan files

Two files were deleted from the working tree but not staged:

- `chronicler_engine/docs/plans/llm-infrastructure-improvements.md`
- `chronicler_engine/docs/plans/trigger-identity-uuid-plan.md`

Stage with `git rm` (or `git add` on the deletions). This is a commit-prep step, not a code change.

**Also archive** the active plan `docs/plans/games-tab-restructure-plan.md` → `docs/plans/archived/games-tab-restructure-plan.md` (it's been fully implemented; the archived copy already exists but the active one is still in place). Delete the active plan and ensure the archived copy stays.

## Critical files & anchors

| File | Anchor | Why |
|------|--------|-----|
| `chronicler_engine/assets/styles.css:65-78` | Global `.btn-primary` | Padding/font-size change target (Step 2) |
| `chronicler_engine/assets/styles.css:1621-1638` | Scoped `.new-game-form .btn-primary` | Deletion target (Step 2) |
| `chronicler_engine/assets/styles.css:1462-1640` | Games panel CSS block | Extraction target to games.css (Step 3) |
| `chronicler_engine/src/server/worlds_fragment/handlers.rs:105,187` | `ok_refresh()` calls | Replace with `ok(render_worlds_panel(...))` (Step 4) |
| `chronicler_engine/src/server/worlds_fragment/handlers.rs:61-85` | `list_worlds_fragment` | Reference pattern for computing `games_per_world` from `list_games` (Step 4) |

## Verification

1. **Build & test:** `cd chronicler_engine && python build.py` — all 1218+ tests must pass, no clippy warnings.
2. **Rename validation:** `grep -r "save-load" chronicler_engine/src/ chronicler_engine/assets/` should return zero hits in active code (archived docs may still match).
3. **CSS extraction:** `chronicler_engine/assets/games.css` exists and is referenced in `index.html`. `styles.css` no longer contains `.games-panel` or `#games-tab` rules.
4. **Worlds inline submit:** Start the server, create a world, click Edit → change name → Submit. Page should NOT do a full reload; the form should swap back to the worlds list smoothly (same as Cancel). Verify in browser DevTools Network tab: POST `/worlds` or `/worlds/{key}` should return HTML (not HX-Refresh header).
5. **Button sizing:** Visual check that `.btn-primary` buttons in the Games tab, Settings, Prompt Presets, and TextCheckPreview all render correctly at the new padding/font-size.

## Assumptions & contingencies

- **`ctx` ownership in `update_world_handler`:** If `ctx` is consumed by the `update_world` call, clone it before the call with `ctx.clone()`. The `create_world_handler` already does this (`ctx.clone()` on line 103). If the `update_world_handler` doesn't already clone, add it — `Arc` clones are cheap. **Fallback:** if `list_worlds` or `count_games_per_world` isn't available from the handler's scope, revert both handlers to `ok_refresh()` and file a follow-up.
- **`#command-form button` is independent:** Verified that the action-bar Send button uses `#command-form button` styling (line 506) with its own gradient, padding, and font-size — it does not use `.btn-primary`. The global `.btn-primary` change will NOT affect it. No fallback needed.
- **`games.css` cache busting:** New file gets `?v=1`. Bump `worlds.css` from `v=3` → `v=4` since it's modified in this commit.
