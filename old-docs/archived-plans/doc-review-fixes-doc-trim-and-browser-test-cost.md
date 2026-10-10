# Doc review fixes, doc trim, and browser-test cost

## Summary

This plan is the only record of these items. No `.scratch/` ticket tracks them. All of the work goes into **one commit**, made after the pending phase-2 UI commit. The plan covers four items:

1. Fix the review findings in the `dashboard.md` and `ui_design.md` paragraphs that the UI commit added.
2. Trim both docs in full. Behaviour details (edge cases, focus, scroll, input clearing, error-slot choice) leave the reference docs: the specs define them, and the reference docs do not cite them. Rationale moves to code comments or `dashboard_design.md`. Constants that are defined in code are removed.
3. Make the browser tier cheaper:
   - Audit the stub tests. Remove duplicates, move layout-only checks to `invariants.rs`, and move server halves to tier 1.
   - Measure the cost of each test.
   - Start timer polls from the tests instead of waiting for them (Task 3.3).
   - Measure again. Consolidate tests into runner tests only if the new measurement justifies it (Task 3.4).
4. Add two short rules so the growth does not start again.

**Status (2026-10-10):** Phases 1 to 3 are done; Phase 4 (gate and commit) remains. `/code-review` ran, and its findings are fixed (see Review). User decisions: do Task 3.4 now, fix the story-log scroll race now (Task 3.5), and trim `ui_design.md` again under the revised principle (Task 2.4).

**Precondition (met: c754deb9):** The phase-2 UI work (tickets 02–06) is committed before Phase 1 starts. The working tree has no other changes to `dashboard.md`, `ui_design.md`, `styles.css` or `tests/browser/stub/`.

## Key Changes

- `docs/diataxis/reference/frontend/dashboard.md`, `ui_design.md`: corrected, then trimmed.
- `docs/diataxis/explanation/dashboard_design.md`: gets one new short section, "Text-check preview emphasis".
- `assets/styles.css`: the comment on the action-row button size rule is corrected.
- `tests/browser/stub/*.rs`: layout-only checks move to `invariants.rs`. Tests start timer polls themselves. The tests are merged into runner tests only if Task 3.4 goes ahead.
- `tests/browser/stub/support.rs`: `poll_now`, which fires a polled container's own request. It is stub-only, so it lives in the stub support module.
- `tests/test_utils/wait.rs`: `watch_htmx_requests`, `htmx_request_count` and `wait_for_htmx_requests` are deleted. Task 3.3 removed their last callers.
- `docs/specs/browser_*.md`: a scenario is removed only when the audit removes or moves its test.
- `docs/AGENTS.md`, `tests/STRATEGY.md`: one rule each.
- Only if Task 3.4 runs: `scripts/validate_feature_spec.py`, new `scripts/tests/test_validate_feature_spec.py`, `.config/nextest.toml`, `tests/browser/stub/support.rs`.

## Implementation

### Phase 1: Fix the review findings

- [x] #### Task 1.1: Correct the factual errors (3 SP)
  - [x] ##### SubTask 1.1.1: Fix the two errors in `dashboard.md` (1 SP)
    - Failure paragraph: the short line is always "That action failed.". For a 4xx response, the refusal text follows it.
    - Failure paragraph: the inline slot for an unreachable engine reads "The engine is unreachable.". The banner line is longer.
  - [x] ##### SubTask 1.1.2: Fix the button size rule in `ui_design.md` (1 SP)
    - Describe the rule by its selector: "Buttons in `.form-actions`, `.card-actions` and `.game-actions` render at `--font-size-base` and `8px 20px`".
    - Correct the CSS comment in `styles.css` to say the same thing.
    - Remove "switch" from the "Typical actions" cell of the `.btn-cyan` row.
  - [x] ##### SubTask 1.1.3: Remove the copied constant (1 SP)
    - Replace "within 8px of" with "at or near its bottom".
