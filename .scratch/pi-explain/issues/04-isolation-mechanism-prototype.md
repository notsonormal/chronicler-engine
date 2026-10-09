# 04 — Measure the isolation mechanism

Type: prototype
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Which mechanism keeps the artifact body out of the main transcript, at an
acceptable cost?

Charting settled the **invariant** — a few seconds of extra startup is fine, as
long as the artifact body never reaches the main transcript. It did **not**
settle the mechanism. The user answered "I don't know" on that question. This
ticket produces the evidence instead of guessing.

## Mechanisms to compare

- **Option A — no subagent.** The main agent emits a small spec. A bundled local
  renderer builds the artifact. The artifact body never reaches the model.
- **Option B — child `pi` process.** `pi --mode json -p --no-session
  --no-extensions --model <m>`. Full process isolation. Precedent: pi's own
  `examples/extensions/subagent`. Cost: 1–3 s startup, process management, and
  the child needs the same credentials and configuration.
- **Option C — in-process SDK session.** `createAgentSession()` with
  `SessionManager.inMemory()`. No process cost. Cost: shares the parent's
  process, needs a custom resource loader to stop extension recursion, and abort
  is messier.
- **Option D — reuse `pi-herdsman`.** Call `agent_delegate` through
  `ctx.executeTool`. Least code. Cost: a hard dependency on a third-party
  extension, plus its lifecycle and UI.

## What to measure

1. Tokens added to the **main** context per `/explain html`.
2. End-to-end latency beyond the model call.
3. Whether the mechanism survives an abort cleanly.
4. Implementation cost, in rough line count.

## Deliverable

A prototype extension plus a short comparison note. Link the note as an asset.
The note states which mechanism wins and why.

## Background

- The pollution comes from the artifact body, not from the act of delegating. A
  tool that writes HTML puts that HTML in the transcript as tool input. Any
  mechanism that produces the body elsewhere satisfies the invariant.
- pi has no built-in subagent. `pi-herdsman` is installed in this environment
  and provides `agent_delegate`, `agent_continue`, and related tools.
- `ctx.executeTool()` runs nested tool calls whose results do not become
  transcript entries.

## Recommendation

Do not pre-commit. Measure Option A against Option B. Treat Options C and D as
contingency, not as first choices.
