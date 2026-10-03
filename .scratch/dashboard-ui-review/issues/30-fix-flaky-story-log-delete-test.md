# Stop the story-log delete test flaking under parallel load

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

`test_delete_mid_sequence_http` (`tests/http/story_log.rs:57`, scenario 8.2) failed inside a full-gate run with `should have 3 Input entries: got 1`, then passed in isolation and on the next full-gate run of the same commit. What is the race, and what change makes the test deterministic under CPU contention?

## Context

- Surfaced during this map's [ticket 01](01-fix-nested-story-log.md) run: two concurrent cold `python build.py` gates plus browser suites on a 4-core container. The implementer's handoff records the failure text and that the rerun on identical code was green. The failing run's worktree was removed with its nextest log, so the handoff is the surviving evidence.
- The failing assertion is the Input-count check, not a `wait_idle` assert — all three `wait_idle(&state, 1000)` calls returned true before the count was read. The failure is not a plain wait timeout. [known from the assertion messages]
- `wait_idle` polls `!input_buffer.status.is_generating()` every 15ms (`tests/http/support/http_requests.rs:103`). A window where the status has not yet flipped to generating after `post_action` returns would let the next action land on a busy engine. Candidate mechanisms to check: early-true idle, a dropped or rejected follow-up action, and `post_empty("/history/delete")` removing a different message than the test assumes. [inferred]
- The pattern `wait_idle(&state, 1000)` appears 86 times under `tests/http/`; state whether a fix applies to all call sites or only this test.
- Not plausibly caused by tickets 01 or 02: ticket 01 changes template rendering, ticket 02 changes client JS; neither touches the pipeline or storage. [known from the diffs]
- Related precedent: `.scratch/test-strategy-execution/issues/09-nextest-config.md` kept `retries = 1` as a small safety net; this failure exhausted it.

## Done when

- The race is named with evidence — a reproduction (a loop of the test under load, or an instrumented wait) or a documented reason it cannot be a race.
- A fix lands, placed by `tests/STRATEGY.md` (HTTP E2E or unit per the mechanism).
- The test passes repeatedly under contention — e.g. two concurrent full gates on this host, or a targeted loop with `-j 4` nextest plus a CPU-load generator.
- `python build.py` is green. Commit after user approval.

## Answer

**The race.** `wait_idle` (`tests/http/support/http_requests.rs`) treated idle as the persisted `GenerationStatus` alone. The pipeline persists `Idle` in `PipelineRun::phase_finalize` (`src/application/pipeline/pipeline_run.rs` ~433-456) before the spawned task returns. The per-game generation slot is freed only when the task's `GenerationGuard` drops (`claim_and_spawn` in `src/application/pipeline/action_pipeline/core.rs`; `src/application/generation/guard.rs`). In that window the status reads Idle but `GenerationGate::try_claim` still sees the slot as busy. The test's next `post_action` or `/swipe/new` gets `ConcurrentGeneration` ("Still thinking..."), and its Input is never saved. That gives `should have N Input entries: got 1` while every `wait_idle` returned true. Cancellation (`handle_cancellation`) and phase errors (`finalize_phase_error`) persist a non-generating status before the guard drops too, so they share the window. [mechanism known: verified in code by the reviewer]

**Evidence.** Three tests failed with this signature under full-gate load: `test_delete_mid_sequence_http`, `test_delete_last_between_actions_http`, `test_sequential_execute_retry_execute_http`. The implementer's probe (temporary, removed) ran 2000 action cycles at load average ~15 and hit the window once, with a real `post_action` answered "Still thinking". After the fix: 2000 cycles, 0 hits. 40 loops each of two of the flaky tests under contention: 80/80 pass. [reported by the implementer, not re-run by the coordinator]

**Fix (test code, tier 1 HTTP helper).** `wait_idle` also requires `!state.generation_gate.is_busy(game_id)`. All 87 `wait_idle` call sites in `tests/http/` go through this helper, so the fix covers every one. No production change.

**Gate:** `nextest: 1634 passed, 0 failed, 2 skipped`, browser 23 passed (worktree log `build_20260930_195007.log`). The coordinator then trimmed the doc comment only.

**Code review** (`/code-review`, verdict ISSUES → fixed): the comment said "See ticket 30", which `CODING_STANDARDS.md` forbids; removed and the comment trimmed to the ordering constraint. Judgement calls left open:
- `game_id` is captured once before polling. That is correct for every current caller, since none switch games during a wait.
- Reading `generation_gate` deepens the internal-observation question [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) is weighing. `wait_idle` is a sync helper, not an assertion, and already read internal status.
- The same window exists in production. The UI does not prevent it: `/status/generating` reads the same lagging status. It lasts microseconds against a 5s poll, so no ticket. If it ever matters, the engine could release the slot together with persisting `Idle`.