- [x] #### Task 1.2: Remove rationale from the reference prose (3 SP)
  - Remove these explanations:
    - the htmx `afterRequest`/`sendError` order (the comment in `dashboard-action-area.js` holds it)
    - the reason for the empty state ("only renders with the whole panel")
    - the slogan "A command the engine never took is never lost"
    - the "cannot leave it in the Ready colour…" clause, which becomes "the class always matches the content"
  - Move the Send Original emphasis reason to `dashboard_design.md`, in the section "Text-check preview emphasis".
  - Remove the Send-button width reason from `ui_design.md`. The CSS comment keeps it.
  - Divide sentences that state more than one fact into one statement per sentence (STE).

### Phase 2: Trim both docs in full

- [x] #### Task 2.1: Trim `dashboard.md` (8 SP)
  - Rule (revised by the user during implementation): keep regions, controls, endpoints, cadences, and state names, one sentence each. Remove behaviour details. The specs define them, so the reference doc does not restate them and does not cite scenario numbers for them. One overview sentence points to the specs in Document References.
  - Result: 3634 → 2049 words.
  - [x] ##### SubTask 2.1.1: Failure and health states, Tabs, Polling Cadences (3 SP)
  - [x] ##### SubTask 2.1.2: Game tab: Story Log, Action Area, command input, text check, slash palette (3 SP)
  - [x] ##### SubTask 2.1.3: Edit, Delete and Swipe flows, Games tab, Settings tab (2 SP)
- [x] #### Task 2.2: Trim `ui_design.md` (5 SP)
  - Keep the current format, which lists token and value entries.
  - Remove rationale sentences.
  - Remove behaviour that `dashboard.md` or a spec already states. Do not link to the spec for it.
  - Result: 3298 → 3068 words. Existing error fixed: the visual sidebar holds the location image, not a location-header bar.
