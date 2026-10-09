# Collapse the Prompt Presets add forms

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

The Prompt Presets panel always shows an empty Add form under each category. Each form is taller than the presets above it, and the panel is 3609px tall. Hide each Add form behind an "Add preset" button that opens it.

## Context

- Finding 4.5. Screenshots 12, 13.
- The scroll-container and width problems (4.6, 4.7) belong to [Decide the panel layout convention and supported viewports](19-decide-layout-convention.md). Do not fix them here.

## Done when

- The Add forms are closed by default and open on request.
- Tests are placed by `tests/STRATEGY.md`. Existing prompt-preset browser tests (`browser_prompt_presets.md` 28.x) still pass or are updated.
- `python build.py` is green. Commit after user approval.

## Answer

Each category's Add form is wrapped in a native `<details class="preset-add">` with a `<summary class="btn-cyan preset-add-toggle">` toggle, closed by default. No JavaScript: `assets/index.html` is untouched. After a successful add the panel re-renders and the form is closed with no stale input. The toggle has the documented panel-control focus style (`ui_design.md`).

**Scope kept.** The failed-add error path is byte-identical to before: `save_preset_handler` and its tests show no change. Where failures display is [Stop server errors from wiping whole panels](25-stop-errors-wiping-panels.md), blocked by [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md). The scroll and width problems (4.6, 4.7) belong to ticket 19.

**Tests (tier 1).** `test_prompt_presets_add_forms_collapsed_by_default` — new scenario 21.28 (`docs/specs/prompt_presets.md`), asserting each category's Add form is collapsed until opened. 21.1's toggle assertions updated (the stable `preset-add-toggle` hook plus toggle text, not a presentation class). Stub fixture mirrors the template. The existing tier-3 browser test 28.1 passes unchanged.

**Gate:** worktree on `f1521a0f`: `nextest: 1642 passed, 0 failed, 2 skipped`, browser 26 passed (`build_20260930_210811.log`).

**Code review** (`/code-review`, verdict ISSUES → fixed): missing focus-visible style, spec 21.28 written in HTML-mechanism terms, and the tier-1 test pinning the `.btn-cyan` class. All three fixed. Side finding: a failed preset **edit** (`update_preset_handler`) still replaces the card with a bare error span and loses in-progress edits — noted on ticket 25's scope.

