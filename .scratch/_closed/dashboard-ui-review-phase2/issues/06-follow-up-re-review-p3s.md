# Follow up on small issues found in the final re-review

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

The [re-review](../../dashboard-ui-review/re-review-2026-10-09.md) found P3 issues. Which still hold, and what fixes them? Same rules as [Follow up on small issues found during review](../../dashboard-ui-review/issues/_resolved/36-follow-up-small-review-issues.md) and its [round 2](../../dashboard-ui-review/issues/_resolved/37-follow-up-small-review-issues-2.md): check each item first and drop what no longer holds with a reason.

## Items

- **R4** The command input narrows 899 → 855px while the button reads "Generating…" (layout shift). Shot `tmp/t24/08-options.png`.
- **R5** The failure banner pushes the tab bar and the page down 31px when it appears.
- **R11** Text-check preview: "Send Original" is the large primary, "Send with edits" and "Cancel" are small and stacked. Connection form: Test and Cancel are smaller than Save.
- **R12** Panels go stale across tabs (after a game delete, Worlds still says "(1 game)"). Decide whether a cross-panel refresh is worth it or rule it out.
- **R13** Deleting the last saved game leaves an empty "Saved Games" heading with no empty-state line.
- **R14** Rename reloads onto the Game tab, not Games; the rename disclosure is cramped and misaligned.
- **R15** Create World with an empty Map JSON shows only "That action failed."; the grey placeholder JSON looks like a value; the "Auto-generate options" checkbox is centred with its label below.
- **R17** Original finding 4.8: world cards use two layouts by description length.
- **R18** Preset previews cut mid-word with no ellipsis; "Set Active (…)" is a large primary beside small buttons.
- **R23** Original finding 3.5: the large empty sidebar area below the portraits.
- **R24** `.agents/skills/chronicler-ui-investigator/SKILL.md` still describes `#connection-status` ("always reads Connected") and `/fragment/action-area`; both are gone.
- **R25** `stub::options::test_options_dock_edit_fills_without_submitting` is reported LEAKY by nextest.