- [x] #### Task 2.4: Trim `ui_design.md` under the revised principle (3 SP)
  - Principle (user): no doc needs to describe everything the application does. A reader can read the code. A reference doc names the parts; it does not describe behaviour, even behaviour that no spec defines.
  - Remove the button-visibility table and the other behaviour descriptions (states, when controls appear, what a click does).
  - Keep design tokens, component structure, and visual states.
  - Result: `ui_design.md` 3068 → 2915 words. Removed the visibility column, the "when it appears" clauses, the swipe and retrigger rules, and the save-timing spec map.
  - Same principle on `dashboard.md`: the Edit, Delete, Swipe and Retrigger click-by-click flows became one table of controls and endpoints, plus the two domain facts (a swipe switch restores the swipe's snapshot; retrigger re-runs the previous turn's trigger narration). `dashboard.md` 1935 → 1750 words (after the second review dropped the table's Entry column).
  - `docs/AGENTS.md`: "Coverage is not a goal" is in `## Documentation Layers`; the behaviour rule is in `### Reference defers to source` (the anchor `chronicler-docs-hygiene` checks); the names of regions, controls, endpoints, cadences and state names are one more bullet in "What Reference docs **do** carry".
- [x] #### Task 2.3: Add rules against new growth (1 SP)
  - `docs/AGENTS.md`, Reference section: a Reference doc does not document behaviour details, does not restate them, and does not cite scenario numbers for them. It names regions, controls, endpoints, cadences and state names.
  - `tests/STRATEGY.md`, placement rule: "A check that asserts only computed style or layout goes in `tests/browser/stub/invariants.rs`. It has no spec scenario. It puts its markup in place with client-side JS and does not change the shared stub's state."

### Phase 3: Browser-test cost

- [x] #### Task 3.1: Audit the stub tests (5 SP)
  - Apply three criteria to all 71 stub tests. Start with `failure_display.rs` (15) and `story_log_edit.rs` (16).
    1. **Duplicate:** two tests assert the same behaviour. Delete the test whose failure name is less precise.
    2. **Layout-only:** the test asserts only computed style, size, or position. Move it to `invariants.rs` as a `check_*` subtest that puts its markup in place with client-side JS. Remove its scenario and update the spec text. Example: the layout part of 16.38. Its "dead engine renders the inline error" part is already covered by `test_dead_engine_keeps_the_typed_command`.
    3. **Server half:** the test asserts a server response. Move that assertion to `tests/http/`.
  - A behaviour test that also measures layout stays as it is. Splitting it saves no launch.
  - Record the lists of deleted and moved tests in the commit notes.
  - Result: no test deleted (closest candidate 16.20 is the only test of the banner after a dead send). The layout part of 16.38 moved to `invariants.rs` as `check_action_area_makes_room_for_the_inline_error`. Scenario 16.38 now covers only the recovery. No server half moved; each already has a tier-1 test. Full audit table: `.pi-herdsman/stub-audit-3.1-3.2.md`.
- [x] #### Task 3.2: Measure the cost of each test (3 SP)
  - Run `python build.py --test-timings` (full gate). Read the median stub-test duration and the browser-step total.
  - Run `NEXTEST_SUCCESS_OUTPUT=final python build.py test-pattern test_invariants`. Read the median subtest duration from the printed summary.
  - Launch overhead = median stub test − median invariant subtest.
  - If the summary does not print, use the shortest stub-test duration as an upper bound for the overhead, and record this.
  - Record the median stub-test time, the launch overhead, and the browser-step total in the commit notes.
  - Result: median stub test 5.18 s, median invariant subtest 0.79 s, so the formula gives 4.39 s (85%). Direct launch cost is only 1.5–2.2 s (29–42% uncontended). The formula also counts the poll waits inside each test. Browser step: 185.9 s.
  - Finding: the slowest stub tests (8–25 s) wait passively for the 5 s timer polls. The idle wait costs more than the Chrome launch.
- [x] #### Task 3.3: Start timer polls from the tests (3 SP)
  - Add one helper that fires a polled container's own trigger (for example `htmx.trigger(selector, 'load')`), so the request and the swap path are the same as a timer poll.
  - Change each stub test that waits for a timer poll to start the poll after it changes the stub state. Do not weaken assertions. Assertions must accept one extra timer-driven swap.
  - Keep on the timer only the tests where the cadence itself is the behaviour.
  - Measure the browser step and the changed tests before and after. Run the browser tier twice to check for flakes (retries are 0).
  - Record the results in the commit notes.
  - Result: 20 tests converted, 204.6 s → 81.3 s. Stub-test median 5.18 s → 4.29 s, maximum 24.0 s → 7.0 s. Browser step 185 s → about 133 s (−28%), over 7 runs.
  - Left on the timer, because the cadence is the behaviour: 30.3 (`test_polling_pauses_during_edit`), 30.7 (`test_escape_cancels_edit`), 30.17 (`test_successful_save_holds_the_edit_lock_until_the_poll`).
  - Existing flake found and fixed: 30.13 (`test_story_log_is_keyboard_scrollable`) failed when the first follow-the-bottom scroll took effect. The test now scrolls the log to the top before its baseline read.
  - Not fixed (outside scope, `assets/`): `dashboard-story-log.js` reads `scrollHeight` on the load swap before layout, so the first follow-the-bottom is nondeterministic.
- [x] #### Task 3.4: Consolidate stub tests into runner tests (8 SP)
  - The user decided to do it now. Baseline: browser step about 130–140 s after Task 3.3. Each stub test still pays about 2 s of launch.
  - [x] ##### SubTask 3.4.1: Move the runner to shared code (3 SP)
    - Move `run_subtest` and `print_summary` from `invariants.rs` to `tests/browser/stub/support.rs`.
    - Each subtest gets a fresh `StubServer` with its own `StubActionOutcome`, and one `SharedBrowser` is shared across the subtests.
    - The runner panics after the summary if any subtest failed.
  - [x] ##### SubTask 3.4.2: Change the spec-coverage validator (3 SP)
    - In `scripts/validate_feature_spec.py`, treat `async fn check_*` in `tests/browser/stub/` as a tag anchor and a tag-rule target.
    - Exempt the `run_*` runner tests by name.
    - Add tests in the new `scripts/tests/test_validate_feature_spec.py`.
  - [x] ##### SubTask 3.4.3: Convert the modules and the timeout (2 SP)
    - Convert one stub module per runner test.
    - Add a `.config/nextest.toml` override that gives `binary(browser) and test(/run_/)` a 180s slow-timeout.
    - Keep the change only if the browser-step total drops clearly compared with Task 3.3. If it does not, revert Task 3.4.
  - Result: **kept.** Browser step 133.5 / 134.3 s → 91.1–94.5 s warm (−31 %). 13 tier runs, all 27 passed.
  - `StubRunner` in `tests/browser/stub/support.rs`: one `SharedBrowser` per runner; per check a fresh `StubServer` with the outcome the runner names and a fresh page; `catch_unwind`; the report name comes from the check function (`type_name`); `finish()` prints the summary, closes the browser, then panics naming the failed checks.
  - 71 tests → `check_*` functions in 16 runners. 12 modules have one runner; `failure_display`, `story_log_edit` and `dashboard` have two each. Largest runner 23.3 s. Bodies are unchanged except one pre-existing flake fix in `games.rs` (wait for the row removal, then assert focus; it failed 2/8 runs on the old harness).
  - Validator: `check_*` anchors a tag and is a tag target in the stub tier; `run_*` is exempt. After review, a `check_*` that no runner in its file calls is reported as unwired. 10 tests in `scripts/tests/test_validate_feature_spec.py`.
  - Files outside the listed set: `with_stub_page` deleted from `tests/test_utils/browser.rs` (no callers); a guardrail test comment in `tests/infrastructure/guardrails/structure_tests.rs` reworded; `docs/diataxis/reference/coding_standards/testing.md` and `tests/STRATEGY.md` ("Stub-tier shape") describe the runner design.
- [x] #### Task 3.5: Fix the story-log first-scroll race (2 SP)
  - `assets/dashboard-story-log.js` assigns `scrollTop = scrollHeight` on the load swap before layout is complete, so the first follow-the-bottom does not always take effect.
  - Write a failing test first. Read `scrollHeight` after layout (for example in `requestAnimationFrame`).
  - Then check whether the scroll reset added to 30.13 in Task 3.3 is still needed. Keep it only if the test still needs a known start position.
  - Cause: the log's height is what the flex column leaves over. At the load swap the log does not overflow yet (571 px, no scroll); the options-dock swap about 28 ms later shrinks it to 422 px. One `requestAnimationFrame` was not enough (failed 1/5).
  - Fix: `followStoryLogBottom` assigns at once and, while the log cannot scroll, retries each frame for at most `STORY_LOG_FOLLOW_TIMEOUT_MS` (1000). After review, the retry also stops when an edit opens.
  - Test first: scenario 30.23 and `check_first_load_swap_lands_at_the_logs_bottom` failed before the fix ("gap 30 px").
  - 30.13 reset **kept**: the load swap now reliably leaves the log at its bottom, so ArrowDown needs a known start position.

### Phase 4: Close

- [ ] #### Task 4.1: Gate and commit (1 SP)
  - Run `python build.py`. It must be green.
  - The user reviews the diff. After approval, make one commit through `/commit-and-push`. Put the notes from Tasks 2.1, 3.1, 3.2, 3.3, 3.4 and 3.5 into the commit message body.

## Review

`/code-review` (Standards and Spec axes) on the uncommitted diff against `c754deb9`. Fixes:

- `dashboard.md`: removed the remaining behaviour details (the failure-display bullets, the state order, the empty-input continuation) and the stray `.:`. Restored the "No other saved games." empty state, which the trim removed with its reason.
- `poll_now`: htmx queues a request behind one in flight on the same element. The helper now waits until the element is idle, fires, and waits for `htmx:afterRequest` with its own `xhr`.
- `story_log.rs`: the repeated `poll_now` pair is now `poll_story_log_twice`.
- Dead helpers deleted from `tests/test_utils/wait.rs`.

Second `/code-review` (Tasks 2.4, 3.4, 3.5). Fixes:

- `ui_design.md` and the `styles.css` comment name the utility classes the size rule covers (`.game-actions` also holds `.btn-reset-small`).
- `dashboard.md`: the controls table drops its Entry column, which restated visibility.
- `followStoryLogBottom`: the per-frame retry stops when an edit opens.
- `.config/nextest.toml`: the comment gives the real reason for the block order (first matching override wins).
- Validator: unwired `check_*` rule with tests.
- `narrator_mode.md`, `options.md`: scenario ranges removed from Document References.
- Rejected: "imports are not fmt-stable". `rustfmt.toml` sets `reorder_imports = false`, and `cargo fmt --check` passes.

Resolved with the user: the button-visibility table in `ui_design.md` is removed (Task 2.4). A doc need not describe behaviour that only the code defines.

## Test Plan

- Docs: `python build.py validate-docs` passes.
- Specs and tags: `python build.py spec-coverage` passes after Tasks 3.1, 3.3 and 3.4.
- Browser tier: `python build.py browser` passes after each Phase 3 task.
- Validator: the tests in `scripts/tests/test_validate_feature_spec.py` pass (only if Task 3.4 runs).
- Final: `python build.py` is green. The browser-step time is compared with the baseline from Task 3.2.

## Per Task/Sub Task Validation Steps

- 1.1–1.2: Read each changed sentence against `assets/dashboard-errors.js`, `styles.css`, and the templates. Then run `validate-docs`.
- 2.1–2.2:
  - Word count with `wc -w`. Targets: `dashboard.md` ≤ 2600 words, `ui_design.md` ≤ 3100 words. If a target is missed, record the reason.
  - Run `validate-docs`.
- 2.3: Run `validate-docs`.
- 3.1: Run `python build.py test-pattern invariants`, `spec-coverage`, `browser` and `integration`.
- 3.2: The three values are recorded.
- 3.3: Run `clippy`, `spec-coverage` and `browser` twice, and compare the step time.
- 3.4: Run the validator tests, `guardrails`, `spec-coverage` and `browser`, and compare the step time.
- 4.1: The full gate is green.

## NOT in scope

- Moving client behaviour (focus, scroll, input value) to tier 1.
- A JS unit runner (node + jsdom).
- Changes to tier-3 browser tests (`tests/browser/*.rs`).
- The open items in the phase-2 map (snapshot-restore paths, R12, the preview corrected-text loss).
- Other Diátaxis docs.

## What already exists

- `SharedBrowser`, `run_subtest` and `print_summary` (`tests/test_utils/browser.rs`, `tests/browser/stub/invariants.rs`).
- `--test-timings` in `build.py`, and the `spec-coverage` step.
- `scripts/tests/` as the home for Python script tests.
- Scenario links already used in `dashboard.md` and `options.md`.

## Failure modes

- The invariant summary does not print: use the shortest stub test as an upper bound.
- A test fires a poll and a timer poll swaps at the same time: the assertion waits for a condition, so an extra swap does not fail it.
- Consolidation gains too little (for example, because the largest module becomes the longest single item): Task 3.4 is reverted.
- A subtest fails inside a runner: the summary names it, then the runner panics.
- A moved layout check depends on server state: it is not moved, per the `STRATEGY.md` rule from Task 2.3.

## Assumptions

- This plan file is the only tracker. No `.scratch/` ticket is created.
- `ui_design.md` keeps its token and value format. The rule against constants applies to behaviour prose only.
- nextest runs each test in its own process. A shared Chrome is only possible inside one runner test (inferred from `invariants.rs` and `with_stub_page`).
- The 13 new browser tests close real coverage gaps. They stay at tier 2 unless the audit moves them.
- `NEXTEST_SUCCESS_OUTPUT` reaches nextest through `build.py test-pattern` (verified in Task 3.2: the subtest summary printed).
- A test may start a poll itself, because the behaviour under test is the swap, not the timer (inferred).
