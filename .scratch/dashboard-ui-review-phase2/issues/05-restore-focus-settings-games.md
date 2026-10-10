# Restore focus after Settings, Games and story-log delete swaps

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Keyboard focus falls to `<body>` after swaps that the focus restore from [Keep DOM state and focus through in-place swaps](../../dashboard-ui-review/issues/_resolved/65-keep-dom-state-through-swaps.md) does not cover. Extend the same mechanism to them.

## Items

From the [re-review](../../dashboard-ui-review/re-review-2026-10-09.md):

- **R10 (P2)** With real focus on the control, these leave focus on `<body>`: Settings connection Edit (→ form page), Cancel / back link (→ list), and Games saved-game Delete. `FOCUS_SWAP_TARGETS` in `assets/dashboard-panels.js` lists only `.worlds-panel, .prompt-presets-panel, .preset-card`. The Settings panel was rewritten by [Split Settings into Connections and Text Check sub-tabs](../../dashboard-ui-review/issues/68-split-settings-sub-tabs.md) after the restore shipped; the Games panel was never listed. Check every Settings and Games swap (role select, Test, Delete refusal, Add, rename, Switch).
- **R7 (P3)** Deleting the last story-log entry removes the focused Delete button and leaves focus on `<body>`. Move focus to a sensible target, such as the new last entry or the command input.

## Done when

- Each listed path keeps focus on a sensible control; stub-tier browser tests cover a Settings path and the Games delete.
- `python build.py` is green. Commit after user approval.

## Answer

Extended the swap focus restore to the Settings and Games panels, and gave the
story-log delete its own landing spot. Test tier: **stub browser (tier 2)** for
all three tests — focus after a swap is client behaviour, so faking the server
does not change it (`tests/STRATEGY.md`, placement rule step 2).

### R10 — Settings and Games swaps

`assets/dashboard-panels.js`:

- `FOCUS_SWAP_TARGETS` now also lists `.settings-panel` and `.games-panel`, so
  every whole-panel swap (Edit, Add, the back link/Cancel, Save, a refused
  delete) lands focus inside the panel instead of on `<body>`.
- The landing spot is the panel's first form field, falling back to its first
  focusable (`focusInto`): Edit lands in `#conn_name` rather than on the form
  page's back link, the back link lands on `#role-select-narrator`. A candidate
  now counts only once `document.activeElement` really is that node — `focus()`
  is a no-op on a hidden control (inside a closed disclosure) or a detached one,
  which the old loop mistook for success.
- A row that a swap removes (`hx-target="closest .game-item" hx-swap="outerHTML"`)
  fires no `afterSettle` at all when it was the last child, so its landing spot
  is planned in `htmx:beforeSwap` and applied on `htmx:afterRequest` (htmx
  re-fires that event on the nearest surviving ancestor of a request element the
  swap removed): the row that takes its place, else the row before it, else the
  panel's first field.

Checked, unchanged:

- **Role select** — keeps focus through htmx's own id restore (the select
  survives the panel swap with the same id). Now pinned by a test leg.
- **Test** — its swap target is the sibling result slot, so the button is never
  replaced.
- **Rename, Switch** — both answer `HX-Refresh`: a full page load, not a swap, so
  no swap-level restore applies (their landing tab is R14 / ticket 06).
- **Posture selects** — stable ids, already pinned by `browser_games.md` 27.2.
- **Delete refusal** — takes the same swap shape as the back link (the panel is
  replaced) and lands on the panel's first field, which is the shipped Worlds
  refusal convention. A *failed* delete (4xx/5xx) does not swap, so the focused
  control stays put.
- Found, not fixed: the Text Check card's "Check before sending to LLM" checkbox
  has no `id`, so toggling it (`hx-swap="outerHTML"` on `#text-check-card`) still
  drops focus to `<body>`. See the follow-up below.

### R7 — story-log delete

`assets/dashboard-story-log.js`: `deleteMessage` now returns the fragment-swap
promise and focuses, once it settles, the entry that became last (its Edit
control) — the same "first control of the row that took its place" rule the
Games row uses — falling back to `#command-input` when no entry is left. The
edit sits above `deleteMessage`'s tail so ticket 03's append at the end of the
file merges cleanly.

### Tests (tier 2, `tests/browser/stub/`)

- `settings.rs::test_settings_panel_swaps_keep_keyboard_focus_in_the_panel`
  (`browser_settings.md` 40.3): Edit → `#conn_name`; back link →
  `#role-select-narrator`; changing the Quantifier select →
  `#role-select-quantifier` (only htmx's id restore can produce that one).
- `games.rs::test_deleting_a_saved_game_keeps_keyboard_focus_in_the_panel`
  (`browser_games.md` 27.3, new file): first delete → the surviving row's Switch
  control; last delete → focus inside `.games-panel`.
- `story_log_edit.rs::test_successful_delete_keeps_focus_in_the_story_log`
  (`browser_story_log.md` 30.22 (renumbered from 30.20 at integration; 03 took 30.20–30.21)).

All three fail on the pre-fix shell: verified by running them with `HEAD`'s
`assets/dashboard-*.js` restored in the worktree. (The Settings test's third leg
passes either way on purpose: it pins htmx's own id restore for the role select,
which is one of the paths this ticket was asked to check.)

Stub support added: the games fixture shows two saved games; `/games/:id/delete`
answers an empty 200; `/connections/set-quantifier` re-renders the panel;
`StubSaveHandle::set_delete_succeeds` makes `/history/delete` succeed and drop
the served log's last entry (the default stays a 500, so 30.18 still drives the
failure path).

Specs: `browser_settings.md` 40.3, `browser_games.md` 27.3,
`browser_story_log.md` 30.22 (renumbered from 30.20 at integration; 03 took 30.20–30.21). Docs: `dashboard.md` Delete Flow, Saved Games and
Settings Tab now state the landing rule.

### Validation

`python build.py` green: 1595 integration, 72 browser (69 + 3), 162 guardrails,
1 architecture, 2 skipped.

### Follow-ups (non-blocking)

- **Text Check card checkbox.** `enable_auto_check` has no `id`, so htmx's
  restore cannot find it after the card swap and focus falls to `<body>`. Fix:
  add `id="enable_auto_check"` to the checkbox in `TextCheckCardTemplate`
  (`src/adapters/driving/http/settings/templates/settings.rs`). Left out of this
  ticket: it is not one of R10's listed paths, and it edits a template ticket 02
  also touches. Belongs in ticket 06.
- The story-log delete's `.catch` now also covers a failed fragment reload and
  reports the existing "Failed to delete the message." line; the log is stale in
  that case, so the wording still reads true to the player.
