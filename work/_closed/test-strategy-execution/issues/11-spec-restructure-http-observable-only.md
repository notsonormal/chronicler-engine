# Spec restructure: HTTP-observable, endpoint-named

Type: task (HITL)
Status: closed

## Question

Restructure the action-pipeline + flow specs so every spec is
**HTTP-observable**, **endpoint-named**, and **covered by an HTTP E2E test**
in `tests/http/`. Today the work is split across two misfit files:

- `docs/specs/action_pipeline.md` — component-named ("pipeline"), pipeline-
  framed Given/When/Then (`execute_action_impl returns Ok(...)`), 18
  scenarios S1.1–S5.2 (pilot-tracked via `#### Scenario` headings) +
  invariant bullets I.1–I.5 (not pilot-tracked — pilot only sees
  `Scenario N.N` headings). Covered by **unit** tests in
  `src/application/pipeline/pipeline_tests.rs` via `src/` SCENARIO tags —
  not by HTTP E2E. Violates STRATEGY.md ("Every spec scenario maps to at
  least one HTTP E2E test").
- `docs/specs/flow.md` — component-tier name ("flow", from the dissolved
  `tests/integration/flow/` directory). HTTP-framed but mixes three
  endpoints under one invented concept. Covered by `tests/http/flow_sequence.rs`
  (also mis-named).

This ticket delivers the full HTTP-observable restructure atomically: no
mixed-framing interim state, endpoint-named specs + test files, HTTP E2E
coverage for every surviving scenario.

### Decision context (from ticket 03 planning + review)

- Specs are the behavioural authority for **HTTP-observable** behaviour
  only. Unit tests cover branch coverage (internal state, cancellation,
  mid-flight timing, call sequencing) and do not need spec scenarios.
  Driven-adapter tests cover the storage seam. Mixing tiers in one spec
  muddies what the spec is for.
- Spec/test names follow the **endpoint**, not the internal component.
  `action_pipeline` and `flow` both leak component-tier names; the
  endpoint is `POST /action`, `POST /reset`, `POST /history/delete`.
- HTTP-observable scenarios (S1.x, S2.x, S3.3, S3.4-success, S6.x) must
  have HTTP E2E tests with SCENARIO tags in `tests/http/` — not `src/`.
  This was missed when S1–S5 reframe was split from ticket 03 to here.
- `tests/STRATEGY.md` is the authority: "SCENARIO: tags go on HTTP E2E
  tests in `tests/http/`. For scenarios that live at the unit tier
  (cancellation, internal state, mid-flight, call sequencing), the tag
  goes on the unit test in `src/`." S1.x/S2.x/S3.3/S3.4 are HTTP-observable
  → tags belong in `tests/http/`, not `src/`.

## Target structure

Three specs, three test files, one per endpoint:

| Spec | Test file | Scenarios | Endpoint |
|---|---|---|---|
| `docs/specs/actions.md` | `tests/http/actions.rs` (replace existing) | S1.1–S1.5, S2.1–S2.4, S3.3, S3.4-success, S6.1–S6.3 | `POST /action` |
| `docs/specs/reset.md` | `tests/http/reset.rs` (new) | S7.1, S7.2 | `POST /reset` |
| `docs/specs/story_log.md` | `tests/http/story_log.rs` (new) | S8.1, S8.2, S8.3 | `POST /history/delete` |

**Dissolved:**
- `docs/specs/action_pipeline.md` → renamed/replaced by `actions.md`
  (S1–S5 reframed) + S6.x moves in from `flow.md`
- `docs/specs/flow.md` → split across `actions.md` (S6), `reset.md` (S7),
  `story_log.md` (S8); file deleted
- `tests/http/flow_sequence.rs` → S6 tests join existing `tests/http/actions.rs`;
  S7 tests move to new `tests/http/reset.rs`; S8 tests move to new
  `tests/http/story_log.rs`; file deleted

## Work

1. **Reframe HTTP-observable scenarios to endpoint Given/When/Then** in
   `docs/specs/actions.md` (renamed from `action_pipeline.md`):
   - S1.1, S1.2, S1.3, S1.4, S1.5 (normal flow — observable via
     `load_messages()` + status endpoint)
   - S2.1, S2.2, S2.3, S2.4 (error recovery — observable via
     `load_messages()` + status; S2.4 System log observable)
   - S3.3 (delayed LLM — observable via `load_messages()`)
   - S3.4 success case (phase reset — observable via `load_messages()`)
   - S6.1, S6.2, S6.3 (multi-action sequences — moved from `flow.md`)
   - Drop `execute_action_impl returns Ok(...)` framing; replace with
     `POST /action` → `wait_idle` → assert on `load_messages()` /
     status endpoint / story-log fragment.

2. **Split `docs/specs/flow.md` by endpoint**:
   - S6.x → `docs/specs/actions.md`
   - S7.x → new `docs/specs/reset.md`
   - S8.x → new `docs/specs/story_log.md`
   - Delete `docs/specs/flow.md`.

3. **Remove non-HTTP-observable scenarios** from the spec entirely:
   - S3.1 (`last_trigger` field — internal state → unit only)
   - S3.2 (mid-flight streaming timing — sync flags → unit only)
   - S4.1, S4.2, S4.3 (cancellation — `CancellationToken` → unit only)
   - S5.1, S5.2 (snapshots — not HTTP-observable; no endpoint exposes
     `snapshot.db_id`. **No `snapshots.md` created** — snapshots leave
     the spec entirely.)
   - I.5 (trigger continuation call sequencing — call-count assertion →
     unit only; I.5 is an invariant bullet, not a `#### Scenario`
     heading, so the pilot does not track it — but it still leaves the
     spec text)
   - **I.1–I.4 stay** as spec invariants in `actions.md` — they are
     HTTP-observable (I.1: status Idle/Error after action; I.2: Input
     before Narration in `load_messages()`; I.3: one Narration per call;
     I.4: no empty Narration persisted). They remain as invariant
     bullets, not pilot-tracked.

4. **Replace the existing `tests/http/actions.rs`** (currently 8
   failure-path tests, no SCENARIO tags) with new S1–S6 SCENARIO-tagged
   HTTP E2E tests:
   - Delete all 8 existing failure-path tests (see step 4a for where
     their coverage goes).
   - Add HTTP E2E tests for the reframed scenarios that currently have
     no HTTP coverage: S1.1, S1.2, S1.3, S1.4, S1.5, S2.1, S2.2, S2.3,
     S2.4, S3.3, S3.4-success.
   - Add S6.1, S6.2, S6.3 (moved from `flow_sequence.rs`).
   - Each test: `POST /action` → `wait_idle` → assert on
     `message_service.load_messages()` and/or status. SCENARIO tag points
     at `docs/specs/actions.md`.

4a. **Of the 8 HTTP tests deleted in step 4, 5 need no replacement and
   3 are replaced by new unit tests** in
   `src/adapters/driving/http/action/handlers/actions_tests.rs`.

   No replacement needed (5 deleted tests):
   - `test_action_check_handler_empty_command` — already covered by the
     unit test of the same name in `actions_tests.rs`.
   - `test_action_handler_special_characters` — form deserialization,
     already covered by unit `test_action_form_with_special_characters`.
   - `test_action_handler_load_state_failure_graceful_degradation`,
     `test_action_confirm_handler_load_state_failure_graceful_degradation`,
     `test_action_handler_load_messages_failure` — these hit the
     `Ok(Started)` branch (pipeline swallows storage errors and returns
     `Started`). Handler `Ok(Started)` branch is already covered by unit
     `test_action_handler_started`. The pipeline's error-swallowing is a
     pipeline-tier concern, not a handler concern — no handler test
     needed.

   New unit tests (3 tests, covering 3 `dispatch_action` branches not
   currently covered at unit):
   - `Err(e)` branch → 500 (was indirectly hit by 3 deleted HTTP tests:
     `test_action_handler_snapshot_save_failure`,
     `test_action_handler_message_insert_failure`,
     `test_action_confirm_snapshot_save_failure`)
   - `Ok(ShuttingDown)` branch → 503 (not previously covered)
   - `Ok(ConcurrentGeneration)` branch → 200 "Still thinking..." (not
     previously covered)
   Inject branch outcomes directly via a pipeline stub/fake that
   returns the desired `ProcessActionResult` — do NOT inject via
   storage `with_failure` (that tests pipeline swallowing, not handler
   dispatch).

   Net: -8 HTTP tests, +3 unit tests; handler branch coverage complete
   and properly tiered.

5. **Move S7 tests from `flow_sequence.rs` to new `tests/http/reset.rs`**;
   move S8 tests to new `tests/http/story_log.rs`. Update SCENARIO tag
   paths:
   - S7.x → `docs/specs/reset.md`
   - S8.x → `docs/specs/story_log.md`
   (S6 tests stay in `tests/http/actions.rs` per step 4.)
   Delete `tests/http/flow_sequence.rs`.

6. **Remove SCENARIO tags** from `src/application/pipeline/pipeline_tests.rs` —
   all 14 tags removed (the file currently has 14 tags, all referencing
   `action_pipeline.md`):
   - Removed-from-spec scenarios (8 tags): S3.1, S3.2, S4.1 (×2 —
     duplicate tag at lines 1124 and 1277), S4.2, S4.3, S5.1, S5.2.
   - HTTP-observable scenarios now covered at HTTP tier (6 tags): S1.3,
     S1.5, S2.1, S2.3, S3.3, S3.4.
   - Note: 5 HTTP-observable scenarios (S1.1, S1.2, S1.4, S2.2, S2.4)
     have **no** src/ tag today — they are the pre-existing pilot gaps.
     Step 4 adds HTTP E2E tests for them, closing the gaps.
   - Replace each removed tag with a plain `//` comment describing what
     the test covers (no spec link). **Tests themselves stay** — they
     still do branch-coverage work at the unit tier; they just lose the
     spec link because the spec no longer describes unit-tier behaviour.

7. **Wire new HTTP test files** in `tests/http/mod.rs`:
   `mod reset;`, `mod story_log;`. `mod actions;` already exists — do not
   re-add. Unwire `mod flow_sequence;`.

8. **Verify `validate_feature_spec.py`** runs clean: no gaps (every
   declared scenario has a covering test) and no orphans (every tag
   references a declared scenario). Expected counts after this ticket:
   19 declared (14 in `actions.md` + 2 in `reset.md` + 3 in `story_log.md`),
   19 covered, 0 gaps, 0 orphans. (Invariants I.1–I.4 are not
   pilot-tracked — pilot only sees `#### Scenario N.N` headings.)

## Out of scope

- Creating `docs/specs/snapshots.md` (decided against: snapshots not
  HTTP-observable, same category as `last_trigger`).
- Touching driven-adapter tests in `tests/integration/storage/` (no
  spec-scenario tags there today; snapshots leave the spec, so none
  needed).
- New unit tests for pipeline internals — the unit tier already covers
  pipeline branch coverage; this ticket only removes spec tags from
  pipeline unit tests, not the tests themselves. The 3 new handler unit
  tests in step 4a are in scope (they cover `dispatch_action` branches
  currently indirectly covered by the deleted HTTP tests).

## Acceptance

- `docs/specs/actions.md`, `docs/specs/reset.md`, `docs/specs/story_log.md`
  exist, each endpoint-named, each containing only HTTP-observable
  scenarios in endpoint-level Given/When/Then framing.
- `docs/specs/action_pipeline.md` and `docs/specs/flow.md` are deleted.
- `tests/http/actions.rs` contains **only** new S1–S6 SCENARIO-tagged
  HTTP E2E tests (8 old failure-path tests deleted); `tests/http/reset.rs`
  and `tests/http/story_log.rs` exist with S7 and S8 tests respectively.
  SCENARIO tags point at the matching spec.
- `tests/http/flow_sequence.rs` is deleted; `tests/http/mod.rs` updated
  (`mod flow_sequence;` removed; `mod reset;` and `mod story_log;` added).
- `src/adapters/driving/http/action/handlers/actions_tests.rs` has 3 new
  unit tests for `dispatch_action`'s `Err` / `ShuttingDown` /
  `ConcurrentGeneration` branches.
- `src/application/pipeline/pipeline_tests.rs` has **zero** SCENARIO tags
  (all 14 removed: 8 for removed scenarios, 6 for scenarios now covered
  at HTTP tier). Tests themselves remain for branch coverage.
- `python scripts/validate_feature_spec.py` reports
  19 declared, 19 covered, 0 gaps, 0 orphans.
- `cargo nextest run` green; `cargo nextest run --test guardrails` green
  (SCENARIO-tag placement guardrail still passes — tags only in
  `tests/http/`).

## Notes for the agent

- Scenario IDs stay stable across the rename (S1.1 stays S1.1, S6.1 stays
  S6.1, etc.). The pilot dedups by ID across `docs/specs/*.md`, so IDs
  must stay unique. No renumbering.
- The HTTP E2E tests for S1.x/S2.x/S3.3/S3.4 are new work — they don't
  exist yet. Use the in-process tower/oneshot harness pattern already in
  `tests/http/flow_sequence.rs` (post_action, wait_idle, load_messages).
- Unit tests in `pipeline_tests.rs` that lose their SCENARIO tags still
  pass — they assert internal state for branch coverage, not spec
  behaviour. Only the spec link is removed.
- The 3 new handler unit tests in step 4a should use a pipeline stub
  returning the desired `ProcessActionResult` variant, not storage
  `with_failure` injection. The goal is handler branch coverage, not
  pipeline error-swallowing coverage (that belongs at the pipeline
  tier).
- Mixed-framing interim is avoided by doing the rename + reframe + new
  tests + tag removal in one ticket.
