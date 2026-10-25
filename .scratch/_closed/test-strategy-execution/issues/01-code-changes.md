# Land the three retry-spec code changes

Type: task (HITL)
Status: resolved

## Question

Make the three code changes required by the retry spec
(`docs/specs/retry.md`), confirmed clear from the
[planning map's ticket 04](../../test-strategy/issues/04-consolidate-pipeline-suites.md).
These are retry/retrigger-specific — they block the retry tracks (tickets
04, 05, 06) but not the action pipeline tracks (02, 03), which can
proceed in parallel.

### 1. System log lost in retry error path (R4.8)

`handle_retry_outcome` passes `None` to `finalize_phase_error`, which
loads a fresh state, losing the System message added by
`phase_trigger_continuation_llm_call`. Fix: pass the mutated state
through (same as `run_from_input`'s
`finalize_phase_error(&run, Some(&mut state), e)` pattern).

Current code: `pipeline.rs` line 659 calls
`Self::finalize_phase_error(&run, None, e)`.

### 2. Generation gate on retry/retrigger (R5.3/R5.4, I.8)

`retry()` and `retrigger()` don't take `&GenerationGate` and bypass the
generation gate — no `try_claim` / `is_busy` check. Fix: add
`&GenerationGate` parameter to both; replace `prepare_retry_state` with
`try_claim` (matching `process_action`'s pattern); return
`ProcessActionResult` so the HTTP handler can return 200 with "Still
thinking…" on concurrent generation.

### 3. No-snapshot as 500, not async error (R4.3/R4.4, I.7)

The snapshot check is inside `retry_last_response()` (post-spawn, after
HTTP response sent). Fix: check anchor/snapshot existence in `retry()`
before spawning the task; return `ApplicationError` → HTTP handler
returns 500. (For retrigger, the spec doesn't require a snapshot check —
retrigger operates on current state, not a rollback snapshot.)

### Acceptance

- All three changes pass the retry spec's scenarios (R4.3, R4.4, R4.8,
  R5.3, R5.4) and invariants (I.7, I.8).
- Existing unit tests in `retry_tests.rs` still pass (or are updated to
  match the new signatures).
- The HTTP handlers for `POST /swipe/new` and `POST /retrigger` handle
  the new return types correctly.

## Answer

All three retry-spec code changes landed. Split into three phases (over
5 story points, per `.scratch/AGENTS.md`); each phase verified with a
fresh-context doubt-driven check (temporarily reverted the fix, confirmed
the new test fails, restored).

### Fix #1 — System log lost in retry error path (R4.8)

`handle_retry_outcome` passed `None` to `finalize_phase_error`, which
loaded a fresh state and lost the System message added by
`phase_trigger_continuation_llm_call`. Fixed by threading `&mut state`
through so the in-flight state (with System message) reaches
`finalize_phase_error` via `Some(&mut state)`, mirroring `run_from_input`'s
pattern. `retry_event_continuation` now handles its own non-Cancelled
errors internally (`finalize_phase_error(&run, Some(state), e); return
Ok(())`); `phase_trigger_continuation` takes `&mut GameState` and
returns `Result<String, PhaseError>` (state stays in caller scope).
`handle_retry_outcome` simplified to log `Cancelled` only (both inner
functions now persist their own errors).

Test: `test_retry_event_trigger_narration_fails` fixed to use
`setup_event_flow` (old setup hit the wrong path — Main narration had no
`snapshot_id`, so `find_retry_anchor` returned None and the trigger
continuation was never reached) and gained an assertion that a System
message mentioning "Trigger narration failed" is persisted in history.

### Fix #3 — No-snapshot as 500, not async error (R4.3/R4.4, I.7)

Added `check_retry_anchor()` in `retry()` (pre-spawn): loads messages,
`find_retry_anchor`, `load_snapshot_by_id`. On missing anchor or
snapshot, persists `GenerationStatus::Error(msg)` and returns
`ApplicationError::internal(msg)` → HTTP 500. The defensive check in
`retry_last_response()` is kept for direct callers (tests) — decision
locked during planning. Retrigger unchanged per spec (no snapshot check).

Tests: `test_retry_returns_internal_error_when_anchor_has_no_snapshot_id`
(R4.3) and `test_retry_returns_internal_error_when_snapshot_row_missing`
(R4.4) — both assert `Err(ApplicationError::Engine(EngineError::Internal(_)))`
and the persisted Error status.

### Fix #2 — Generation gate on retry/retrigger (R5.3/R5.4, I.8)

`retry()` and `retrigger()` now take `&GenerationGate`, call `heal_stale`
+ `try_claim` (matching `process_action`'s pattern), and return
`Result<ProcessActionResult, ApplicationError>`. The spawned task holds a
`GenerationGuard` (released on drop). `prepare_retry_state` removed (dead
after the rewrite). HTTP handlers match on `ProcessActionResult`:
`Started` → 200 "Retrying..."/"Retriggering...", `ConcurrentGeneration` →
200 "Still thinking...", `ShuttingDown` → 503.

Tests: `test_retry_returns_concurrent_generation_when_gate_busy` and
`test_retrigger_returns_concurrent_generation_when_gate_busy` (unit,
pre-claim the gate, assert `Ok(ConcurrentGeneration)`);
`test_retry_handler_concurrent_generation` and
`test_retrigger_handler_concurrent_generation` (HTTP, assert 200 +
"Still thinking..." body).

### Verification

- `cargo nextest run -p chronicler_engine`: 1358 passed, 2 skipped
  (platform-specific).
- `guardrails`: 26/26 pass (long-comment-run guardrail satisfied by
  keeping explanatory comments ≤4 lines).
- `cargo clippy --tests`: 0 errors (4 pre-existing-style warnings on
  `format!("...{var}")` in test assertions, matching the existing
  `test_retry_records_missing_snapshot_id` pattern).

### Files changed

- `src/application/pipeline/pipeline.rs` — `retry`,
  `retrigger`, `retry_event_continuation`, `phase_trigger_continuation`,
  `handle_retry_outcome`, `check_retry_anchor` (new),
  `prepare_retry_state` (removed).
- `src/adapters/driving/http/chat_window/handlers/chat_window.rs`
  — `retry_handler`, `retrigger_handler` match on `ProcessActionResult`.
- `src/application/pipeline/pipeline_tests.rs` —
  `test_trigger_continuation_save_post_trigger_error` updated for
  `phase_trigger_continuation`'s new `&mut` signature.
- `src/application/pipeline/retry_tests.rs` —
  `test_retry_event_trigger_narration_fails` fixed + strengthened; 4 new
  tests (R4.3, R4.4, R5.3, R5.4 unit).
- `tests/http/fragment.rs` — 2 new HTTP tests (R5.3,
  R5.4 handler).

### Unblocks

Tickets 04 (retry unit), 05 (retry HTTP E2E), 06 (lifecycle/sequencing/
arrival research) are now unblocked. The retry tracks can proceed.

## Code-review fixes

Two-axis review (`/code-review` skill) flagged two issues, both fixed:

1. **Self-referential comment labels** — stripped `R4.3`/`R4.4`/`R4.8`/
   `R5.3`/`R5.4`/`I.7`/`I.8` plan-spec labels from 10 code comments across
   `pipeline.rs`, `retry_tests.rs`, `tests/http/fragment.rs`. Breached
   `AGENTS.md` DOCUMENTATION STRATEGY → "No
   Self-Referential Comments". Kept the WHY prose; labels belong in commit
   messages, not code.
2. **R4.3 error message contradicted spec** — `check_retry_anchor` emitted
   `"Retry failed: no anchor message"` when the anchor existed but lacked
   `snapshot_id`, but spec R4.3 requires the message "indicate the snapshot
   is missing". Root cause: `find_retry_anchor` used `?` on
   `snapshot_id().as_ref()`, returning `None` for both "no Input" and
   "Input without snapshot". Fix: extracted `find_retry_anchor_msg` (returns
   the anchor without the snapshot check); `find_retry_anchor` now wraps it
   (existing callers unaffected); `check_retry_anchor` calls the new helper
   and branches on `snapshot_id().is_none()` explicitly, emitting
   `"Retry failed: anchor message has no snapshot_id"`. Updated the R4.3
   test assertion to match (`msg.contains("no snapshot_id")`).

Verification: 1358 pass, 26/26 guardrails. Doubt-driven check reverted the
R4.3 fix — test failed with old conflated message — confirmed the test now
encodes the spec, not the bug.
