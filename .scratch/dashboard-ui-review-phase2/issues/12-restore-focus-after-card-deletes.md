# Restore focus after preset and world deletes

Type: task (AFK)
Status: open
Blocked by: —

## Question

Deleting a prompt preset or a world drops keyboard focus to `<body>`. Extend the removed-row landing rule to these cards.

## Context

- Finding **S4 (P2)** of the [2026-10-11 re-review](../re-review-2026-10-11.md), same class as R10.
- Both Delete controls use `hx-target="closest .preset-card"` / `"closest .world-item"` with `hx-swap="outerHTML swap:0.3s"` (`src/adapters/driving/http/builders/presets.rs`, `src/adapters/driving/http/worlds/templates/worlds.rs`).
- [Restore focus after Settings, Games and story-log delete swaps](05-restore-focus-settings-games.md) added the removed-row rule for `.game-item` only (`FOCUS_ROW` and the fallback to `.games-panel` in `assets/dashboard-panels.js`). Generalise it: the next card, else the previous card, else the card's own panel. Check the 0.3s swap delay against the `htmx:afterRequest` timing.

## Done when

- Deleting a preset card and a world card lands focus on the next card, the previous card, or the panel.
- A stub-tier browser test covers one of them (extend `test_deleting_a_saved_game_keeps_keyboard_focus_in_the_panel`'s pattern). Name the tier.
- `python build.py` is green. Commit after user approval.
