# Ticket 8 — Execute the browser tier changes

## Summary

Execute ticket 07's target-state design: dissolve the 17 move-down
browser tests (delete 4 files), reorganize the 13 keep-tests into
`behaviour.rs` (6, specced) + `invariants.rs` (7, exempt), write 6 new
HTTP tests covering the 9 gaps 07 identified (7 fragment assertions + 2
I.5 restorations as new `actions.md` scenarios 1.7 + 1.8), write
`docs/specs/browser.md` (6 scenarios 16.1–16.6), amend `STRATEGY.md`
+ `validate_feature_spec.py` to allowlist `tests/browser/behaviour.rs`
for SCENARIO tags. Final spec count: **52 declared, 52 covered** (was
44; +2 actions.md, +6 browser.md). Full suite stays green.

## Key Changes

- **Delete 17 browser tests** across `trigger.rs` (6), `editing.rs` (4
  of 10), `structure.rs` (6 of 13), `interaction.rs` (1). Delete all 4
  source files.
- **Reorganize 13 keep-tests** into 2 new files:
  - `tests/browser/behaviour.rs` — 6 tests (editing + form + status
    wiring), tagged against `docs/specs/browser.md` scenarios 16.1–16.6.
  - `tests/browser/invariants.rs` — 7 tests (CSS/layout), no tags, named
    exemption in STRATEGY.md.
  - `tests/browser/mod.rs` rewired: `mod behaviour; mod invariants;`.
