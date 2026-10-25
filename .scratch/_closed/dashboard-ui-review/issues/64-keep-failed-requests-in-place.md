# Keep failed forms and cards in place

Type: task (AFK)
Status: resolved
Blocked by: 63

## Question

One class of failure, two surfaces: a failed add, edit or refused delete must
not swap into the region it describes. Today it does, and the panel or card
disappears with only the error text left.

- **Whole panels** (finding 05.F1, P1). An invalid provider on Settings → Add
  connection replaces all of `.settings-panel`'s inner HTML (every connection
  card, the add form, Text Check); an invalid `preset_type` on Prompt Presets
  replaces `.prompt-presets-panel` with the text `Invalid preset type`. Both are
  HTTP 200 bodies, so no toast fires.
- **Entity cards** (finding 05.F2, P1). A failed connection edit replaces the
  connection's own card (`hx-target="closest .connection-edit-form"`,
  `outerHTML`), so the connection leaves the list. Deleting an active preset is
  correctly refused, but the refusal body replaces the preset card
  (`hx-target="closest .preset-card"`), so the card vanishes and only the error
  text is left.
- A failed preset **edit** (`update_preset_handler`) replaces the card with a
  bare error span and loses in-progress edits (added from ticket 17).

Both recover only on a page reload.

## Context

- Findings 05.F1 and 05.F2; screenshots 54 (empty Settings panel holding only the
  error), 51 (Prompt Presets showing only "Invalid preset type"), 56 (connection
  card replaced by `Error: Configuration error: Unknown LLM backend
  'bogus_provider'`) and 52 (bare `Preset is a mode default; change the default
  before deleting` where the card was); ticket 05 answer.
- Mechanism: handlers return `Html(render_error(..))` / `<span class='error'>`
  with status 200, and the forms target the whole panel
  (`hx-target=".settings-panel" hx-swap="innerHTML"` for connections,
  `hx-target=".prompt-presets-panel" hx-swap="outerHTML"` for presets).
- Where the message appears is decided in
  [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md);
  this ticket is the implementation for panels and cards. A failure never swaps
  into the region it describes. The message renders in an inline slot inside the
  form or card — a short user sentence, with the raw server text behind an
  anchored popover.
- [Redesign the error and health display](63-redesign-error-health-display.md)
  owns the shared short-message + anchored-popover error fragment and the
  `beforeSwap` behaviour. Consume it here rather than building a second shape.
- The same bodies are also returned for storage failures (`Save failed: ..`,
  `Load failed: ..`), so this is not only reachable through tampered form values.
- The refusal logic itself is fine and should stay: default presets and
  mode-default references must not be deleted.

## Done when

- A failed add (connection or preset), a failed connection edit, a failed preset
  edit, and a refused preset delete all leave the panel and its other controls,
  or the card, intact and usable, with the message shown nearby.
- Tests cover the failed add path and both card paths; tier by
  `tests/STRATEGY.md`.
- `python build.py` is green. Commit after user approval.

## Answer

Resolved. A failed add, a failed edit and a refused delete leave the panel or
the card in place; the message shows in an inline slot inside it.

- **Mechanism.** The listed actions answer non-2xx — 400 for a refusal, 500 for
a storage or load failure — so htmx never swaps the region the failure
describes. The shell's one `htmx:responseError` handler renders the shared
short-message + Details disclosure (`error_disclosure` / `raw_error_detail`)
into that surface's `[data-error-slot]`. `error_response` keeps its 200-fragment
contract for its live callers (`worlds.rs`, `prompt_presets.rs`), so
`create_world`'s shape is unchanged.
- **Slots.** Added to the four preset Add forms, the preset edit form and every
preset card; the connection form's slot already shipped. Refusal logic is
unchanged: default presets and mode-default references stay refused.
- **Specs and docs.** `settings.md` 20.17/20.18, `prompt_presets.md` 21.39 plus
updated statuses, `browser_dashboard.md` 16.29–16.32; `dashboard.md`,
`ui_design.md`.
- **Tests.** Tier 1 HTTP for every path (`tests/http/settings.rs`,
`tests/http/prompt_presets.rs`). Tier 2 stub-browser for the connection form
(16.29), the preset Add form (16.30) and both preset card paths (16.31, 16.32 —
added during the review fix). Unit assertions in `prompt_presets_tests.rs`.
- **Deferred, same class.** `duplicate_preset_handler`, `activate_preset_handler`
and `create_world_handler` keep their 200 failure shape; none are in this
ticket's paths. Closed later by
[71](71-close-review-findings.md): all three answer non-2xx and
`error_response` is deleted.
- **Build.** Full gate green: architecture 1, guardrails 165, integration 1567
(2 skipped), browser 68, 0 failed (`logs/build_20261007_195200.log`).
- **State.** The combined 64+69 change is committed as `6077371b` on
`dashboard-ui-issues-2`. Development commits: `6a91631a` on `wf/t64`;
`54371f99` on `wf/t69`.
