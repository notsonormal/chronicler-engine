# Plan: Port the Smoke Break nudge into a pi extension

**Date:** 2026-09-14, calibrated against real session data 2026-09-15
**Status:** Implemented 2026-09-16. Trigger: elapsed time, default 30 minutes
(revised from turn-count after session-data review, same day). Reminder
wording: the original plugin's text verbatim plus a blind-disclaimer suffix
(decided 2026-09-16); real sessions validate it.
**Goal:** Add a pi extension that, once per elapsed interval of a run, appends
a short model-visible reflection prompt to a tool result, so an agent grinding
down a rabbit hole is nudged to reassess before the user has to intervene.

> The mechanism mirrors the original plugin (elapsed-time buckets, fires only
> on tool completion) with a 30-minute default instead of 5. Wording is the
> original's verbatim plus a blind-disclaimer suffix; iteration is an edit
> plus `/reload`.

## Motivation

DeepSeek V4.1 Flash goes into rabbit holes: it keeps taking defensible actions
after the point where more work can no longer change the conclusion. Two
confirmed incidents in this repo's session history:

- **The marathon (2026-09-13 15:12–17:51).** A user prompt at 15:12:18 produced
  a 2 h 39 m run with 233 tool calls while the user was away; they returned
  48 minutes after it went quiet.
- **The build-chase (2026-09-13 19:14–19:58).** Asked about a warm/cold build
  gap, the agent ran successive 4–8 minute builds for 44 minutes. The user
  intervened at 19:58:33 with "What is the endgoal here? I feel like this
  isn't productive".

Prose steering in `AGENTS.md` is the existing lever and helps some models; a
scheduled in-context nudge covers the unattended case without paying context
cost on every turn.

## Measured behaviour (2026-09-15, corrected)

Parsed 188 session files under `~/.pi/agent/sessions/…chronicler-engine…/`
(2026-09-08 onward, top-level only; subagent `tasks/` sessions excluded).
DeepSeek is identifiable per assistant message: provider `synthetic`, model
`hf:deepseek-ai/DeepSeek-V4.1-Flash`. Six sessions used it, producing 52
user-prompt runs. An earlier pass carried a timestamp-slicing bug (hours read
with one digit); all durations below are from the corrected pass.

| Measure | Value over the 52 DeepSeek runs |
|---|---|
| Active time per run (gaps > 5 min treated as idle) | mean 9.5 min, max 137 min |
| Tool calls per run | median 6–20; 12 runs ≥ 50; max 233 |
| Working pace | ~2–12 tool calls per active minute |
| Runs with one ≥ 4 min single-tool call (long builds) | 13 of 52 |
| Runs reaching 30 min of activity | 3 of 52 (~6%) |

Incident case studies:

- **Marathon (15:12:18 → 17:51:49):** 159 min span, 137 min active, 233 tool
  calls. User away throughout; next input 18:40:23.
- **Build-chase (19:14:40 → 19:58:33):** 44 min span, ~35 tool calls, no gap
  over 5 min — continuous build-edit-build cycling. A 30-minute trigger fires
  at ~19:45:35 (first completion past minute 30), 13 minutes before the user
  intervened.
- **The literal "What are you doing?" (2026-09-12 20:13:24)** happened in a
  Kimi-K3/GLM ticket-18 session, 3 m 39 s into its run, and ended with
  "Oh, continue" — a benign check-in, not a hole.
- **Fast bursts exist and can be legitimate.** A 4.3-minute run with 52 tool
  calls (the 2026-09-14 survey/git-log sweep) completes before any 30-minute
  trigger. The user's reading: a 50-call burst inside five minutes is not
  by itself hole-digging, so missing it is acceptable.
- **Overnight idles are real and must not fire.** Two sessions sat idle for
  757 and 853 minutes; no activity occurred. A trigger that fires on tool
  completion only stays silent through idles — one reason the design keeps
  the original's completion-anchored firing.

## Prior art

