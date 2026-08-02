# Rename `ActionResult::next_state` to `post_commit_state`

## Summary
Mechanical rename of one struct field. Closes the naming asymmetry between `ActionResult::next_state` (still old) and `run_from_input`'s local `post_commit_state` variable (already renamed). No behavior change.

## Key Changes
- `ActionResult::next_state` → `ActionResult::post_commit_state` (struct field).
- One struct-literal site (line 354): `next_state,` → `post_commit_state: next_state,` (shorthand won't work because the local var keeps the old name).
- 13 `.next_state` field-access sites across 3 files become `.post_commit_state`.
- Local variable `let mut next_state = self;` inside `execute_freeaction_impl` (line 314) stays as is — it is a coincidental name match, not a use of the field. Renaming it is out of scope; the minimum diff that fixes the reported asymmetry is the field rename.

## Implementation

### Phase 1: Rename + verify

- [ ] #### Task 1.1: Rename `ActionResult::next_state` field (1 SP)
  - `chronicler_engine/src/domain/model/state/game_state.rs`:
    - line 185: `pub next_state: GameState,` → `pub post_commit_state: GameState,`
    - line 354: `next_state,` → `post_commit_state: next_state,` (inside `Ok(ActionResult { ... })`)
  - `chronicler_engine/src/application/pipeline/pipeline.rs:470`: `turn_result.next_state` → `turn_result.post_commit_state`
  - `chronicler_engine/src/domain/model/state/game_state_action_processing_tests.rs`: rename all 6 `.next_state` accesses to `.post_commit_state` (lines 112, 142, 171, 193, 382, 623). Local `let next_state = ...` bindings stay.
  - `chronicler_engine/tests/infrastructure/invariant_contract.rs`: rename all 6 `.next_state` accesses to `.post_commit_state` (lines 171, 175, 358, 362, 440, 450).
  - **Validation:** `cargo check --all-targets --all-features` green. `python chronicler_engine/build.py` green.
  - **Resolve:** set `Status: resolved`, append answer to ticket, add one-line gist + link to map's Decisions-so-far.

## Test Plan
- `cargo check --all-targets --all-features` must be green.
- `python chronicler_engine/build.py` must be green (12 steps; full nextest run, with `--retries 2` per repo convention).
- `grep -rn "\.next_state" chronicler_engine/src chronicler_engine/tests` returns no field-access hits (only the local `next_state` var + `next_state(...)` function calls in `game_state.rs` may remain, which are unrelated).

## Per Task Validation Steps
- **Task 1.1:** Both commands above green. `grep` confirms only local-var / unrelated function-call references remain.

## Assumptions
- The 18 `next_state` occurrences in `game_state_action_processing_tests.rs` break down as 6 field accesses (the ones we rename) + local `let next_state = ...;` bindings + `next_state.narrative.history()[...]` chained reads (which are reads of the local binding, not field accesses). Only the 6 `.next_state` field-access sites get renamed; the locals stay.
- `execute_freeaction_impl`'s local `next_state` (line 314) keeps its name. The struct-literal at line 354 uses explicit `post_commit_state: next_state,` to disambiguate from the local. The asymmetry "local `next_state` is the value assigned to field `post_commit_state`" is mild but explicit, and matches the decision document's "mechanical rename only" constraint.
- No documentation, ADR, or comment text references the old field name (verified by `grep ActionResult` + `grep next_state` — no doc comments mention the field name).
