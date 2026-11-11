# Restore focus after preset and world deletes

Type: task (AFK)
Status: resolved
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

## Answer

Built. The removed-row landing rule covers every card a panel deletes, and its
timing was wrong for the 0.3s deletes — both were checked against the running
shell.

**The rule, generalised.** `FOCUS_ROW` in `assets/dashboard-panels.js` is now
`.game-item, .preset-card, .world-item`, and the landing spot is the next card in
the row's container, else the previous card, else the row's own panel
(`FOCUS_ROW_PANEL`: `.games-panel, .prompt-presets-panel, .worlds-panel`). The
neighbour search takes the next *card*, not the next element: a preset section's
add form sits between its last card and the next section, and its fields live in a
closed disclosure.

**The timing assumption did not hold.** Ticket 05 applied the landing spot on
`htmx:afterRequest`, guarded by `document.contains(removed.focused)`. Reading htmx
1.9 (`assets/htmx.min.js`) shows why that only ever worked for
`hx-swap="outerHTML"`: `htmx:afterRequest` is triggered in `xhr.onload` after
`handleAjaxResponse` returns, while a `swap:0.3s` delay removes the row in a
`setTimeout` scheduled inside it. So for both new deletes the row was still in the
document when afterRequest fired, the guard bailed, and focus fell to `<body>`
300 ms later. `htmx:afterSettle` cannot cover it either: htmx fires it on the
elements a swap *inserted*, and an empty delete body inserts nothing. Both deletes
carry `swap:0.3s` (`builders/presets.rs`, `worlds/templates/worlds.rs`).

**The fix.** Focus is claimed in `htmx:beforeSwap`, before the row goes, so the
landing spot survives either timing. The claim fires only when the row itself is the
swap target, `shouldSwap` is true, and the response inserts no panel or card of its
own (`insertsFocusTarget`): a replacement still takes focus from
`htmx:afterSettle`, as before, which keeps the edit form's first field on the
landing ticket 05 gave it. The rename, switch and reset controls inside a game row
(`hx-swap="none"`, row left in place) never match, and a failed delete keeps focus
on the control that asked for it. The `htmx:afterRequest` listener and its
`removedRow` plan are gone.

**Tests, by tier.**

- **Stub browser (tier 2)** — `tests/browser/stub/prompt_presets.rs`
  (`run_prompt_preset_checks`) and `tests/browser/stub/worlds.rs`
  (`run_world_checks`), one runner per spec as `tests/STRATEGY.md` asks:
  `check_deleting_a_preset_card_keeps_keyboard_focus_on_a_card`
  (`browser_prompt_presets.md` 28.3) deletes the first of two non-default cards and
  asserts the next card's Edit control, then deletes the last and asserts the
  previous card's View control, and
  `check_opening_a_preset_editor_moves_focus_straight_into_the_form` (28.4) asserts
  the editor's single landing, which guards the double move below.
  `check_deleting_a_world_keeps_keyboard_focus_in_the_panel`
  (`browser_worlds.md` 29.4) deletes the first world and asserts the surviving
  world's Edit control, then deletes the last and asserts focus inside
  `.worlds-panel`. Both delete checks wait for the removal itself first, because the
  landing spot is claimed before the `swap:0.3s` swap lands. Tier 2 by the placement
  rule: faking the server does not change where focus lands after a swap.
- Stub support: the presets fixture gained a second non-default card; the stub's
  `/prompt-presets/:id/delete` and `/worlds/:key/delete` answer the engine's empty
  200 when `StubRowDeleteHandle::set_row_deletes_succeed(true)` arms them, and keep
  their refusal/failure by default so `form_failures` 16.32 and `failure_display`
  16.34 keep driving those paths. 16.34 also asserts now that a failed row action
  leaves focus on the control that asked for it — the other half of the
  `shouldSwap` gate.
- Specs: `browser_prompt_presets.md` 28.3 and 28.4, `browser_worlds.md` 29.4. No
  `docs/diataxis/` change: the dashboard reference doc states invariants, and this
  fine-grained post-swap behaviour belongs to the scenario.

**Evidence the timing claim is real.** With the minimal generalisation the ticket
asks for — `.preset-card`/`.world-item` added to `FOCUS_ROW`, the panel fallback,
the afterRequest listener kept — both new checks fail
(`logs/build_20261011_154726.log`): "deleting a preset card must move focus to the
next card's control" and "deleting a world must move focus to the surviving world's
Edit control". On the base `a4ac8814` shell both fail the same way
(`logs/build_20261011_154750.log`), and ticket 05's games check is green on this
tree (`logs/build_20261011_164738.log`, the clean-tree full gate).

**Review (`/code-review`, two axes).** Fixed: the neighbour search walked
`nextElementSibling` and could stop on the preset add form; it now takes the next
card among the row's siblings. Four copies of the "wait for the count" poll became
`wait_for_count`. The preset check scopes its baseline to the section its two
deletable cards share. Rejected: preempting only for delayed swaps (parsing htmx's
swap spec duplicates its parser). The skill's parallel `reviewer` agents were not
in this session's roster (only `scout`), so both axes ran inline.

**Independent reviews (P3s).** A `reviewer` agent verified `1e352add` and accepted
it; its five P3s are resolved as follows. A second `reviewer` then verified
`d548f5d2` and accepted it too; its three P3s are the corrected `wait_for_count`
evidence above, the lasting guard for the double move, and the failed preset GET
now in the follow-up list.

