# Keep DOM state and focus through in-place swaps

Type: task (AFK)
Status: open
Blocked by: —

## Question

Four pieces of work, one root cause: an `hx-swap` replaces the node holding
focus, selection or the poll trigger, and the DOM state is lost. Do them
together because they edit the same file (`assets/index.html`) and the same
story-log spec and test files, and because a session that touches one of them
collides with the others.

### 1. Morph the story-log poll (finding 2.4, K1, K7)

The 2s poll's `innerHTML` swap destroys DOM state. Replace it with a morph, so
an idle poll touches no nodes and a changed poll touches only the nodes that
changed. Decision: [Decide how the story-log poll keeps DOM state](10-decide-story-log-poll-swap.md),
option B — client-side morph.

- Vendor idiomorph's htmx extension at `assets/idiomorph-ext.min.js`
  (`https://unpkg.com/idiomorph/dist/idiomorph-ext.min.js` — the
  `idiomorph-ext` build bundles the extension). 0BSD. The existing `/assets`
  static route and the stub server's `ServeDir` both serve it with no wiring
  change.
- `assets/index.html`: `<body hx-ext="morph">`.
- `assets/index.html`: `#story-log` `hx-swap="innerHTML"` →
  `hx-swap="morph:innerHTML"` (morphs the children, leaves the container itself
  alone).
- `assets/index.html`: add `tabindex="0"` and an accessible name (`aria-label`)
  to `#story-log`, so a keyboard user can focus it and arrow-scroll (K7). Do
  **not** add `role="log"` — it is an ARIA live region and would announce the
  narrative on every poll; announcing is [Announce dynamic state changes to
  assistive technology](47-announce-state-changes-to-at.md). If ticket 46 is in
  flight, coordinate the accessible name.
- `NarrativeLogTemplate` (`src/adapters/driving/http/templates.rs`): add
  `id="entry-{{ entry.id }}"` to each `.log-entry`. Morph matches on `id`;
  `data-id` will not do, and the 50-entry cap removes the oldest entry every
  turn, so removal matching matters.
- Keep `pausePolling`/`resumePolling` (the edit-mode pause). Morph does not
  cover a stale server view overwriting an in-progress edit.
- Scope: the story log only. Leave `#options-dock`, `#status-display`,
  `#visual-sidebar` and `#llm-messages-panel` on `innerHTML`.
- `switchSwipe`'s direct `innerHTML` write stays. A following morph onto equal
  content is a no-op. No hash, no bookkeeping.
- Watch: the `MAX_LOG_DISPLAY = 50` cap makes removal routine, not an edge case.
  An edit or swipe that changes an entry still updates that entry's nodes, so a
  selection inside the changed entry may collapse; a selection elsewhere must
  survive.

Tests — **tier 2** (`tests/browser/stub/story_log.rs`), per `tests/STRATEGY.md`.
New scenarios in `docs/specs/browser_story_log.md`, beside Scenario 30.3:

- a text selection in the log survives at least two poll cycles (finding 2.4);
- focus inside a `.log-entry` survives at least two poll cycles (K1);
- `#story-log` takes focus and arrow keys scroll it (K7).

