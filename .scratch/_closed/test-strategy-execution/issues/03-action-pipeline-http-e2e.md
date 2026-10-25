# Move action pipeline tests to HTTP E2E and grow the spec

Type: task (HITL)
Status: closed

## Scope (revised)

Originally this ticket's acceptance required "`action_pipeline.md` is
rewritten in HTTP framing" over all of S1–S5 + flow scenarios. After a
planning review (see map Decisions-so-far), that full reframe is split off
to [ticket 11](11-spec-restructure-http-observable-only.md).

**This ticket 03 delivers:**
- Move 8 `flow/sequence.rs` tests to HTTP E2E in `tests/http/flow_sequence.rs`.
- Append **new** HTTP-framed S6 (sequencing), S7 (reset), S8 (delete) scenarios to `action_pipeline.md`.
- Delete `flow/sequence.rs`.
- Fix `validate_feature_spec.py` scan dirs.

**Split off to ticket 11:**
- Reframe S1.1–S2.4, S3.3, S3.4-success to HTTP Given/When/Then.
- Remove S3.1, S3.2, S4.1–4.3, S5.1, S5.2, I.5 from `action_pipeline.md`.
- Remove 14 SCENARIO tags from `pipeline_tests.rs`.

The "spec rewritten in HTTP framing" acceptance below is therefore
**partially met** by this ticket (flow scenarios only); the remainder is
ticket 11.

## Question

Move the action pipeline flow tests from the component tier to HTTP E2E
(`tests/http/`), deriving HTTP-framed spec scenarios from the component
tests as you go. The existing `action_pipeline.md` spec is reference
material — it was written at the pipeline level (`execute_action_impl`),
but specs describe what the client observes at the endpoint. The real
spec grows alongside the E2E tests.

### Workflow

For each component test moving to HTTP E2E:

1. Read the component test to understand what behaviour it asserts.
2. Derive an HTTP-framed spec scenario (Given/When/Then at the endpoint
   level — what the client sends and observes). Write it into
   `docs/specs/action_pipeline.md`, replacing or revising the existing
   pipeline-level scenarios.
3. Write the HTTP E2E test from the spec scenario, using the in-process
   tower/oneshot harness.

Some scenarios will translate cleanly (single-call). Others will need
multi-call framing (submit action → poll status → fetch fragment).
Internal-state assertions that have no HTTP observable equivalent get
dropped or translated to observable proxies.

### Tests to move (from the diff asset — "keep (flow)" classification)

Per the [diff asset](../../test-strategy/assets/pipeline-suite-diff.md),
the action pipeline flow tests moving up are the "keep (flow /
cross-boundary)" set from `flow/sequence.rs`:

- `test_sequential_execute_retry_execute` — multi-action sequence
- `test_sequential_execute_delete_execute` — delete between actions
- `test_async_action_sequence_then_retry` — async sequence + retry
  (depends on code changes — ticket 01)
- `test_three_actions_in_sequence` — three-action ordering
- `test_delete_input_then_retry_fails_gracefully` — delete-then-retry
  (depends on code changes — ticket 01)
- `test_reset_clears_history_and_state` — reset
- `test_reset_then_execute_works` — reset-then-execute
- `test_delete_mid_sequence` — mid-sequence delete

Note: some of these involve retry and are blocked by ticket 01. The
non-retry ones (reset, delete, pure sequencing) can proceed immediately.
Split the work accordingly within the ticket.

### What the spec rewrite involves

The existing `action_pipeline.md` has scenarios S1.1–S5.2 and invariants
I.1–I.5. The HTTP-framed rewrite:

- Keeps the scenario structure but reframes Given/When/Then to endpoint
  level (POST /action, GET /fragment/story-log, GET /status/generating).
- Drops assertions on internal state that can't be observed via HTTP
  (e.g., `narrative.input_buffer.phase`) or translates them to
  observable proxies (e.g., status endpoint response).
- Adds scenarios for the flow tests that aren't in the current spec
  (sequencing, reset, delete).
- Preserves scenario numbering where possible (the tagging guardrail
  references it), but renumber if the reframe changes the scenario set.

### Acceptance

- Every action pipeline flow test has an HTTP E2E equivalent.
- `docs/specs/action_pipeline.md` is rewritten in HTTP framing.
- `flow/sequence.rs` is deleted after its tests are moved (the retry-
  dependent ones may wait for ticket 01).
- SCENARIO tags on the new E2E tests match the spec scenarios.
- Suite green.

## Resolution

All 8 flow tests from `tests/integration/flow/sequence.rs` ported to
`tests/http/flow_sequence.rs` as HTTP E2E, driving the real adapter (POST
/action, /swipe/new, /history/delete, /reset). Spec grown with S6
(sequencing), S7 (reset), S8 (delete) — HTTP-framed Given/When/Then.
`flow/sequence.rs` deleted; unwired from `tests/integration/mod.rs`.
`validate_feature_spec.py` scan dirs fixed to match STRATEGY.md
(tests/http/, src/, tests/integration/storage/).

**Scope split:** S1–S5 reframe + non-HTTP scenario removal + 14 tag removal
from `pipeline_tests.rs` split to ticket 11 (spec restructure). The
"spec rewritten in HTTP framing" acceptance is partially met by this
ticket (S6/S7/S8 only); remainder is ticket 11.

**Spec relocation during implementation:** S6/S7/S8 scenarios moved out
of `docs/specs/action_pipeline.md` into a new `docs/specs/flow.md`.
Flow (multi-call sequencing, reset, delete mid-sequence) is a distinct
concept from action pipeline (single-call lifecycle). Keeping them in
the same spec would muddy both. Scenario IDs (6.x/7.x/8.x) kept for
continuity with the `tests/http/flow_sequence.rs` SCENARIO tags; the
pilot dedups by ID across `docs/specs/*.md`, so flow.md owns 6.x–8.x
and action_pipeline.md owns 1.x–5.x. SCENARIO tag paths in
`flow_sequence.rs` updated to point at `docs/specs/flow.md`.

**Spec correction during implementation:** S7.1/S7.2 initially asserted
"history empty after reset" — corrected to match actual `/reset` behaviour
(deletes game, creates fresh with opening narration: previous Input
gone, exactly 1 Narration = opening). The original `flow/sequence.rs`
reset tests used manual `save_test_state` to clear state, NOT `/reset`;
the HTTP tests test the real endpoint, which has different semantics.

**Pilot note:** `validate_feature_spec.py` reports 5 pre-existing gaps
(S1.1, S1.2, S1.4, S2.2, S2.4) — scenarios declared with no covering test
anywhere. Pre-existing, out of scope; ticket 11 resolves (reframe or
remove). 0 orphans.

Suite: 1358 pass, 2 skipped. Guardrails 101/101.

Unblocks: nothing (downstream tickets 04/05/06 were already unblocked by
01). Ticket 11 (spec restructure) is independent.