[smoke-break](https://github.com/ElKornacio/agent-plugins/tree/main/plugins/smoke-break)
— a Codex plugin, MIT, by ElKornacio. Read from source:

- `UserPromptSubmit` → `turn_start` records `{ turnId, startedAt, notifiedBucket:
  0, intervalMs }` in a `Map` keyed by session id. Duplicate starts for the
  same turn id do not reset the timer. Above `DEFAULT_MAX_SESSIONS = 1024` it
  evicts the oldest session (insertion order).
- `PostToolUse` → `tool_end` computes `elapsedMs = max(0, now - startedAt)` and
  `bucket = floor(elapsedMs / intervalMs)`. Unknown session or stale turn id:
  silent. Bucket 0: silent. Bucket already notified: silent. Otherwise it sets
  `notifiedBucket = bucket` and returns `hookSpecificOutput.additionalContext`
  with the reminder and an elapsed-minutes label.
- Config: `SMOKE_BREAK_INTERVAL_MS` from `~/.smoke-break.env`, re-read per turn,
  default 300000 ms.
- Tests: `node --test` on the pure tracker with an injected clock.

Its design property worth keeping: it fires only on tool completion, so it
cannot interrupt an idle agent — only one actively working.

## Why a pi port is smaller

| Concern | Codex original | pi |
|---|---|---|
| Transport | MCP stdio server + hooks.json | none — extension runs in-process |
| Hook | `mcp_tool` → `src/server.mjs` | `pi.on("tool_result", …)` |
| Turn anchor | `${turn_id}` via `UserPromptSubmit` | `pi.on("before_agent_start", …)` |
| Turn identity | `${session_id}` key in a bounded `Map` | none — pi rebinds extension instances per session, so closure state is already session-scoped |
| Dependencies | zero | zero |

Target: roughly 40–60 lines across `index.ts` and `tracker.ts`.

## Scope

### Task 1: Turn-state tracker — settled
- One closure record per extension instance:
  `{ startedAt, notifiedBucket, intervalMs } | null`.
- The original's session-keyed `Map`, eviction cap, and turn ids disappear: pi
  rebinds extension instances per session (session replacement emits
  `session_shutdown`, then reloads — `docs/extensions.md`, session replacement
  lifecycle), so instance closure state is already session-scoped.
- Reset to `{ startedAt: Date.now(), notifiedBucket: 0, intervalMs }` on every
  `before_agent_start` (parity with the original's `UserPromptSubmit` anchor).
  Auto-compaction retries do not involve user input and should not reset.
- Keep the tracker a pure module with an injected clock, mirroring
  `turn-tracker.mjs`, so `node --test` exercises it without pi.
- Algorithm, unchanged from the original: `elapsedMs = max(0, now -
  startedAt)`; `bucket = floor(elapsedMs / intervalMs)`; fire only when
  `bucket > notifiedBucket`; on fire set `notifiedBucket = bucket` and produce
  the reminder with an elapsed-minutes label
  (`max(1, round(bucket * intervalMs / 60000))`).

### Task 2: Extension hook wiring — settled
- Location: `.pi/extensions/smoke-break/` in chronicler-engine (project-local,
  versioned). Directory style, so sibling test files are not auto-discovered
  as extensions. Pi discovers `extensions/*/index.ts` as the entry point:

  ```
  .pi/extensions/smoke-break/
    index.ts          # entry, default export factory
    tracker.ts        # pure timing logic (Task 1)
    tracker.test.ts   # unit tests (Task 5)
  ```

- `pi.on("before_agent_start", …)` — record turn start; re-read the interval
  config (Task 4).
- `pi.on("tool_result", …)` — bucket check; on a new bucket, return a patch
  appending one text part:
  `{ content: [...event.content, { type: "text", text: reminder }] }`.
- Delivery to the model is confirmed by docs, not inferred: `tool_result`
  "**Can modify result**", handlers return partial patches, and the final
  `toolResult` message events are emitted from the patched result
  (`docs/extensions.md`, Tool Events → `tool_result`). End-to-end confirmation
  still runs as the manual smoke test (Task 5).
- Named trade-off: the reminder persists in session history as part of that
  `toolResult` message (inferred from the message flow; verify in the smoke
  test). Cost is one short text part per interval; the benefit is that the
  model can see when it was nudged. Pruning old reminders via the `context`
  event is possible later and is out of scope for v1.
- Handler failure contract: extension errors are logged and the agent
  continues (`docs/extensions.md`, Error Handling). No try/catch wrapping — a
  throwing `tool_result` handler leaves the tool result unchanged. The
  `tool_call` blocking fail-safe does not apply to `tool_result`.
- Assumptions to check in implementation, acceptable either way:
  `before_agent_start` does not refire on auto-compaction retry, and a
  mid-stream steering message may or may not reset the timer.

### Task 3: Reminder wording — settled (original verbatim + blind disclaimer)
Decided 2026-09-16, after an interim draft was rejected. The interim line
"state how the current work connects to the original request" was vacuous: a
model inside a rabbit hole can always rationalize a connection, so the ask
elicits a justification, not a check. The original's text instead asks for an
evaluation (progress reasonable? time proportionate?) with a
continue-by-default rule and an only-if escape hatch. The original's final
clause does name remedies, which earlier brushed against the "nothing
concrete" constraint; the user accepted it — the remedies are conditional on
the model's own verdict, not presupposed trouble.

Final text (`E` is the elapsed-minutes label, `C` the cadence-minutes label
from the configured interval; both use the original's
`max(1, round(ms / 60000))` floor):

> Smoke break: this turn has been running for about {E} {minute|minutes}. This
> is only a gentle checkpoint. Consider whether the work is progressing
> reasonably and roughly according to plan. If so, or if any deviation seems
> modest, simply continue. Only if it appears substantially off course or the
> time spent feels disproportionate, consider whether changing approach or
> asking the user would help. This is an automated message, posted every {C}
> {minute|minutes} without user input. It has no direct knowledge of what is
> happening in the session.

The disclaimer suffix is the one part the original lacks; it prevents the
model from reading the nudge as a trouble signal. Still gated on the
real-session smoke test (Task 5, step 4): pure verbatim without the suffix is
the fallback if the disclaimer proves noisy.

### Task 4: Configuration — settled
- `SMOKE_BREAK_INTERVAL_MS` from the process environment, a positive integer
  number of milliseconds. Default 1800000 (30 minutes — the user's choice
  after session-data review; the original's default was 5 minutes). A value
  that parses to a non-positive integer disables the extension (handler
  no-op). A value that does not parse (non-numeric, non-integer) falls back
  to the default, mirroring the original's invalid-value behaviour.
- Re-read on every `before_agent_start`, preserving the original's "edits
  apply to the next turn" property without the `~/.smoke-break.env` file.

### Task 5: Tests
- Tracker unit tests via `node --test` on `tracker.ts` with an injected clock,
  mirroring the original suite minus its session-map cases:
  - silent through bucket 0 (interval minus 1 ms), fires at exactly the
    interval;
  - exactly one notification per bucket across repeated tool completions, and
    again at the second interval;
  - a new turn resets elapsed time and the notified bucket;
  - elapsed label: "30 minutes" at the first bucket, "60 minutes" at the
    second, singular "1 minute" for a 60 s interval;
  - reminder text carries the original's ask ("simply continue") and the
    automated-message disclaimer ("no direct knowledge");
  - a custom interval is respected;
  - an interval change applies from the next turn; the run in progress keeps
    the interval it started with (parity with the original's per-turn
    interval capture);
  - disabled (non-positive interval) is always silent;
  - tool completions with no active turn are silent (replaces the original's
    unknown-session/stale-turn cases).
- First step: confirm `node --test` runs `.ts` directly. Node here is
  v24.18.0, which strips types by default (since 23.6) for erasable syntax —
  expected to work; the probe was blocked under plan mode, so verify on the
  first implementation step. Fallback: keep the tracker as plain `.mjs`
  imported by the TS entry; jiti resolves either.
- Manual smoke test (the acceptance gate that matters):
  1. `/reload` with the extension in `.pi/extensions/`.
  2. Run a session with `SMOKE_BREAK_INTERVAL_MS=60000` (set on the `pi`
     process itself) and a task that runs several minutes; confirm exactly
     one reminder per 60 s of run time and none while idle.
  3. Confirm the reminder text lands as `tool_result` content in the
     transcript.
  4. Verify on a real long DeepSeek session that the nudge changes behaviour —
     the wording's test, and the only one that matters.

## Out of scope

- Changing `AGENTS.md` prose. The existing anti-analysis-paralysis rule is kept;
  this extension is an addition, not a replacement.
- Fixing the DeepSeek V4.1 Flash behaviour by model routing or model choice.
- Any change to the Chronicler Engine application code, its test suite, or its
  docs. The extension adds files under `.pi/extensions/` only.
- Pruning emitted reminders from context via the `context` event (v1 keeps
  them; see Task 2).
- Catching fast bursts (sub-interval runs with many tool calls). Accepted
  loss: the user judges fast bursts as not-necessarily-holes, and they are
  visible in the moment; the nudge targets the unattended grind.

## Acceptance criteria

- A run that exceeds the interval emits exactly one reminder per elapsed
  interval, never more.
- An idle session emits nothing — including sessions left open overnight
  (852-minute idle observed in real data).
- Tracker unit tests pass.
- The extension can be disabled (`SMOKE_BREAK_INTERVAL_MS=0`).
- Verified on a real long-running session that the nudge actually changes
  behaviour — the wording's test, and the only one that matters.

## Open questions — resolved

1. ~~Does `tool_result` reliably deliver an injected message to the model?~~
   **Answered by docs**: `tool_result` "Can modify result" and the patched
   result becomes the model-visible `toolResult` message (`docs/extensions.md`,
   Tool Events). End-to-end confirmation is step 3 of the smoke test.
2. ~~Time-based or turn-count trigger?~~ **Decided twice**: turn-count first
   (2026-09-15 morning), revised to elapsed time the same day after session
   data showed the confirmed holes are marathon-shaped (44 min and 2.6 h
   runs) while fast bursts are judged legitimate. 30 minutes is the interval;
   at that setting the build-chase run gets its nudge 13 minutes before the
   user had to intervene.
3. ~~Interval value~~ **Decided**: 30 minutes, from the user; adjustable via
   `SMOKE_BREAK_INTERVAL_MS`.
4. Does this conflict with the anti-analysis-paralysis instruction? No by
   construction: the wording's default is "simply continue"; remedial action
   is only considered if the model itself judges the work substantially off
   course, and nothing commands a stop. The prose rule stays untouched.