- **From 04 (R22 follow-up)**: a failed `/action/confirm` (transport failure or 5xx) closes the preview and the inline error the client creates lives inside the preview's own `<form>`, so `closeActionPreview()` wipes it — only the banner reports that failure. Decide whether the confirm path should report on the command form's own slot.
- **From 04 (R9/R21 follow-up)**: a 200 `ConcurrentGeneration` answer ("Still thinking...") still clears the command input, although that command was refused rather than consumed. The status display's `wait` state is the only client-side signal; ticket 02 owns that markup.
- **From 05**: `enable_auto_check` has no `id`, so htmx's focus restore cannot find the check box after the Text Check card's swap and focus falls to `<body>`. Fix (given in 05's own answer): add `id="enable_auto_check"` to the check box in `TextCheckCardTemplate` (`src/adapters/driving/http/settings/templates/settings.rs`).

## Done when

- Every item is fixed or dropped with a reason.
- `python build.py` is green. Commit after user approval.

## Answer

Every item was checked against the tree in this worktree (tickets 03 and 04
already applied). Nine of the twelve R-items are fixed (R4, R11, R13, R14, R15,
R17, R18, R24, R25), three are dropped with a reason (R5, R12, R23), and of the
two follow-ups filed by ticket 04 one is fixed (the confirm path's failure
reporting) and one is deferred to ticket 02.

### Fixed

- **R4 — the command input no longer narrows while the button reads
  "Generating…".** `assets/styles.css`: `#command-form button` takes a fixed
  `width: 150px` (the measured width of "Generating…" plus its icon and padding)
  in place of `min-width: var(--button-min-width)`, so a label change cannot
  resize the flex row's input. The 480px media query still sets `width: 100%`.
  **Tier 2**, `tests/browser/stub/invariants.rs` subtest
  `command_input_width_is_stable` (input width identical in both states, label
  not clipped). Sensitive: reverting the width fails the subtest.
- **R11 — the text-check preview's and the connection form's action rows are one
  row at one size.** `assets/styles.css`: a `.form-actions` / `.card-actions` /
  `.game-actions` row that holds a primary button renders every button in it at
  the primary's size (`--font-size-base`, `8px 20px`), so the hierarchy comes
  from the utility class's colour instead of a size jump; the preview's three
  controls also become one row (each confirm is its own `<form>`, so the row is
  now a flex container). The emphasis **stays** on "Send Original": the
  corrections are suggestions the player opts into (phase-1 ticket 04's "do not
  present an auto-apply as an authoritative 'Did you mean?'"), and the engine's
  own suggestions in shot 18 are wrong. `ui_design.md` documents the rule.
  No test (CSS).
- **R13 — deleting the last saved game answers with the list's empty state.**
  `POST /games/:id/delete` now answers with the same "No other saved games."
  line the panel renders (one source: `SavedGamesEmptyTemplate`, reached through
  `GamesPanelTemplate::saved_games_empty_html()`), and answers with an empty body
  when a saved game is left behind. **Tier 1**,
  `tests/http/games_delete.rs::test_delete_game_handler_last_saved_game_answers_with_the_empty_state`
  (SCENARIO 19.4, added to `docs/specs/games.md`). Sensitive: reverting the
  handler fails it.
- **R14 (both halves).**
  - The reload no longer lands on the Game tab: `assets/dashboard-panels.js`
    remembers the active tab in `sessionStorage` and restores it on load, so a
    rename (and a switch, a new game) returns to the tab the user was on. A fresh
    page still starts on Game. **Tier 2**,
    `tests/browser/stub/dashboard.rs::test_active_tab_survives_a_reload`
    (SCENARIO 16.39). Sensitive: removing the restore fails it.
  - `assets/games.css`: the rename disclosure opens *beside* its summary
    (`display: flex` on `details.game-rename`, the form a row) and its input is
    240px, so opening it neither grows the row nor drops the summary above the
    badges' line, and the display name is no longer truncated. No test (CSS).
- **R15 — the Create World form's three complaints.**
  - The reason is inline: a **4xx** body is the explanation the engine wrote for
    that slot, so it follows the short line ("That action failed. Invalid map
    JSON: EOF …"); a 5xx keeps the generic line with its text behind Details
    (`assets/dashboard-errors.js`, `refusalSummary`). Asserted in **tier 2**
    `tests/browser/stub/form_failures.rs::test_failed_connection_add_keeps_the_form_and_renders_inline`
    (SCENARIO 16.29, wording added). Sensitive: neutering the 4xx rule fails it.
    The two 5xx assertions in `failure_display.rs` that pin "the short line is
    not the server's text" still pass unchanged.
  - The placeholder reads as a hint: `Example: {"overworld": …}` / `Example: []`
    (`worlds.rs`) plus an italic `::placeholder` for `.json-editor`
    (`worlds.css`).
  - The checkbox label sits beside its box again: `.world-form-container
    .checkbox-label` restores `flex-direction: row` over the form's stacked
    labels.
- **R17 — one world-card layout.** The card is a column: a `.world-item-head` row
  with the name, the game count and the Edit/Delete actions, then the description
  below it, whatever its length (`worlds.rs`, `worlds.css`; the hand-copied stub
  fixture `tests/test_utils/stub_fixtures/worlds.html` follows). No test (CSS +
  markup); the existing card assertions (SCENARIO 25.8, the stub row checks) pass.
- **R18 — preset previews cut at a word boundary, and the card's action row is
  one size.** `truncated_preview` (`builders/presets.rs`) now trims to 120 chars,
  backs up to the last whole word and appends "…", so a preview cannot read as
  the whole field; the card's buttons take the same size rule as R11. **Unit**,
  `builders/presets_tests.rs::test_preset_card_html_preview_cuts_at_a_word_boundary`
  and `…_keeps_a_short_text_whole` (the existing truncation test now also pins
  the ellipsis). Sensitive: reverting the truncation fails both.
- **R24 — the `chronicler-ui-investigator` skill is current.**
  `.agents/skills/chronicler-ui-investigator/SKILL.md`: the DOM probe reads
  `#status-display` instead of the deleted `#connection-status`, the fragment
  probe fetches `/fragment/options-dock` instead of the retired
  `/fragment/action-area`, that path is dropped from the full-set list, and the
  "#connection-status always reads Connected" sentence is gone.
- **R25 — the leaky browser test.** The reported test was a symptom: nextest's
  leak detector waits for a test's stdio handles to close, and `with_stub_page`
  closed the page but left the shared Chromium running, so an orphaned child kept
  those handles open (the config's own description of the cause: "a test that
  creates a child process and lets it inherit those handles, but doesn't clean
  the child process up"). `tests/test_utils/browser.rs` now closes the shared
  browser *and* shuts the Playwright driver down before the test returns
  (`SharedBrowser::close`, called by `with_stub_page` and by the invariants
  test), and `with_test_page` shuts its driver down too. The reported test's code
  is unchanged: which test is reported depends on where the teardown happens to
  be slow. Evidence: the review's gate reported one leaky stub test; this
  worktree's browser tier is 77 passed, 0 failed, 0 leaky (twice, and once with
  the fix reverted it stayed leak-free, so the LEAK is intermittent by nature).
- **From 04: a 200 `ConcurrentGeneration` answer ("Still thinking…") no longer
  clears the command input.** The refusal now has a state of its own that the
  client can key on: `assets/dashboard-action-area.js` derives a `wait` state in
  `syncStatusClass` (reusing `statusIsGenerating()` for the thinking case) and
  syncs it after a swap into `#status-display`, `onCommandAfterRequest` clears
  the input only when the answer is neither a refusal nor a failed request, and
  `assets/styles.css` draws `.status.wait` in `--color-text-primary` per
  `ui_design.md`. The engine's own wait markup is now the exported
  `CONCURRENT_GENERATION_STATUS` (`action/handlers/actions.rs`), which the stub
  server serves, so its fixture cannot drift from the handler. **Tier 2**,
  `tests/browser/stub/dashboard.rs`
  `test_refused_concurrent_send_keeps_the_command_and_shows_the_wait_state`
  (SCENARIO 16.42 in `docs/specs/browser_dashboard.md`): the stub answers the
  refused outcome, the test snapshots the status display through a
  `MutationObserver` (the next 5s poll replaces the transient state) and asserts
  the container carries `wait` and not `ready`, that the label is drawn in the
  resolved `--color-text-primary`, and that the input still holds the typed
  command. Sensitive: reverting the `syncStatusClass` state fails it. Docs:
  `dashboard.md`'s status-class and command-input-lifecycle paragraphs.
- **From 05: the Text Check card's check box carries the id htmx restores focus
  to.** `TextCheckCardTemplate` now renders
  `<input type="checkbox" id="enable_auto_check" …>` with a matching `for` on
  its label, so the card's swap can put focus back on the control the player
  toggled. No test added: the stub serves no `/settings/text-check` route, so a
  browser leg would need new stub support (and a scenario) rather than an
  extension of the existing focus test, and an assertion that a template prints
  an `id` is tautological.

### Dropped, with the reason

- **R5 — the failure banner still pushes the tab bar down ~31px.** Phase-1
  ticket 08 decided this shape explicitly: the banner "renders only when there is
  something to say… While healthy it does not exist, so it costs 0px. When up it
  costs about 30px", with no close control. Reserving the space instead would
  leave a permanently empty strip — the same complaint R23 makes about the
  sidebar — and an overlay would cover the tab bar or the log. 31px is the
  decided cost, not a defect.
- **R12 — panels are not refreshed across tabs, as a decision of scope.** The
  panels fetch once by design (`dashboard.md`), and the only staleness that
  contradicted another surface (role health vs the banner) is R8, which ticket 02
  fixes. A refresh-on-show would either discard in-progress input (a half-typed
  connection form, an open world form, a preset edit) or need per-panel state
  preservation and dirty checks; a targeted refresh of the Worlds panel after a
  game delete can wipe unsaved JSON edits in the panel it refreshes.
  Correcting the echoing surface alone is *not* ruled out: an out-of-band swap of
  the affected count, the way ticket 02 refreshes the Settings role-health cells
  from the header poll, would leave the panel's forms untouched. It is left as a
  follow-up (below) rather than claimed impossible; until then the stale
  "(1 game)" count is corrected by the next full page load.
- **R23 — the empty sidebar area stays.** It is the scene column's own
  background, sized to the story log so the two frames align; there is no content
  to put there. Content-sizing the sidebar instead leaves the same empty space
  beside a ragged edge, and filling it would mean inventing a scene-details
  region — a product decision, not a fix.

### Follow-ups (non-blocking)

- **R12's echoing surface.** After a game delete the Worlds panel still reads its
  stale "(1 game)". An out-of-band swap of that count (the mechanism ticket 02
  uses for the role-health cells) is the shape a fix would take; it is not in
  this ticket's scope.
- `tests/browser/stub/story_log.rs::test_story_log_is_keyboard_scrollable` failed
  once in a browser-tier run (161s, three-way concurrency) and passed in
  isolation and in the next full run. It presses ArrowDown for up to 3s and
  expects the log to scroll; it is pre-existing and untouched by this ticket.
- A failed confirm still closes the preview, so the corrected text in its
  textarea is lost and the player re-sends from the original command. The failure
  is now visible where it matters (the command form's slot), so this is only the
  next step if the corrected text should survive a failure. The same holds for a
  refused confirm: a 200 `ConcurrentGeneration` answer to a send control closes
  the preview and clears the input, because the input holds the original text
  rather than the corrected one the textarea held.

### Checked on a probe server (shots in this worktree's `tmp/`)

`./target/debug/chronicler_engine --world redmist_estate --port 3202` (its own
database, so the user's games were untouched), headless Chrome at 1280×856:

- `tmp/p2-06-01-worlds.png` — R17: both cards put the name, the count and the
  actions on one head line (Edit/Delete right edges both at x=989) with the
  description below, long or short.
- `tmp/p2-06-02-world-form.png` — R15: the checkbox sits beside "Auto-generate
  options after each turn" (same top), and the JSON editors read "Example: …" in
  italic grey.
- `tmp/p2-06-03-games-rename.png` — R14: the open rename disclosure keeps the
  summary on the badges' centre line (191 = 191), shows the whole display name in
  a 240px input with Save beside it, and the row stays one line (52px).
- `tmp/p2-06-04-presets.png`, `tmp/p2-06-05-preset-copy.png` — R18: previews end
  at a word boundary with "…" ("Your goal is to…"), and a card's Set Active /
  Edit / Delete / Duplicate are all 34px at 14px.
- `tmp/p2-06-07-preview-row.png` — R11: "Send
  Original", "Send with edits" and "Cancel" share one row at one size.
- R4 measured in the page: the command input is 832px wide in both the idle and
  the "Generating…" state, and the 150px button does not clip the label.

The probe's Text Check was set back to Disabled before the server was stopped.
