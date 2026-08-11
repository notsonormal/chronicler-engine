# Move retry tests to HTTP E2E and update the spec

Type: task (HITL)
Status: resolved
Blocked by: 01

## Question

Move the retry flow tests from the component tier to HTTP E2E
(`tests/http/`), updating the retry spec (`docs/specs/retry.md`) as a
living document alongside the tests. The retry spec was written as a
planning artifact — execution will stress-test it, and scenarios may need
revision when they meet the real code (especially after ticket 01's code
changes).

### Tests to move (from the diff asset — "keep (flow)" classification)

Per the [diff asset](../../test-strategy/assets/pipeline-suite-diff.md):

**flow/retry_main.rs (10 tests):**
- `test_retry_main_narration_applies_new_quantifier_result` — retry
  re-runs quantifier → player moves (R1.2)
- `test_retry_with_different_narration_text_reruns_quantifier` — per-call
  narration sequence on retry
- `test_double_retry_increments_swipe_and_reruns_quantifier` — double-retry
  swipe increment (I.5)
- `test_retry_preserves_input_and_does_not_create_extra_swipe` — input
  integrity across retry (I.2, I.4)
- `test_retry_after_edited_input_uses_new_text` — edit-then-retry (R1.4)
- `test_main_retry_reevaluates_triggers` — retry re-evaluates triggers
  (R1.5)
- `test_retry_completes_when_quantifier_returns_none` — quantifier-None
  completion
- `test_retry_no_pre_main_snapshot` — **drift**, align to spec (R4.3/R4.4:
  Error)
- `test_movement_with_arrival_narration_retry` — movement + retry
- `test_retry_appends_swipe_to_existing_narration` — swipe append (I.5)

**flow/retry_event.rs (3 tests):**
- `test_event_retry_does_not_create_extra_swipe_on_narration` — event
  retry → exactly 2 narrations (R2.1)
- `test_retry_event_continuation_preserves_quantifier_result` — event
  retry does not re-run quantifier (R2.2)
- `test_trigger_continuation_runs_quantifier_and_detects_new_npc` —
  trigger continuation re-runs quantifier, detects new NPC

### Notes from ticket 11 (spec restructure)

Ticket 11 dissolved `action_pipeline.md` + `flow.md` into three
endpoint-named specs and rewrote `tests/STRATEGY.md` + the pilot. Retry
work (ticket 05) inherits the following:

- **Reuse `tests/http/test_helpers.rs`** — `post_action`, `post_empty`,
  `wait_idle`, `app_with_narrator` already live. Do NOT re-implement. If
  retry tests need a custom-quantifier helper (likely — retry re-runs
  the quantifier), add `app_with_narrator_and_quantifier` back to
  `test_helpers.rs` as a shared helper (it was inlined into `actions.rs`
  during ticket 11 but is reusable here).
- **Spec format rule (pilot-enforced).** `validate_feature_spec.py` now
  checks every `**Given**`/`**When**`/`**Then**`/`**And**` line ends with
  exactly two trailing spaces (markdown hard line break) and no blank
  line separates consecutive keyword lines within a scenario. New or
  revised `retry.md` scenarios MUST comply.
- **Shutdown → 503 is HTTP-observable.** Ticket 11 made the `ShuttingDown`
  arm live in `retry()` + `retrigger()` (top-of-function guard). Whether
  this earns a retry spec scenario (R?.x) or stays at the handler unit
  tier (ticket 04 covers it) is a decision for ticket 05. Recommend:
  handler unit tier (ticket 04) — 503 on shutdown is handler behaviour,
  not retry-domain behaviour.
- **S2.4 pattern.** Ticket 11 used `with_trigger_narration_fail()` for
  trigger-only failure (main preserved + trigger fails + System log +
  Error status). Retry scenarios with similar trigger-only failure must
  use the same backend.
- **`retry.md` ID scheme.** Retry spec uses `R1.x`–`R5.x` IDs (not `N.M`).
  The pilot's `SCENARIO_RE` only matches `\d+.\d+`, so `retry.md` scenarios
  are currently NOT pilot-tracked. This is pre-existing. If ticket 05
  wants pilot tracking for retry scenarios, the regex needs widening —
  flag for a separate decision.

### Workflow

Same as ticket 03: for each component test, derive the HTTP-framed spec
scenario, write the E2E test from it, update the spec if the real
behaviour doesn't match what the spec says.

### Spec updates

