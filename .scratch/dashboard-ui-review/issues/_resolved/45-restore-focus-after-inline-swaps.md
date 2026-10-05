# Restore keyboard focus after in-place htmx swaps

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Keyboard focus falls to `<body>` whenever a swap replaces the element holding it. Reproduce: Games tab, focus a posture or preset select, press ArrowDown — `#game-posture-controls` is swapped with `hx-swap="outerHTML"` and focus is lost (the select has no `id`, so htmx's id-based focus restore does nothing); likewise Worlds Edit and Cancel, Prompt Presets View/Close/Cancel/Set Active/Duplicate, and cancelling edit mode with Escape. A keyboard user must re-Tab from the top of the page after each. Choose one mechanism — a stable `id` on each replaced control so htmx's built-in focus restore works, or moving focus to the replacement (or a sensible target) after each swap — and apply it consistently.

## Context

- Findings K3 and K6 (P2/P2) of [Review keyboard use and screen-reader output](07-review-keyboard-screen-reader.md). Evidence `tmp/ui-review/A1-games-posture-focus.png`, `A2-worlds-edit-focus.png`, `A3-worlds-cancel-focus.png`, `A6-presets-view-focus.png` (local only); focus trace `focus-mode {value: novel}` → `ArrowDown` → `active: BODY`; `after-escape {focus: BODY}`.
- The preset paths (Close, Cancel, Set Active, Duplicate) are recorded as `[inferred]` from the shared `hx-target="closest .preset-card"` pattern, not exercised — confirm them while fixing.
- The text-check preview's own open/Cancel/confirm focus path is [Rework the action area so a text check cannot strand it](42-rework-the-action-area.md); this ticket owns the other swap sites. Pick one mechanism across both.
- Do not break the two paths that already keep focus: command submit keeps focus in the input via `HX-Retarget: #status-display`, and edit mode autofocuses `#edit-textarea`.

## Done when

- Each listed swap either keeps focus on the equivalent control or moves it somewhere deliberate, and Tab from there continues forward rather than restarting at the top.
- A browser test covers at least the posture-select and Worlds-Cancel paths.
- Test placement follows `tests/STRATEGY.md`, and the answer names the tier. Until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves, write new spec Givens and Thens in `CONTEXT.md` terms rather than field names.
- `python build.py` is green, the user reviews the diff, and the work is committed through `/commit-and-push`.

## Answer

Absorbed into [Keep DOM state and focus through in-place swaps](65-keep-dom-state-through-swaps.md).
Focus restoration moved into the DOM-state sweep, so every swap site that loses
focus or selection is fixed with one mechanism in one pass over
`assets/index.html`. Closed as absorbed, not fixed.
