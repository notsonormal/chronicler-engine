# Smoke Break nudge as a project-local pi extension

## Summary
Port the smoke-break Codex plugin as a project-local pi extension at `.pi/extensions/smoke-break/`. Once per 30-minute interval of a run, on the next tool completion, append one short model-visible reflection prompt to the tool result — so an unattended agent grinding down a rabbit hole is nudged before the user has to intervene. Full design, measured session data, review findings, and decision history live in `docs/plans/pi-extension-smoke-break-nudge.md`.

## Key Changes
- New directory `.pi/extensions/smoke-break/`: `index.ts` (hook wiring), `tracker.ts` (pure timing module), `tracker.test.ts` (unit tests). Directory style so the test file is not auto-discovered as an extension.
- `before_agent_start` resets `{ startedAt, notifiedBucket, intervalMs }` and re-reads `SMOKE_BREAK_INTERVAL_MS` (default 1800000 ms; non-positive disables; unparseable falls back to default). Original algorithm verbatim: `bucket = floor(elapsedMs / intervalMs)`, fire only when `bucket > notifiedBucket`.
- `tool_result` returns `{ content: [...event.content, { type: "text", text: reminder }] }` on a new bucket — delivery contract confirmed in pi docs (Tool Events → `tool_result`). Handler failures are pi's problem: logged, agent continues, result unchanged — no try/catch.
- Draft wording: elapsed-minutes fact + "not a signal that anything is wrong" disclaimer + one-line "state how the current work connects" ask. No stop command, no prescription.
- Interval grounded in measured data: the confirmed build-chase hole (44 min run) would get its nudge 13 min before the user's manual intervention; only ~6% of DeepSeek runs reach 30 min; fast bursts (judged legitimate) stay silent; overnight idles (852 min observed) never fire because firing requires a tool completion.

## Implementation
- [ ] #### Task 1: `tracker.ts` — pure timing module (1 SP)
  - Injected clock; `startTurn(intervalMs)` resets; `onToolComplete(now)` applies the bucket algorithm and returns the reminder text with the elapsed-minutes label (`max(1, round(bucket * intervalMs / 60000))`) only on a new bucket; silent with no active turn or when disabled.
- [ ] #### Task 2: `index.ts` — extension entry and hook wiring (1 SP)
  - Default export factory; read/validate `SMOKE_BREAK_INTERVAL_MS` (non-positive → disabled; unparseable → default); `pi.on("before_agent_start")` → `startTurn(Date.now(), intervalMs)`; `pi.on("tool_result")` → bucket check and conditional content patch. No try/catch — pi logs handler errors and continues. Wording per plan Task 3 draft.
- [ ] #### Task 3: `tracker.test.ts` — unit tests (1 SP)
  - First verify `node --test` runs `.ts` on Node v24.18.0 (type stripping; fallback: `.mjs` tracker). Cases: silent through bucket 0 (interval − 1 ms), fires at exactly the interval with the disclaimer present, one notification per bucket (silent at N+1, fires again at 2N), new-turn reset, "30 minutes"/"60 minutes" labels and singular "1 minute" for a 60 s interval, custom interval, interval change applies from the next turn while the run in progress keeps its interval, disabled (non-positive), silent with no active turn.
- [ ] #### Task 4: Real-session smoke test (1 SP)
  - `/reload`; run with `SMOKE_BREAK_INTERVAL_MS=60000` set on the `pi` process itself, on a multi-minute task; verify one reminder per 60 s of run time, none while idle, reminder visible as `tool_result` content in the transcript. Then a real long DeepSeek session decides whether the wording ships.

## Test Plan
- Unit: `node --test .pi/extensions/smoke-break/tracker.test.ts` — all Task 3 cases green.
- Manual: acceptance criteria in the plan file — exactly one reminder per elapsed interval, idle sessions (including overnight-open ones) emit nothing, disable switch works, and observed behaviour change on a real long DeepSeek session.

## Per Task/Sub Task Validation Steps
- Task 1: module imports cleanly; no pi imports; pure functions take/return plain values.
- Task 2: `/reload` loads without errors; patch shape matches the `content: [{ type: "text", text }]` contract; disabled mode makes `tool_result` a no-op.
- Task 3: all tests green; if type stripping fails, tracker rewritten as `.mjs` and tests rerun.
- Task 4: transcript shows reminder cadence at interval boundaries only; session history shows the reminder appended to a tool result.

## Assumptions
- `before_agent_start` does not refire on auto-compaction retry; a mid-stream steering message may or may not reset the timer — both acceptable, observed during implementation.
- The reminder persists in session history as part of the patched `toolResult` message (inferred from the message flow); v1 keeps it, pruning via the `context` event is a later option.
- Fast bursts (many tool calls within one interval) are accepted losses — the user judges them not-necessarily-holes.
- Wall-clock elapsed includes long tool waits (e.g. 43-minute builds), so a nudge may land right after a legitimate long build; accepted for v1 parity with the original.
- `/reload` mid-run loses tracker state; the next nudge waits for the next user prompt. Accepted.