The stub fixture `tests/test_utils/stub_fixtures/story_log.html` gains `id`
attributes on its entries, so the fixture keeps the structural elements the
shell addresses (the stub tier's accepted tax). Exercise morph's removal path if
cheap: serve a second fixture without the oldest entry.

### 2. Lock the other Edit buttons while an edit is open

Implement the decision in [Stop editing a second entry from freezing the
story-log poll](38-edit-another-entry-freezes-poll.md). While an edit is open on
a story-log entry, disable every other entry's `.edit-btn`. Also fix the root
cause, so a second `showEditForm` call cannot corrupt the poll:

- `pausePolling` saves `hx-trigger` only when `originalTrigger` is null. A
  second pause keeps `load, every 2s`.
- `showEditForm` returns early when `editState` is set.

Detail:

- Edit state lives in `editState` in `assets/index.html`. The lock and its
  release belong beside `showEditForm` and `revertEdit`.
- Release the lock on cancel, Escape, and a failed save. A successful save keeps
  the lock until the resumed poll re-renders the log (≤2 s); releasing it earlier
  lets a second editor open on an entry that still shows a textarea.
- The disabled style follows the existing pattern in
  `docs/diataxis/reference/frontend/ui_design.md` (`opacity: 0.5;
  cursor: not-allowed`). `.action-btn:hover` needs a `:not(:disabled)` guard,
  like `.option-btn` and `.swipe-btn`.
- Spec: `docs/specs/browser_story_log.md` is silent on a second Edit click. Add
  a scenario in the register of 30.9. Scenarios 30.1–30.10 stay.
- Test: tier 2 stub browser, `tests/browser/stub/story_log.rs`, beside the other
  edit tests. Follow `tests/STRATEGY.md`.
- Ticket 11 locks the edited entry's own controls; this extends the same pattern
  to the rest of the log.
- Source of the finding: the ticket 11 implementer, recorded in ticket 36.

### 3. Restore keyboard focus after in-place swaps (findings K3, K6)

Keyboard focus falls to `<body>` whenever a swap replaces the element holding
it. Reproduce: Games tab, focus a posture or preset select, press ArrowDown —
`#game-posture-controls` is swapped with `hx-swap="outerHTML"` and focus is lost
(the select has no `id`, so htmx's id-based focus restore does nothing);
likewise Worlds Edit and Cancel, Prompt Presets View/Close/Cancel/Set
Active/Duplicate, and cancelling edit mode with Escape. A keyboard user must
re-Tab from the top of the page after each. Choose one mechanism — a stable `id`
on each replaced control so htmx's built-in focus restore works, or moving focus
to the replacement (or a sensible target) after each swap — and apply it
consistently.

- Findings K3 and K6 (P2/P2) of ticket 07. Evidence
  `tmp/ui-review/A1-games-posture-focus.png`, `A2-worlds-edit-focus.png`,
  `A3-worlds-cancel-focus.png`, `A6-presets-view-focus.png` (local only); focus
  trace `focus-mode {value: novel}` → `ArrowDown` → `active: BODY`;
  `after-escape {focus: BODY}`.
- The preset paths (Close, Cancel, Set Active, Duplicate) are recorded as
  `[inferred]` from the shared `hx-target="closest .preset-card"` pattern, not
  exercised — confirm them while fixing.
- The text-check preview's own open/Cancel/confirm focus path is ticket 42's;
  this ticket owns the other swap sites. Pick one mechanism across both.
- Do not break the two paths that already keep focus: command submit keeps focus
  in the input via `HX-Retarget: #status-display`, and edit mode autofocuses
  `#edit-textarea`.

### 4. Delete the dead `htmx:refresh` calls and fix the docs

Six `htmx.trigger(..., "htmx:refresh")` calls in `assets/index.html` fire an
event nothing listens for. Delete them, or give the refresh a real event name,
then fix the docs that describe them as working.

- htmx 1.9.10 has no `htmx:refresh` event and no `refresh` trigger. The bundle's
  only two `refresh` strings are `refreshOnHistoryMiss`. `git grep
  "htmx:refresh"` finds no listener and no test. The string first appears in
  `da46d199`, when the page loaded htmx 1.9.10 from unpkg — so it has never
  worked here.
- Call sites: `assets/index.html:341,362,394` (`#story-log`), `:421`
  (`#visual-sidebar`), `:422` (`#header`), `:740` (the LLM panel).
- A seventh dead event sits beside them: the
  `document.body.addEventListener("action-area-refresh", ...)` at
  `assets/index.html:778` has no dispatcher either (noted by ticket 42). Fold it
  into the same cleanup.
- Do not confuse it with `HX-Refresh`, the response header in
  `src/adapters/driving/http/utils/response.rs:22`. That one is real and tested
  (`tests/http/games_create.rs`, `games_switch.rs`,
  `tests/http/requires_migration/fragment.rs`, `worlds_fragment_handlers.rs`).
  This ticket does not touch it.
- Docs that describe the dead event as working:
  `docs/diataxis/reference/frontend/dashboard.md:96,118`.
- Source: the implementer's report on ticket 27, recorded in
  [Decide how the story-log poll keeps DOM state](10-decide-story-log-poll-swap.md).
- After the morph ships, the dead calls are harmless: a morph onto equal content
  is a no-op, and the 2s poll covers the update within a cycle. Deleting is
  therefore acceptable.
- If an immediate refresh is wanted instead, name a real event in each
  container's `hx-trigger` (for example `load, every 2s, panel-refresh`) and fire
  that name. Do **not** fire `load`: htmx 1.9.10 handles `load` as a one-shot
  init trigger with an internal `loaded` guard, so re-firing it reaches nothing.

## Done when

- An idle story-log poll touches no nodes; a text selection and focus inside a
  `.log-entry` survive at least two poll cycles; `#story-log` takes focus and
  arrow keys scroll it.
- While an edit is open, every other `.edit-btn` is disabled. Cancel, Escape, and
  a failed save re-enable them in the same tick. A successful save re-enables
  them when the poll re-renders the log. `pausePolling` cannot overwrite the
  saved trigger, and `showEditForm` does nothing while an edit is open. The new
  spec scenario is added and a tier 2 test drives it and fails on the pre-fix
  shell.
- Each listed swap either keeps focus on the equivalent control or moves it
  somewhere deliberate, and Tab from there continues forward rather than
  restarting at the top. A browser test covers at least the posture-select and
  Worlds-Cancel paths.
- No dead `htmx:refresh` (or `action-area-refresh`) remains, and
  `docs/diataxis/reference/frontend/dashboard.md` matches the shell.
- Test placement follows `tests/STRATEGY.md`, and the answer names the tier.
  Write new spec Givens and Thens as domain outcomes per ticket 31, not field
  names.
- `python build.py` is green, the user reviews the diff, and the work is
  committed through `/commit-and-push`.
