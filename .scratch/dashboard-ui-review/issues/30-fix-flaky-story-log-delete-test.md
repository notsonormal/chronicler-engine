# Stop the story-log delete test flaking under parallel load

Type: task (AFK)
Status: open
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
