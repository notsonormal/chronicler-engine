# Review-findings fix plan — narrator-modes-ui-tests branch (revised)

## Summary

Fix the HIGH (1) and MED (2-6) findings from the four-review consolidation, all LOW findings, and the three valid findings from the supplementary external review. One production bug (preset-id collision), one missing rationale (assert_log_invariants), dead-machinery removal in build.py, a validator single-scan refactor, a settle-harness unification, a guardrail tightening, three stub-tier fixes, a test_helpers cohesion split, and a docs-hygiene pass. Permission-config changes are explicitly out of scope. Each phase is independently shippable and validated before the next.

## Key Changes

- `handler_helpers.rs`: `generate_preset_id` gets a random uuid-v4 suffix (dependency already present); workarounds deleted from both preset tests.
- `game_state.rs`: comment recording why the log-ordering invariant is not asserted (unsound as a global invariant after `/history/delete`), plus a note in the owning effort's ticket doc.
- `build.py`: delete the dead FLAKY machinery **and its tests in `scripts/tests/test_build_cli.py`**; collapse `browser` kind into `tests` with a one-shot LLM note; single builder for the test-pair plan; rename `exclude_browser` → `browser_only`.
- `scripts/validate_feature_spec.py`: one scan per file producing annotations + attributes + fn names; finders become pure; delete empty `TAG_EXEMPT_TESTS`.
- `tests/test_utils/htmx_settle.rs`: snapshot-returning poll; one `SettleOutcome` constructor, one `settled` rule with the cap-window why-comment; rename `SettleOutcome::describe` → `log_line`.
- Guardrail + `tests/browser/prompt_presets.rs`: ban `.set_checked(`/`.check(`/`.uncheck(`; path-based stub exemption; per-line trailing `settle-guard-exempt` marker for the no-hx checkbox.
- `tests/test_utils/stub_server.rs`: port range from `tests/test_config.json`; consume the now-`pub` `add_status_swap_headers`; render the options dock from `OptionsDockTemplate` (fixture deleted).
- `tests/http/test_helpers.rs` → `tests/http/support/{http_requests,http_fixtures,http_assertions,app_wiring}.rs`; merge the two body readers; `post_action`/`post_action_check` reimplemented over `post_form`.
- `tests/test_utils/html.rs` (new): shared `preset_card_html_slice` used by the HTTP test and `browser.rs`.
- Docs: STRATEGY.md `flow/` ghost, `SharedBrowser`/testing.md launch-cost claims, WAIT_HELPERS.md deleted sync entry, options.md scenario order, ticket references stripped from `assets/`.

## What already exists (reused, not reimplemented)

- `wait_for_condition_async` (wait.rs) — stays for HTTP/browser helpers; the settle-harness poll loop replaces it only where a snapshot return is needed.
- `get_config_port` + `TestConfig::from_file` (tests/test_utils/server.rs) — stub port resolution.
- `uuid` v4 and `askama` — normal dependencies, visible to integration test targets.
- `post_form` (test_helpers.rs) — canonical form-POST builder the action twins will delegate to.
- `OptionsDockTemplate::render` — already exercised by `templates_tests.rs`; the stub reuses the same path.
- `TAG_EXEMPT_DIRS`/`TAG_EXEMPT_FILES` — the validator's existing exemption pattern the guardrail marker mirrors.
- `_NEXTEST_SUMMARY_RE` + `_nextest_summary_line` — survive; only the flaky segments die.

## Implementation

### Phase 1: Production fixes

- [ ] #### Task 1.1: Collision-proof preset ids (1 SP)
  - `generate_preset_id()` (`src/adapters/driving/http/utils/handler_helpers.rs:29`) → `format!("preset-{millis}-{8 hex chars of Uuid::new_v4().simple()}")`. Counter suffix rejected: restarts reset it.
  - Delete the collision workarounds: the "Do not stress-loop" paragraph in `tests/browser/prompt_presets.rs:16-18`; the fixed-id rationale sentences in `tests/http/prompt_presets.rs:815-819` (keep the seed — it is the chain precondition — and the provenance sentence).
  - Grep tests for `preset-\d`-format assumptions before finalizing.