- **6 new HTTP tests** in `tests/http/`:
  - `fragment.rs`: 1 new test (story-log after 2 actions → 4 assertions:
    edit-btn, delete-btn, retry-btn-absent, delete-btn-count ≤2).
  - `fragment.rs`: 1 new test (`/fragment/header` contains "Chronicler
    Engine" within `.game-title`).
  - `index_handler.rs`: 1 new test (`GET /` contains
    `id="connection-status"`).
  - `fragment.rs`: 1 new test (`/fragment/action-area` `<input>` has no
    `required` attribute).
  - `actions.rs`: 2 new tests (S1.7 no-trigger NPC, S1.8 no-refire) with
    SCENARIO tags → `docs/specs/actions.md`.
- **2 new `actions.md` scenarios**: 1.7 (NPC without triggers →
  Narration, `event_header() == None`), 1.8 (repeat action → Narrations
  with `event_header() == Some(...)` ≤ 1). Both HTTP-observable via
  `load_messages()`.
- **New spec** `docs/specs/browser.md`: 6 Given/When/Then scenarios
  (16.1–16.6) for the behaviour tests. No invariant section.
- **Amend `tests/STRATEGY.md`**: SCENARIO tags allowed in `tests/http/`
  **and** `tests/browser/behaviour.rs`. Named exemption for
  `tests/browser/invariants.rs` (no tags, no spec, test code is the
  definition).
- **Amend `validate_feature_spec.py`**: add `tests/browser/` to
  `TEST_DIRS` so `behaviour.rs` SCENARIO tags count as coverage.
  (`invariants.rs` has no tags → contributes nothing.)
- **Dead helper cleanup** (only used by deleted tests): remove
  `element_exists`, `element_count`, `get_status`, `wait_for_log_entries`
  from `tests/test_utils/browser.rs`; remove `wait_for_non_loading_value`
  from `tests/test_utils/wait.rs`. Keep `wait_for_element_children`
  (used by `test_form_stays_static_after_submission` keep-test),
  `count_log_entries` + `send_action` + `wait_for_status_ready` (used by
  keep-tests in `behaviour.rs`).
- **Skip optional partials** (#11 `page_loads`, #14 `action_area_elements`):
  ticket 07 marked optional. Existing HTTP tests already cover these
  partially. Ponytail: skip.

### Finding: no guardrail enforces SCENARIO-tag placement

Ticket 07/08 say "amend the guardrail in `tests/infrastructure/guardrails/`
to allowlist `tests/browser/behaviour.rs`." **No such guardrail exists.**
Grepped `tests/infrastructure/guardrails/` — no rule references `SCENARIO`
or tag placement. Ticket 11's plan already noted this ("isn't implemented
today; rule enforced socially + by pilot's TEST_DIRS scan"). Mechanical
enforcement lives in `validate_feature_spec.py`'s `TEST_DIRS`. This plan
amends that instead. STRATEGY.md text describing a "mechanical guardrail
in `tests/infrastructure/guardrails/`" is corrected to point at
`validate_feature_spec.py`.

## Implementation

### Phase 1: New HTTP coverage + spec scenarios (5 SP)

Coverage lands BEFORE browser deletions so there's no gap.

- [ ] #### Task 1.1: Add 2 new scenarios to `docs/specs/actions.md` (1 SP)
  - **1.7**: Given NPC `"bartender"` in room with `triggers: vec![]`, And
    quantifier returns `["bartender"]`, And narrator returns dialogue.
    When `POST /action "talk to bartender"`, And pipeline idle. Then
    `load_messages()` contains ≥1 `Narration`, And every `Narration`
    has `event_header() == None`.
  - **1.8**: Given NPC `"shopkeeper"` with `times_met == 0` trigger, And
    quantifier returns `["shopkeeper"]`. When `POST /action "talk to
    shopkeeper"`, And idle, And `POST /action "talk to shopkeeper"`,
    And idle. Then `load_messages()` `Narration` entries with
    `event_header() == Some(...)` count ≤ 1.
  - Format: Given/When/Then/And lines end with exactly 2 trailing spaces
    (hard line break, per `validate_feature_spec.py` format check).
  - IDs 1.7, 1.8 — don't reuse 1.3 (dropped; ticket 11 rule "dropped
    scenarios don't get reused").

- [ ] #### Task 1.2: 2 new I.5 HTTP tests in `tests/http/actions.rs` (2 SP)
  - [ ] ##### SubTask 1.2.1: `test_no_trigger_npc_produces_narration_no_event_http` (S1.7) (1 SP)
    - Pattern from `test_quantifier_npc_fires_trigger_http` (line 88):
      `NpcCard` with `triggers: vec![]`, `TestDataBuilder.default_test().npcs(...)`,
      `make_test_pipeline_with_mock_quantifier`, narrator returns
      "The bartender nods.". `POST /action "talk to bartender"` →
      `wait_idle` → assert ≥1 Narration, all `event_header() == None`.
  - [ ] ##### SubTask 1.2.2: `test_trigger_does_not_refire_on_second_encounter_http` (S1.8) (1 SP)
    - `NpcCard` "shopkeeper" with `TriggerRequirement::Eq(0)` trigger.
      Quantifier returns `["shopkeeper"]` both calls. `POST "talk to
      shopkeeper"` ×2 with `wait_idle` between. Assert: count of
      `Narration` entries with `event_header().is_some()` ≤ 1 (semantic
      check — trigger continuation sets `event_header`, main narration
      doesn't; matches S1.4 pattern at actions.rs:140). Do NOT assert on
      `narration_prompt` substring — that's the narrator input, not the
      stored text.

- [ ] #### Task 1.3: 4 new fragment/index HTTP tests (2 SP)
  - [ ] ##### SubTask 1.3.1: `test_story_log_fragment_renders_action_buttons_after_actions` in `fragment.rs` (1 SP)
    - Pre-check: `grep -rn "retry-btn" chronicler_engine/src/` — confirm it
      only appears as `class=\"retry-btn\"` (or not at all) in the story-log
      template. If non-class contexts exist (CSS, JS, comments), use
      `class=\"retry-btn` substring instead of bare `retry-btn`.
    - Setup: `app_with_narrator(Arc::new(MockBackend::default().with_narrations(vec!["First.".into(), "Second.".into()])))`
      → `(app, state)`. `post_action(&app, "first").await` +
      `wait_idle(&state, 1000).await` + `post_action(&app, "second").await`
      + `wait_idle(&state, 1000).await`. Then
      `let body = fetch_body(app.clone(), "/fragment/story-log").await;`.
    - 4 assertions on `body`: contains `edit-btn`, contains `delete-btn`,
      does NOT contain `retry-btn`, And `delete-btn` substring count ≤ 2.
    - Do NOT use `TestAppBuilder.log(...)` — that seeds storage directly,
      bypassing the pipeline; template rendering of edit/delete buttons
      may differ for seeded vs. pipeline-produced entries.
  - [ ] ##### SubTask 1.3.2: 3 GET-only fragment/index tests (1 SP)
    - `test_header_fragment_displays_game_title` in `fragment.rs`:
      `fetch_body("/fragment/header")` contains "Chronicler Engine"
      (within `.game-title` — check substring "game-title" present And
      "Chronicler Engine" present).
    - `test_index_page_has_connection_status` in `index_handler.rs`:
      `GET /` body contains `id="connection-status"`.
    - `test_action_area_input_has_no_required` in `fragment.rs`:
      `fetch_body("/fragment/action-area")` contains `<input` And does
      NOT contain `required` (or: the `<input` substring is not
      followed by `required` — simple substring check suffices since
      template has one input).

### Phase 2: Reorganize keep-tests into new files (3.5 SP)

- [ ] #### Task 2.0: Enumerate keep-test dependencies + resolve constants (0.5 SP)
  - Grep each keep-test body for helper names. Produce a keep-list:
    - `with_test_page`, `count_log_entries`, `send_action`, `wait_for_status_ready`,
      `wait_for_element_children` (browser.rs) — keep.
    - `wait_for_element_exists`, `wait_for_element_not_exists`,
      `wait_for_element_persist` (wait.rs) — keep.
    - `wait.rs` line ~227 local `element_count` (internal to
      `wait_for_element_children`) — keep. Only `browser.rs::element_count`
      is removed in Phase 3.
  - Resolve `CONFIG_PATH` / `TEST_WORLD` / `TEST_PERSONA`: redefine locally
    in each new file (matches editing.rs/structure.rs majority pattern;
    least churn). Do NOT hoist to `test_utils/mod.rs`.
  - Output: a per-file import + const block spec for Phase 2.1/2.2 to copy.

- [ ] #### Task 2.1: Create `tests/browser/behaviour.rs` (2 SP)
  - Move 6 tests from `editing.rs` (3 keep) + `structure.rs` (1 keep:
    `test_form_stays_static_after_submission`) + `editing.rs` (2 keep:
    `test_delete_removes_message`, `test_status_updates_during_generation`).
    Wait — recount: 6 behaviour tests are:
    - `test_edit_mode_activates_on_click` (editing.rs)
    - `test_edit_cancel_restores_original` (editing.rs)
    - `test_polling_pauses_during_edit` (editing.rs)
    - `test_delete_removes_message` (editing.rs)
    - `test_edit_textarea_matches_original_height` (editing.rs) — **NO**,
      this is an invariant (CSS height). Goes to `invariants.rs`.
    - `test_form_stays_static_after_submission` (structure.rs)
    - `test_status_updates_during_generation` (editing.rs)
  - Corrected: behaviour.rs = 5 from editing.rs + 1 from structure.rs =
    6. `test_edit_textarea_matches_original_height` → invariants.rs.
  - Add `// [chronicler_engine/docs/specs/browser.md] SCENARIO: 16.x`
    tags above each `#[tokio::test]`.
  - Imports: `with_test_page`, `count_log_entries`, `send_action`,
    `wait_for_status_ready`, `wait_for_element_children` from
    `crate::test_utils::browser`; `wait_for_element_exists`,
    `wait_for_element_not_exists`, `wait_for_element_persist` from
    `crate::test_utils::wait`; `expect` from `playwright_rs`; `Duration`
    from `std::time`.

- [ ] #### Task 2.2: Create `tests/browser/invariants.rs` (1 SP)
  - Move 7 tests: 6 from `structure.rs` (`test_story_log_scrollable`,
    `test_no_horizontal_overflow`, `test_log_entry_text_wraps_within_bubble`,
    `test_element_positioning`, `test_npc_portraits_horizontal_layout`,
    `test_npc_portraits_fixed_width`) + 1 from `editing.rs`
    (`test_edit_textarea_matches_original_height`).
  - No SCENARIO tags. `use super::*;` (matches structure.rs pattern).
  - Add `//! Rendering invariants (named exemption in STRATEGY.md): no
    //! spec link, test code is the definition.` module doc.

- [ ] #### Task 2.3: Rewire `tests/browser/mod.rs` (0 SP — bundled with 2.1/2.2)
  - Replace `mod editing; mod interaction; mod structure; mod trigger;`
    with `mod behaviour; mod invariants;`. Update module doc.

### Phase 3: Delete old browser files + dead helpers (1 SP)

- [ ] #### Task 3.1: Delete 4 browser files (0.5 SP)
  - `tests/browser/trigger.rs`, `editing.rs`, `structure.rs`,
    `interaction.rs` — all 17 move-down tests gone with them. (Keep-tests
    already moved in Phase 2.)

- [ ] #### Task 3.2: Remove dead helpers (0.5 SP)
  - From `tests/test_utils/browser.rs`: remove `element_exists` (line
    ~152), `element_count` (line ~163), `get_status` (line ~143),
    `wait_for_log_entries` (line ~182). Verify no remaining callers via
    `grep` first.
  - From `tests/test_utils/wait.rs`: remove `wait_for_non_loading_value`
    (line ~38) + its `capture_failure_state` call if unique to it.
  - Keep `wait_for_element_children` (used by
    `test_form_stays_static_after_submission`), `count_log_entries` +
    `send_action` + `wait_for_status_ready` (used by keep-tests).
  - If `#[allow(unused_imports)]` markers become unnecessary after
    cleanup, remove them.

### Phase 4: Browser spec + STRATEGY.md + validator (3 SP)

- [ ] #### Task 4.1: Write `docs/specs/browser.md` (2 SP)
  - Pre-check: `grep -rn "^#### Scenario 16" chronicler_engine/docs/specs/` must
    return nothing. If non-empty, use 17.x or next free range.
  - 6 scenarios (16.1–16.6), one per behaviour test, Given/When/Then
    format with hard line breaks (2 trailing spaces):
    - 16.1: click `.edit-btn` → `#edit-textarea` appears
    - 16.2: edit → modify → cancel → original text restored
    - 16.3: edit mode → `#edit-textarea` persists 3s (polling pause)
    - 16.4: click `.delete-btn` (confirm) → log-entry count decreases
    - 16.5: submit form → `#command-form` id unchanged (static shell)
    - 16.6: send action → `#status-display` shows generating state
  - No rendering-invariant section (those live as test code in
    `invariants.rs`).
  - Intro: "Endpoint: browser DOM. Behavioural authority for the 6
    browser-only interactions. Each When is a browser action; each Then
    is a DOM-observable outcome."

- [ ] #### Task 4.2: Amend `tests/STRATEGY.md` (0.5 SP)
  - SCENARIO tags section: tags allowed in `tests/http/` **and**
    `tests/browser/behaviour.rs` only.
  - Add named-exemption line: `tests/browser/invariants.rs` carries no
    tags, no spec link — test code is the definition (same shape as
    unit branch tests).
  - Correct "mechanical guardrail in `tests/infrastructure/guardrails/`"
    → "mechanical enforcement in `scripts/validate_feature_spec.py`
    via `TEST_DIRS`".

- [ ] #### Task 4.3: Amend `scripts/validate_feature_spec.py` `TEST_DIRS` (0.5 SP)
  - Add `ENGINE_ROOT / "tests" / "browser"` to `TEST_DIRS` list.
  - `invariants.rs` has no `// SCENARIO:` comments → contributes
    nothing. `behaviour.rs` tags count as coverage for `browser.md`.
  - Update the comment above `TEST_DIRS` to note browser tags allowed
    in `behaviour.rs` only (social rule; validator scans whole dir).

### Phase 5: Verify (1 SP)

- [ ] #### Task 5.1: Run validators + test suites (1 SP)
  - `python3 chronicler_engine/scripts/validate_feature_spec.py` →
    expect "52 declared, 52 covered, 0 gap(s), 0 orphan(s), 0 format
    violation(s)".
  - `cargo nextest run -p chronicler_engine --test guardrails` → green.
  - `cargo nextest run -p chronicler_engine --test http` → green (was
    1369 pass baseline; +6 new tests = 1375 expected, modulo
    retractions).
  - `cargo nextest run -p chronicler_engine --test browser` → 13 tests
    pass (6 behaviour + 7 invariants). Browser tests require Chrome;
    skip if unavailable, but compile must succeed.
  - `cargo nextest run -p chronicler_engine` → full suite green.
  - `cargo build -p chronicler_engine --tests` → no dead-code warnings
    for removed helpers.

## Test Plan

- **Spec pilot**: 52 declared (15 actions.md [was 13, +1.7 +1.8] + 2
  reset.md + 3 story_log.md + 17 swipe_new.md + 9 retrigger.md + 6
  browser.md), 52 covered, 0 gaps, 0 orphans, 0 format violations.
- **Browser tier**: 13 tests in 2 files (6 behaviour + 7 invariants).
  4 old files deleted. No test count change for keep-tests (just moved).
- **HTTP E2E**: 6 new tests covering 7 fragment assertions + 2 I.5
  scenarios. All 9 required coverages from ticket 07 landed.
- **Guardrails**: SCENARIO-tag placement rule holds (tags only in
  `tests/http/` + `tests/browser/behaviour.rs`).
- **Dead code**: no warnings for removed helpers.

## Per Task/Sub Task Validation Steps

- Task 1.1: `grep -n "Scenario 1.7\|Scenario 1.8"
  chronicler_engine/docs/specs/actions.md` shows both; format check
  passes (2 trailing spaces on Given/When/Then/And lines).
- Task 1.2.1: `test_no_trigger_npc_produces_narration_no_event_http`
  passes; asserts all Narrations have `event_header() == None`.
- Task 1.2.2: `test_trigger_does_not_refire_on_second_encounter_http`
  passes; asserts Narrations with `event_header().is_some()` ≤ 1.
- Task 1.3.1: story-log fragment test passes; 4 assertions (edit-btn,
  delete-btn, retry-btn-absent, delete-btn-count ≤2) in one test.
- Task 1.3.2: 3 GET-only tests pass; each asserts one fragment/index
  property.
- Task 2.1: `tests/browser/behaviour.rs` has 6 `#[tokio::test]` fns,
  each with `// [chronicler_engine/docs/specs/browser.md] SCENARIO:
  16.x` tag; `cargo nextest run --test browser behaviour` passes (or
  compiles if Chrome unavailable).
- Task 2.2: `tests/browser/invariants.rs` has 7 `#[tokio::test]` fns,
  no SCENARIO tags; module doc names the exemption.
- Task 2.3: `tests/browser/mod.rs` has `mod behaviour; mod invariants;`
  only; no `mod editing/interaction/structure/trigger`.
- Task 3.1: `ls chronicler_engine/tests/browser/` shows `behaviour.rs`,
  `invariants.rs`, `mod.rs` only.
- Task 3.2: `grep -rn "element_exists\|get_status\|wait_for_log_entries\|wait_for_non_loading_value"
  chronicler_engine/tests/` → no matches outside the validator grep
  itself; `cargo build --tests` clean.
- Task 4.1: `ls chronicler_engine/docs/specs/browser.md` exists; 6
  `#### Scenario 16.x` headings; format check passes.
- Task 4.2: `grep -n "behaviour.rs\|invariants.rs"
  chronicler_engine/tests/STRATEGY.md` shows both amendments.
- Task 4.3: `grep -n "browser" chronicler_engine/scripts/validate_feature_spec.py`
  shows `tests/browser` in `TEST_DIRS`.
- Task 5.1: all commands green with expected counts.

## Assumptions

- **Scenario IDs**: 1.7, 1.8 in actions.md (1.3 stays dropped per ticket
  11 rule). 16.1–16.6 in browser.md (next free range; no collision with
  existing 1.x–15.x). Pilot dedups by ID across `docs/specs/*.md`.
- **No guardrail enforces SCENARIO placement** (confirmed by grep).
  Mechanical enforcement is `validate_feature_spec.py` `TEST_DIRS`.
  Plan amends that, not a non-existent guardrail. STRATEGY.md text
  corrected to match reality.
- **`tests/browser/` added to `TEST_DIRS`** scans `invariants.rs` too,
  but it has no `// SCENARIO:` comments → contributes nothing. The
  "tags only in behaviour.rs" rule stays social (STRATEGY.md text).
  Adding per-file logic to the validator is more code than the leak
  warrants.
- **6 new HTTP tests, not 9** — ticket acceptance said "9 new HTTP
  tests (7 fragment + 2 I.5)" but the 7 fragment assertions group
  naturally by setup/endpoint into 4 tests (1 story-log after 2 actions
  + 3 GET-only on different endpoints). All 7 assertions + 2 I.5
  scenarios covered. One-assertion-per-test would duplicate setup
  boilerplate without adding clarity. User confirmed granularity
  depends on test shape.
- **Skip optional partials** (#11 `page_loads`, #14 `action_area_elements`):
  ticket 07 marked optional. Existing HTTP tests cover partially.
  Ponytail: skip.
- **5 already-covered move-downs** (#1, 2, 5, 16, 17) + **1 S1.3
  conflict** (#6): delete only, no replacement. Per ticket 07.
- **`test_edit_textarea_matches_original_height`** is an invariant
  (CSS height measurement), not behaviour — goes to `invariants.rs`
  despite originating in `editing.rs`. Ticket 07's file split is by
  assertion shape, not source file.
- **Browser tests require Chrome** — if unavailable in CI, compile must
  still succeed. `cargo build --tests` is the minimum bar; `cargo
  nextest run --test browser` is nice-to-have.
- **No nextest config change** in this ticket (ticket 09 owns that).
  Browser test count drops 30 → 13; nextest config unaffected.
- **Dead helper removal** keeps the build warning-free. If removal
  breaks an unexpected caller, restore the helper + `#[allow(dead_code)]`
  rather than churn callers.