The retry spec may need revision after the code changes (ticket 01):
- Scenarios R4.3/R4.4 (no-snapshot → 500) — verify the pre-spawn check
  matches the spec's described response.
- Scenarios R5.3/R5.4 (concurrent generation → "Still thinking…") —
  verify the generation gate produces the spec'd response.
- Scenario R4.8 (System log persistence) — verify the System message is
  persisted after the fix.
- Any scenario that doesn't hold up against the real code gets revised.

### Acceptance

- Every retry flow test has an HTTP E2E equivalent.
- `docs/specs/retry.md` updated to match the real behaviour.
- `flow/retry_main.rs` and `flow/retry_event.rs` deleted after tests are
  moved.
- SCENARIO/INVARIANT tags on the new E2E tests match the spec.
- Suite green.

## Answer

All acceptance criteria met. Suite green: 1369 pass, 2 skipped.
Pilot: 44 declared, 44 covered, 0 gaps, 0 orphans, 0 format violations.
Guardrails: 101/101 pass.

**Spec split** (per ticket 11's endpoint-naming rule, settled during
plan review):
- `docs/specs/retry.md` → **deleted**.
- `docs/specs/swipe_new.md` — 17 scenarios (9.1–9.6 main retry, 10.1–10.2
event retry, 11.1–11.8 retry errors, 12.1 concurrency). Invariants I.1,
I.2, I.4–I.8.
- `docs/specs/retrigger.md` — 9 scenarios (13.1 new event + no rollback,
13.2 no quantifier rerun, 14.1–14.6 errors, 15.1 concurrency).
  Invariants I.3, I.7, I.8, I.9.
- R5.1/R5.2 (cancellation mid-flight) **dropped from specs** —
  unit-only (needs CancellationToken); already covered by `retry_tests.rs`.
- IDs continue the cross-spec N.M sequence (actions 1.x–6.x, reset 7.x,
  story_log 8.x, swipe_new 9.x–12.x, retrigger 13.x–15.x). Pilot regex
  unchanged (user declined widening for R-prefix).
- `docs/specs/actions.md` — added S1.6 (trigger continuation re-runs
  quantifier, detects new NPC) as a regression guard for the deleted
  component-tier test `test_trigger_continuation_runs_quantifier_and_detects_new_npc`.

**Tests:**
- `tests/http/swipe_new.rs` — 17 tests (14 from-scratch + 3 moved from
  `fragment.rs`). SCENARIO-tagged 9.1–12.1.
- `tests/http/retrigger.rs` — 9 tests (6 from-scratch + 3 moved from
  `fragment.rs`). SCENARIO-tagged 13.1–15.1.
- `tests/http/actions.rs` — added S1.6 test
  (`test_trigger_continuation_reruns_quantifier_detects_new_npc_http`).
- `tests/http/fragment.rs` — removed 6 retry/retrigger tests (moved to
  new files, retagged). Removed unused `ProcessActionResult` import.
- `tests/http/test_helpers.rs` — added `app_with_narrator_and_quantifier`
  (one helper only; NPC-trigger tests use inline `TestAppBuilder` pattern
  from actions.rs 1.4).
- `tests/http/mod.rs` — wired `mod swipe_new; mod retrigger;`.

**Component tier dissolved:**
- `tests/integration/flow/retry_main.rs` (10 tests) — **deleted**.
- `tests/integration/flow/retry_event.rs` (3 tests) — **deleted**.
- `tests/integration/mod.rs` — removed `mod flow_retry_main;` +
  `mod flow_retry_event;`.

**Plan-review findings applied:**
1. S1.6 kept (regression guard for deleted component test).
2. 6 existing `fragment.rs` tests moved+retagged (not duplicated).
   Concurrency tests (12.1, 15.1) kept the deterministic `try_claim`
   pre-claim pattern, NOT `with_delay` (no flake).
3. One helper only (`app_with_narrator_and_quantifier`); NPC-trigger
   tests use inline pattern.

**Spec corrections during implementation:**
- 14.5 (retrigger trigger failure) — spec updated to mention the System
  log entry that the error path persists (matches real code).
- 9.5 trigger evaluation — `evaluate_triggers` checks all NPCs (not
  `scene.npcs_in_area`); trigger fires based on `trigger.room_id` matching
  `current_room_id`.

Net test count: 1361 baseline + 21 new-from-scratch (14 swipe_new + 6
retrigger + 1 S1.6) - 13 deleted component-tier = 1369. 6 moved tests
are net-neutral.