- [ ] #### Task 1.2: Record the assert_log_invariants removal (1 SP)
  - Comment on `assert_state_consistency` (`game_state.rs:453`): the last-Narration-after-last-Input ordering was removed because it is not a global invariant — `/history/delete` legitimately pops a Narration. No ticket references in code (standards rule).
  - One-sentence note in the owning effort's ticket file (`.scratch/narrator-modes-and-options/`, ticket 20).

### Phase 2: build.py

- [ ] #### Task 2.1: Delete the dead FLAKY machinery, including its tests (1 SP)
  - Remove `_NEXTEST_FLAKY_RE`, `_nextest_flaky_tests`, `_NextestSummary.flaky`, `flaky_text()`, the epilogue flaky print, the flaky segment handling in `_nextest_summary_line`, and the stale docstring sentences.
  - Delete the now-dead tests in `scripts/tests/test_build_cli.py`: `test_flaky_segment_is_kept`, `test_zero_flaky_segment_is_omitted`, `test_flaky_and_skipped_render_together`, and the `_nextest_flaky_tests` extraction tests (confirmed present at :339-:367); update any summary-line test that asserts a flaky segment.
  - Add a one-line comment in `[profile.default]` of `.config/nextest.toml`: retries stay 0 — a flake must fail the gate; the browser tier runs as its own gate step.
- [ ] #### Task 2.2: Single test-pair builder, fold `browser` kind into `tests` (3 SP)
  - Replace the four label-conditional + GateStep blocks with one pair builder computing `(int_cmd, browser_cmd, suffix)` by `args.coverage`; labels via `f"Running integration tests{suffix}{tail}"`.
  - Delete the `elif step.kind == "browser"` arm; both steps become kind `"tests"`; revert the kind comment.
  - Keep the LLM-skip NOTE printed exactly once: a `tests_note_printed` flag local to `_execute_gate_plan` (folding browser into `tests` would otherwise print it twice).
- [ ] #### Task 2.3: Rename `get_coverage_cmd(exclude_browser)` → `browser_only` (1 SP)
  - Invert the param meaning, update the docstring and the call sites (2 remain after Task 2.2).

### Phase 3: Validator

- [ ] #### Task 3.1: Single scan per file in validate_feature_spec.py (3 SP)
  - New `scan_test_file(path) -> TestFileScan` (annotations + all test attributes with fn names, one `read_text`).
  - `find_untagged_tests` and `find_surface_violations` become pure functions over the scan; `main`'s coverage loop reuses it; the three duplicated OSError handlers collapse into the scan.
  - Delete `TAG_EXEMPT_TESTS: dict[...] = {}` and its consumption branch.
  - `count_quarantine_tests` keeps its own read (different directory, no annotations needed).

### Phase 4: Settle harness

- [ ] #### Task 4.1: Unify poll + one SettleOutcome constructor (3 SP) — full judo per decision
  - New private `poll_settles(page, scope, selector, timeout) -> PollResult{matched, targets}` where scope is an internal enum `SettleScope { WholeDocument, Since(u64) }`; the poll loop keeps the last scoped read, so no extra final read is needed on match or timeout.
  - New `SettleOutcome::from_poll(expected, result)`: `settled: result.matched || !matching_targets.is_empty()` with a comment: the `matched` arm covers a target the poll saw that could fall out of the 64-entry cap before a re-read.
  - `await_panel_ready` and `finish_settle` become thin calls; delete `await_settle_target`, `settles_since`, `settle_targets_all`, the duplicated filter/assembly, and the divergent `&&`/`||` rules. Behavior change is confined to the >64-settles edge (fewer false panics).
- [ ] #### Task 4.2: Rename `SettleOutcome::describe` → `log_line` (1 SP)
  - Update the two call sites (`click_and_settle`, `select_option_and_settle`); keep `SettleTarget::describe`.