*Preset Edit/View/Close/Cancel/Save moved focus twice — real, introduced here.*
Those controls also target `closest .preset-card`, so the preemption fired before
the swap and `afterSettle` then moved focus into the replacement. Evidence: a
throwaway stub check recorded every `focusin` plus the htmx swap events with an
identifier for each element. On the base `a4ac8814` shell the preset Edit sequence
after the click is `button.btn-cyan[/fragment/prompt-presets/custom_ref/edit]` then
`input#edit-name-custom_ref`; on `1e352add` it is the clicked button, then
`button.btn-cyan[/fragment/prompt-presets/custom_two/edit]` (the neighbour), then the
form field. Fixed by skipping the preemption when the response itself inserts a
`FOCUS_SWAP_TARGET` — a card or panel of its own takes focus from `afterSettle`, so
only a row left without one needs its landing spot claimed early:
`insertsFocusTarget(evt.detail.serverResponse)` parses the response into a
`<template>` and tests the same selector. The fix also removed `preemptedFocus` and
restored the original `afterSettle` guard. With it, the Edit log is byte-for-byte
what the base `a4ac8814` shell produces, and the delete landings are unchanged. The
reviewer's "require an empty body" suggestion was rejected: the games last delete
answers
`.games-empty` (non-empty) and the `WorldHasGames` refusal answers a 200 error row,
which is not a `FOCUS_SWAP_TARGET`, so both would lose their landing.

*Module placement.* `tests/browser/stub/card_deletes.rs` covered two specs; it is
now `stub/prompt_presets.rs` and `stub/worlds.rs`, one runner each, and the shared
`wait_for_count` and `accept_delete_confirmation` live in `stub/support.rs`. The
worlds check also asserts the fixture's two worlds rather than "more than one",
which is what the scenario's Given states.

*The double move has a lasting guard.* `stub/prompt_presets.rs` gained
`check_opening_a_preset_editor_moves_focus_straight_into_the_form`
(`browser_prompt_presets.md` 28.4, a new scenario): a page-side `focusin` recorder
plus `wait_until_focused` on the new card's Name field, then an exact assertion on
the recorded sequence. `focusin` is synchronous, so the list is complete once the
final landing is observed — no sleep and no race. Against `1e352add`'s shell the
assertion fails and names the neighbouring card:
`["…/custom_ref/edit", "…/custom_two/edit", "edit-name-custom_ref"]`
(`logs/build_20261011_164055.log`); with `insertsFocusTarget` deleted outright the
check fails earlier, on the landing itself (`logs/build_20261011_164036.log`).

*Scenario 16.34.* The scenario now states that each failed row action leaves focus
on the control that asked for it (both the world Delete and the game Reset), and
the test's comment says "a failed row action" rather than "a failed delete".

*Vacuous pass.* `wait_for_count` reads the count with `expect`, so a Playwright
query error panics — caught per check by `StubRunner` and reported as a failure —
instead of counting as zero and letting "none left" pass. Verified by handing the
wait helper an invalid selector: it fails loudly, in the helper itself
(`logs/build_20261011_163049.log`, `panicked at tests/browser/stub/support.rs:217:
count the elements the selector names: ProtocolError(…)`).

**Gate.** `python build.py` green on this tree before the commit
(`logs/build_20261011_164352.log`): every step OK, clippy clean, integration
`1610 passed, 0 failed, 2 skipped`, browser `29 passed, 0 failed`, guardrails
`162 passed`, architecture `1 passed`. The clean tree that carries this Answer was
gated again after the commit — a citation cannot name a run that has not happened
yet — and `logs/build_history.txt` records that run as a clean tree, so no
dirty-tree digest has to be reconstructed. The earlier commit's run
(`logs/build_20261011_155959.log`) had 28 browser tests before the module split.
Split checks re-verified against the base `a4ac8814` shell: both fail
(`logs/build_20261011_161732.log` for the preset, `logs/build_20261011_161741.log`
for the world), and ticket 05's games check is green in the clean-tree gate above.
(Filing the follow-ups under ticket 13 is the Manager's; this Answer reports them.)

**Follow-ups for ticket 13.**

1. A world delete the engine refuses with `WorldHasGames` answers a 200 that
   replaces the row with an error row: focus now lands on the neighbouring row or
   the panel rather than `<body>`, but not on the error itself.
2. The same for a preset card whose GET fails: `require_preset!` answers a 200
   `div.error-message` with no `FOCUS_SWAP_TARGETS` element, so focus moves to the
   neighbouring card before the error replaces the card, where the base `a4ac8814`
   shell dropped it to `<body>`.
3. `focusedBeforeSwap` is one global (`assets/dashboard-panels.js`) that every
   `htmx:beforeSwap` overwrites. htmx 1.9.10 waits ~20 ms (`defaultSettleDelay`)
   between the swap and `afterSettle`, and the page's 2–5 s polls
   (`assets/index.html`) can answer inside that window, replacing the saved element
   with `<body>`. The swap's `afterSettle` then returns early and a focus check that
   depends on it waits out its timeout: a few percent per click by estimate, never
   seen in 8 consecutive runs of the 28.4 check, and already present in the base
   `a4ac8814` code rather than introduced here. It affects every
   `afterSettle`-dependent focus check.
4. `scripts/prepare_review_bundle.py` files `assets/` under `docs.patch`, so a
   JS-only change reads as a docs change in the bundle's README.

Committed on `phase2-12-focus-after-card-deletes`; the Manager merges it.
