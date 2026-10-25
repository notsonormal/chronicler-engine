# Port retry unit tests down from the component tier

Type: task (HITL)
Status: closed
Blocked by: 01
Resolved: 2026-08-03

## Question

Port retry branch-coverage assertions from the component tier into the
in-memory unit suite (`src/application/pipeline/retry_tests.rs`). Blocked
by ticket 01 (code changes) because the code changes alter retry/retrigger
signatures and behaviour — unit tests must be written against the new
code.

### From the diff asset (action_pipeline/retry.rs)

Per the [diff asset](../../test-strategy/assets/pipeline-suite-diff.md):

- **Retry recovery after LLM failure** (`test_retry_after_llm_failure`) —
  no in-memory equivalent. Port down.
- **Retry room-not-found** (`test_retry_room_not_found`) — Error with
  room-invalid message. No in-memory equivalent. Port down.
- **Main-retry LLM failure** (`test_retry_llm_error`) — in-memory covers
  trigger-narration fail, not main-retry fail. Port down.
- **Main-retry empty narration** (`test_retry_empty_narration`) — in-memory
  covers event empty, not main-retry empty. Port down.
- **Retry no-snapshot** (`test_retry_no_snapshot`) — **drift.** In-memory
  asserts `Error`; SQLite asserts only `!is_generating()` (weaker). Align
  to the spec (R4.3/R4.4: 500 / Error) and the in-memory assertion.
- **Retry no-input-text** (`test_retry_no_input_text`) — noop with existing
  narration. Port down.
- **Retry main narration uses pre-main snapshot** — port completion
  assertions down.
- **Retry finds last input and runs pipeline** — narration-count stays 1
  (replace, not append at history level). Port the count invariant down.

### From ticket 11 (spec restructure — HTTP-observable, endpoint-named)

Ticket 11 added a top-of-function `is_shutting_down()` guard to
`process_action`, `retry`, and `retrigger` in `src/application/pipeline/pipeline.rs`
(consistent 503 across the action surface). New branches needing coverage:

- **Shutdown guard at top of `retry()`** — `is_shutting_down()` → returns
  `Ok(ProcessActionResult::ShuttingDown)` before any spawn or state load.
- **Shutdown guard at top of `retrigger()`** — same shape.
- **Handler unit tests for `/swipe/new` + `/retrigger` shutdown → 503.** Ticket
  11 added a handler test only for `/action` (`dispatch_action` → 503);
  `retry_handler` + `retrigger_handler` have no shutdown-path test today
  (existing tests assert "any 2xx/4xx/5xx"). These belong at the handler
  unit tier alongside `src/adapters/driving/http/action/handlers/actions_tests.rs`.

Ticket 11 also left a unit-tier mislabel fog explicitly deferred for this
ticket:

- **`test_pipeline_trigger_complete_failure`** (pipeline_tests.rs, line 422)
  uses `MockBackend::with_fail()` which fails both main AND trigger narration.
  The corrected spec (S2.4, ticket 11) says "main narration preserved +
  trigger fails → Error + System log". Update the test to use
  `with_trigger_narration_fail()` so it matches the scenario it claims to
  cover.

### From the code changes (ticket 01)

After the code changes land, add unit tests for the new behaviour:

- **Generation gate on retry/retrigger** (R5.3/R5.4): concurrent
  generation rejected → `ProcessActionResult` indicating "Still
  thinking…".
- **No-snapshot pre-spawn** (R4.3/R4.4): snapshot check before task spawn
  → `ApplicationError`, not post-spawn async error.
- **System log persistence in retry error path** (R4.8): System message
  survives `finalize_phase_error`.

### Acceptance

- Every retry branch has a unit test, including the new branches from
  ticket 01 and the shutdown guard branches from ticket 11.
- Handler unit tests for `/swipe/new` and `/retrigger` shutdown → 503 paths
  (alongside the existing `/action` shutdown test in `actions_tests.rs`).
- `test_pipeline_trigger_complete_failure` uses
  `with_trigger_narration_fail()` (matching S2.4's spec claim: main
  preserved, trigger fails).
- `action_pipeline/retry.rs` is deleted after its assertions are ported.
- Drift cases aligned to the retry spec.
- Suite green.

## Resolution

All acceptance criteria met. Suite green: 1361 pass, 17/17 spec, 0
violations.

**Ported to `src/application/pipeline/retry_tests.rs`** (4 new branch-
coverage tests):
- `test_retry_recovers_after_llm_failure` — `with_fail_first_n(1)`;
  input pre-seeded (mirrors component-tier `.message(msg)`); 0→1
  fail→recover cycle, narration count == 1.
- `test_retry_room_not_found_sets_error` — seeds input whose snapshot
  points at `non_existent_room`; asserts `Error` containing
  `"Room not found"`.
- `test_retry_llm_error_sets_error` — main-retry LLM failure path
  (distinct from trigger-fail test).
- `test_retry_empty_narration_sets_error` — main-retry empty-response
  path; asserts `Error` msg contains `"empty"`.

**Added (ticket 11 shutdown-guard branches):**
- `test_retry_returns_shutting_down_when_token_cancelled`
- `test_retrigger_returns_shutting_down_when_token_cancelled`

**Strengthened existing tests:**
- `test_retry_main_narration_happy_path` — now asserts final `Idle` +
  `!narrations.is_empty()` (completion invariant from component-tier
  `test_retry_main_narration_uses_pre_main_snapshot`).
- `test_retry_no_input` — now seeds System + Narration and asserts
  `history.len()` unchanged (noop-on-history invariant).

**Added during review (replace-not-append history invariant):**
- `test_retry_replaces_narration_not_appends_at_history_level` —
  successful action (count 1) → retry → history-level count stays 1
  (replace, not append). Swipe-level append covered separately by
  `test_retry_appends_swipe_to_same_message`.

**Fixed mislabel:** `test_pipeline_trigger_complete_failure`
(`pipeline_tests.rs`) — `with_fail()` → `with_trigger_narration_fail()`;
asserts (a) status `Error` mentioning `"Trigger narration failed"`,
(b) at least one `Narration` preserved (main succeeded), (c) at least
one `System` message mentioning `"Trigger narration failed"`. Matches
`actions.md` S2.4. History-dbg collect collapsed to one binding
(code-simplification review fix).

**Handler tests (`chat_window/handlers/chat_window_tests.rs`):** replaced
loose `test_retry_handler` / `test_retrigger_handler` ("any 2xx/4xx/5xx")
with four specific tests:
- `test_retry_handler_returns_503_on_shutdown`
- `test_retrigger_handler_returns_503_on_shutdown`
- `test_retry_handler_returns_400_when_no_input` (Validation arm)
- `test_retrigger_handler_returns_400_when_no_trigger_context`
  (Validation arm)

**Deleted:** `tests/integration/application/action_pipeline/retry.rs`
(-10 component-tier tests); unwired `mod pipeline_retry` from
`tests/integration/mod.rs`.

**Drift aligned:** the in-memory `test_retry_no_snapshot` already
asserted `Error` containing `"Retry failed: no anchor message"`; the
weaker SQLite version (which only asserted `!is_generating()`) was
removed with the component-tier file.

**Not in scope (ticket 05):** `flow/retry_main.rs` + `flow/retry_event.rs`
+ `docs/specs/retry.md` scenario edits — untouched.