- [ ] #### Task 4.3: Guardrail: path exemption, checkbox strings, exemption marker (3 SP) — marker per decision
  - `check_browser_interactions_use_htmx_settle`: exempt by path — `if !path.starts_with("browser/") || path.starts_with("browser/stub/")` (drop the `with_test_page` marker check; behavior identical for every file in the tree, now matching the docstring's claim).
  - `BANNED_INTERACTIONS` gains `.set_checked(`, `.check(`, `.uncheck(` (`[&str; 4]` → `[&str; 7]`).
  - Rationale (verified): the preset-mode checkboxes (`builders/presets.rs:107-108`) carry no hx attributes, so a settle wait for them would never be satisfied — a blanket ban is unworkable without an escape. Add a per-line exemption marker (`settle-guard-exempt`) skipped by the scan before the ban check; the marker must be a **trailing comment on the flagged line** (the scanner already skips full-comment lines, so a marker on its own line would not suppress anything). Docstring + violation message document it. `tests/browser/prompt_presets.rs:79` gets the marker + why ("no hx-trigger; the form posts via the settle-gated Save click").
  - Grep `tests/browser/*.rs` for existing `.check(`/`.uncheck(` substrings first (false-positive check; only `.set_checked(` at prompt_presets.rs:79 is known).
  - `structure_tests.rs`: add reject-set_checked, allow-marker, and stub-by-path (content containing `with_test_page` AND a raw `.click(`) cases; adjust the existing `exempts_stub_tier` case if it relied on marker absence.

### Phase 5: Stub tier

- [ ] #### Task 5.1: Stub port range from test config (1 SP)
  - `stub_server.rs:69`: `get_config_port(CONFIG_PATH).expect("allocate a stub port")` (imports from `super::`); delete the `3010, 3050` literals.
- [ ] #### Task 5.2: Stub consumes the engine's status-swap headers (1 SP)
  - Widen `add_status_swap_headers` (`builders/headers.rs`) to `pub` (module chain verified: `pub mod headers`); the `action_check` Pending arm builds the response then calls it instead of re-typing `hx-retarget`/`hx-reswap`; update the mirror comment.
- [ ] #### Task 5.3: Options dock rendered, fixture deleted (1 SP)
  - Stub `/fragment/options-dock` renders `OptionsDockTemplate::new(OptionsDockViewModel::new(canned_options, false)).render()` (the three fixture option texts; askama escapes naturally); `.expect("render options dock")` so a template failure fails loud. Delete `stub_fixtures/options_dock.html` + `FIXTURE_OPTIONS_DOCK`. (STRATEGY.md does not itemize fixtures — no doc edit needed.)

### Phase 6: HTTP test helpers

- [ ] #### Task 6.1: Split test_helpers.rs by concept (5 SP)
  - New `tests/http/support/{http_requests,http_fixtures,http_assertions,app_wiring}.rs` + `mod support;` in `tests/http/mod.rs`: requests gets `fetch_body`/`response_body` (merged: one body reader at the 65536 cap, `fetch_body` delegates after its GET/status assert)/`post_action`/`post_action_check` (over `post_form`)/`post_empty` (as-is)/`post_form_with_hx`/`wait_idle`; fixtures gets `seeded_storage_with_initial_game`/`if_world`/`world_form_body`; assertions gets `assert_option_selected`; app_wiring gets the `app_with_narrator*` builders. (`posture_world`/`always_on_world` live in `tests/http/options.rs`/`worlds.rs`, not here — untouched.)
  - Delete `test_helpers.rs`; flip the `use crate::test_helpers::{...}` / `super::test_helpers::` imports in the 14 importing files (`actions`, `worlds`, `options`, `narrator_mode`, `games_config`, `games_fragment`, `reset`, `retrigger`, `story_log`, `swipe_new`, `games_switch`, `games_delete`, `requires_migration/{fragment,games_fragment_handlers,worlds_fragment_handlers}.rs`) to the concept modules. No re-export shims.
  - Trade-off noted: the antipattern report proposed deferring to "the next surface"; the user's fix-all instruction overrides the deferral.

### Phase 7: Preset-card slicing

- [ ] #### Task 7.1: Shared `preset_card_html_slice` (1 SP)
  - New `tests/test_utils/html.rs` (declared in `tests/test_utils/mod.rs`): `pub fn preset_card_html_slice<'a>(body: &'a str, anchor: &str) -> Option<&'a str>` — find anchor, `rfind` the card open before it, bound at the next card open or EOF.
  - `tests/http/prompt_presets.rs:88` uses it with the duplicate-URL anchor (`unwrap_or_else` panic, existing message); `tests/test_utils/browser.rs` `preset_card_rendered` wraps it with the title anchor (bool as today).

### Phase 8: Docs hygiene + glue

- [ ] #### Task 8.1: STRATEGY.md `flow/` ghost (1 SP)
  - Delete "The `flow/` tests are multi-call spec scenarios at HTTP E2E." (line 27).
- [ ] #### Task 8.2: SharedBrowser launch-cost honesty (1 SP)
  - `browser.rs:94-96` doc → wraps one launch so several pages can share the cost (`invariants.rs` does); `with_stub_page` launches one per test for isolation.
  - `docs/diataxis/reference/coding_standards/testing.md:14`: drop "paid once per test binary"; state per-test launch + the invariants.rs exception.
- [ ] #### Task 8.3: WAIT_HELPERS.md catalog fix (1 SP)
  - Delete the sync half (`// Sync: ...` + `wait_for_condition_sync` block) from the Generic condition waits section.
- [ ] #### Task 8.4: options.md scenario order (1 SP)
  - Move Scenario 24.13 after 24.5 within "Using the offered set" (24.4, 24.5, 24.13). Validator keys on IDs, not order — no gate impact.
- [ ] #### Task 8.5: Ticket references in shipped assets (1 SP)
  - `assets/index.html:377`: drop "(ticket 04)"; `assets/styles.css:569`: drop "(ticket 04 V4: ...)"; keep the why in both.
- [ ] #### Task 8.6: New-file glue (1 SP)
  - Every new file (`support/*.rs`, `test_utils/html.rs`) carries the repo-standard `//!` module header; run `python scripts/generate_tests_structure_index.py` after Phases 6-7 so `tests/AGENTS.md` lists the new modules.

## Test Plan

- Targeted validation after each phase (below); final full `python build.py` gate as the acceptance gate.
- Phases are independent commits; each phase's validation must be green before the next starts.

## Per Task/Sub Task Validation Steps

- Task 1.1: `python build.py test-pattern "prompt_presets"`, `python build.py unit`
- Task 1.2: `python build.py unit`
- Tasks 2.1-2.3: `python build.py py-tests` (covers the deleted/updated flaky tests), `python build.py check`, `python build.py clippy`
- Task 3.1: `python scripts/validate_feature_spec.py` exits 0; `python build.py py-tests`
- Tasks 4.1-4.2: `python build.py test-pattern "browser"` (every tier-3 + stub test exercises the helpers)
- Task 4.3: `python build.py test-pattern "structure_tests"`; `python build.py guardrails` (zero violations against the real tree)
- Tasks 5.1-5.3: `python build.py test-pattern "stub"`
- Task 6.1: `python build.py test-pattern "http"` then `python build.py integration`
- Task 7.1: `python build.py test-pattern "prompt_presets"`; `python build.py test-pattern "stub"`
- Tasks 8.1-8.6: `python build.py validate-docs`; `python build.py guardrails`; verify `tests/AGENTS.md` regenerated
- Final: `python build.py`

## Failure modes

- Settle poll timeout → `expect_settled` panics with the observed-targets diagnostics (unchanged contract; the unified rule only removes false panics at the cap-window edge).
- Stub options-dock render failure → `.expect` panics the test (fail loud, correct for a test harness).
- Journal: untouched (Task 2.4 dropped); failures stay swallowed by design.
- Validator scan OSError → single `sys.exit(2)` site (unchanged behavior, one location).
- Preset-id collision probability with an 8-hex random suffix: ~2⁻³² per same-millisecond pair; theoretical residual accepted and noted in the fn's doc comment.
- Guardrail marker abuse (suppressing a real risk) → mitigated by the docstring requiring the why inline; the marker is greppable.

## NOT in scope

- The `.pi/extensions/pi-permission-system/config.json` loosening (explicitly ignored per instruction).
- Restoring `retries = 1` (deliberate ticket-12 decision, empirically validated).
- Proptest case-count 256→4, MockBackend string dispatch, guardrail comment-strip limits, visibility waits after settle, string-slice parsing brittleness beyond the dedup, adding `data-id` to preset cards (product-template change).

## Unresolved decisions

None — all three were resolved by the user: full judo for Task 4.1, Task 2.4 dropped, marker exemption for Task 4.3.

## Assumptions

- `uuid` v4 and `askama` are usable from integration test binaries (normal dependencies are visible to test targets).
- Retries stay 0; the config comment records the fail-fast rationale.
- The `describe`/`log_line` rename is internal to `htmx_settle.rs` (grep confirmed only 2 call sites).
- Phase 6 touches test-file imports only; production code changes are limited to the preset-id generator, `add_status_swap_headers` visibility, and the `game_state.rs` comment.
