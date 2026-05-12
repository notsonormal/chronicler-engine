# Plan: Fix Story Log Button Visibility & Text Bolding

## Problem

1. Delete button shows on every message (should only show on last message).
2. Delete button shows when there's only one message (need at least one message in the game).
3. Retry button shows on the first/only message (it's not generated, nothing to retry).
4. Text bolding is inconsistent — location entries render all text bold.

## Root Causes

- `templates.rs:81`: Delete button is rendered unconditionally. Retry button uses `loop.last` but when there's only one entry, `loop.last == loop.first`.
- `styles.css:123`: `.location { font-weight: bold; }` is a global CSS selector. Log entries with location headers get `class="... location"`, making ALL text inside the entry bold. The `.location-header` span already has its own `font-weight: bold` styling.

## Files to Change

- `src/server/templates.rs` — Add `entries|length > 1` guard to delete and retry buttons.
- `assets/styles.css` — Remove `font-weight: bold` from `.location` rule.
- `src/server/templates_tests.rs` — Add second entries to tests asserting delete/retry presence.
- `tests/browser/editing.rs` — Send an action before testing delete/retry button presence.
- `docs/architecture/system.md` — Update UI Integration section.
- `docs/system/dashboard.md` — Update button descriptions.
- `docs/system/ui_design.md` — Update `.location` and `.delete-btn` descriptions.
- `docs/CHANGELOG.md` — Record fixes.