## Review (two-axis code-review skill)

**Standards:** 6 actionable (1 hard, 5 smells). 1 suppressed by plan
(#3 pipeline-boilerplate DRY vs. plan's "one helper only" decision). 1
pre-existing (#7 code-indexer in specs → chronicler-docs-hygiene).

Actionable:
1. **Hard** — `tests/http/test_helpers.rs` `app_with_narrator_and_quantifier`
   doc-comment says "Used by retry E2E tests". Self-referential. Drop that
   line (AGENTS.md §"No Self-Referential Comments").
2. **Duplicated Code** — `tests/http/actions.rs:176-228` (S1.6) +
   `tests/http/swipe_new.rs:183-215` (S9.5) hand-roll `NpcCard`/`CharacterSheet`.
   `src/test_support/fixtures.rs:84-160` exports `TestNpc::with_times_met_trigger`
   (matches S1.6) + `TestNpc::with_room_scoped_trigger` (matches S9.5). Reuse.
3. **Mysterious Name** — `swipe_new.rs:298` `seed_event_flow(app: &AppState)`:
   param named `app`, is `AppState`, called as `&state`. Rename `app`→`state`.
4. **Dead binding** — `retrigger.rs:170-174` (S14.5) `let _n = messages_before.len()`
   never read. Drop both lines.
5. **Restated-code comments** — `swipe_new.rs`/`retrigger.rs` use `// ----`
   box dividers + `// Scenarios X.Y–Z` section headers duplicating per-test
   `// [doc] SCENARIO: X.Y` markers. Existing `tests/http/{actions,fragment}.rs`
   don't. Remove dividers.

**Spec:** 3 real findings (P3 was false positive — `mod pipeline_retry`
removal is ticket 04, not ticket 05; scoping artifact from mixed
`tests/integration/mod.rs` hunks).

1. **P1** — 10.1 test `test_event_retry_replaces_event_narration_with_new_swipe_http`
   missing active-swipe assertion. Spec 10.1 Then: "event narration message
   has its new swipe as the active swipe". Test only asserts `swipes.len() >= 2`.
   Fix: assert new swipe is active (index/text), as 9.1 does.
2. **P2** — 11.7 test `test_retry_room_not_found_sets_error_http` tests wrong
   code path. Spec 11.7 Given: "last message is a non-event Narration"; test
   seeds only Input. Spec 11.7 Then: "original narration's swipes unchanged";
   test asserts neither. Fix: seed a Narration, assert swipes unchanged.
3. **10.1 `>= 2` should be `== 2`** — Invariant I.5 says "each retry appends
   exactly one swipe". `>= 2` masks double-append regression. 9.1 uses `== 2`.

**Status:** ALL FIXES APPLIED. Suite green: 1369 pass, 2 skipped.
Pilot: 44/44. Guardrails: 101/101. Build clean (0 warnings).

Applied:
- #1 (hard): dropped self-referential "Used by retry E2E tests"
  line from `app_with_narrator_and_quantifier` doc-comment.
- #2 (NpcCard reuse): S1.6 (`actions.rs`) + S9.5 (`swipe_new.rs`)
  now use `TestNpc::with_times_met_trigger` / `with_room_scoped_trigger` /
  `named` from `src/test_support/fixtures.rs`.
- #3 (mysterious name): `seed_event_flow` param `app` → `state`,
  local `state` → `gs` (4 bindings).
- #4 (dead binding): dropped `let _n = messages_before.len()` in
  retrigger S14.5.
- #5 (restated-code comments): removed 5 box-divider blocks from
  `swipe_new.rs` + 3 from `retrigger.rs`.
- Spec P1: 10.1 test now asserts `active_swipe_index == 1` +
  `text() != "Event narration"` (active swipe is the new one).
- Spec P2: 11.7 test now seeds Input + non-event Narration (last
  message) per spec Given; asserts Narration swipes unchanged (== 1)
  after room-not-found error.
- Spec 10.1 `>=2` → `==2` per Invariant I.5.

**Plan deviation (L8):** 13.2 kept as a separate test (3 tests total
for 13.x: 13.1, 13.2, 13.3) instead of folding 13.2 into 13.1 per
plan Task 4.1. Separate tests give better coverage isolation — 13.1
asserts N+1 append, 13.2 asserts room unchanged, 13.3 asserts room
unchanged (distinct from 13.2's quantifier-not-rerun focus). Decision
recorded post-review; plan decision was conservative, impl is stricter.
