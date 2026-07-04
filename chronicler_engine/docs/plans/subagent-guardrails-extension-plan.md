# Plan: Pi Subagent Guardrails Extension

**Date:** 2026-07-02
**Status:** Planning
**Scope:** `.pi/extensions/subagent-guardrails/` (new pi extension, not `chronicler_engine` source)
**Investigation sources:** `/tmp/scoutA-findings.md`, `/tmp/scoutB-findings.md`, session-log analysis of 82 subagent runs (28 Jun – 2 Jul 2026)

> Note: This plan is filed under `chronicler_engine/docs/plans/` as a holding location
> because no `docs/plans/` exists at workspace root yet. The deliverable is a pi
> extension under `.pi/extensions/`, not Chronicler Engine source changes.

---

## Objective

Ship a small pi extension that addresses three recurring subagent failure modes
identified by the 2026-07-02 investigation of 82 subagent runs ($29.24 spend,
17.9% worker fail rate, 8.23 hrs wall-time). The extension is intentionally
narrow: each feature targets a failure class with a deterministic check, no
fuzzy heuristics, and no prediction of success.

The three failure modes addressed:

- **#11 — Missing / underspecified task spec.** Parent agent sometimes delegates
  with a near-empty `task` parameter, relying on the forked context to carry
  scope. Result: worker either stalls ("standing by") or implements the entire
  parent plan rather than its slice.
- **#12 — Long-run spirals on overscoped work.** Worker 1c669ced was assigned a
  2-item SP-1/3 task with explicit "Do NOT touch other items" constraint, still
  went rogue and ran 9 unrelated cleanup items for 48.7 min until exit 143.
  Reliable signal was wall-time, not turn-count or model compliance.
- **#6 — Fork-context role contamination.** Worker launched with `context:"fork"`
  (default for `worker`) inherited parent orchestrator framing. Worker emitted
  "Phase 1.3 worker launched... standing by" — mimicked parent stance instead of
  executing. Pi-subagents already injects `DEFAULT_FORK_PREAMBLE` and the
  `contact_supervisor` bridge instruction, so role reinforcement here is minimal,
  targeted at pointing to the `Task:\n` marker.

Failure modes **explicitly excluded** (no deterministic fix):

- 429 rate-limit bursts, async-runner process exits, exit 143 / timeout
  (infra, not extension-fixable)
- "No edits made" early exits (already-failed, cannot unfail)
- Silent test drops, out-of-scope deletions, acceptance-JSON drift
  (parent-side baseline comparison is the correct layer)
- Recursive `subagent` tool calls from workers
  (never registered for workers — Qwen hallucinated a non-existent tool,
  already rejected at dispatch time)

---

## Background

### Verified API surface (types in `@earendil-works/pi-coding-agent/dist/core/extensions/types.d.ts`)

- `ExtensionAPI.on("before_agent_start", handler)` — handler receives
  `BeforeAgentStartEvent { prompt, systemPrompt, systemPromptOptions }`; returns `BeforeAgentStartEventResult { systemPrompt?: string }`. Can inspect `event.prompt` to detect subagent context.
- `ExtensionAPI.on("tool_call", handler)` — `ToolCallEventResult`
  (`types.d.ts:753`) has `{ block?: boolean, reason?: string }`.
  `CustomToolCallEvent.toolName: string` — subagent tool surfaces
  as `toolName: "subagent"` with `input.task`, `input.agent`, etc. **Can veto.**
- `ExtensionAPI.on("turn_end", handler)` — observe-only, no result type.
  `TurnEndEvent` carries `turnIndex`, `timestamp`.
- `ExtensionAPI.on("session_start", handler)` — `reason` includes `"fork"`,
  plus `previousSessionFile?`.
- `ExtensionAPI.sendMessage(..., { deliverAs: "steer", triggerTurn: true })` — programmatic steer into the worker's next turn.
- `ExtensionAPI.appendEntry(customType, data)` — non-LLM state persistence
  (used to track `startTimeMs` across turns without polluting context).

### Verified pi-subagents behaviour (`pi-subagents/src/...`)

- `DEFAULT_FORK_PREAMBLE` (`shared/types.ts:937`) is wrapped around the task:
  ```
  {DEFAULT_FORK_PREAMBLE}

  Task:
  {user-provided task}
  ```
  The literal `Task:\n` marker is reliable for parsing task scope inside the
  combined prompt.
- Worker builtin defaults to `context:"fork"` (`subagent-executor.ts:1248`).
  Fork inherits parent conversation — useful for "why" context, but carries
  orchestrator framing that Qwen 3.5 struggles to override.
- pi-intercom bridge (`intercom/intercom-bridge.ts`) injects the
  `contact_supervisor` tool for forked subagents. Worker can already escalate
  via `contact_supervisor({ reason: "progress_update", ... })`. Parent receives
  the update as an inbound intercom message. 10-minute timeout on `ask`;
  `send` is fire-and-forget, no timeout.

### Investigation data anchoring thresholds

| Run | Task | Wall time | Failure |
|-----|------|-----------|---------|
| 1c669ced | D2+N16 (2 items, SP-1/3) | 48.7 min | exit 143 — went rogue on 9 items |
| 9a4a3de8 | Phase 2.2 (SP-3) | 25 min | completed successfully (294 turns, 5 s/turn) |
| 26fd6acb | — | — | exit 143, 0-byte output |
| cce7abfb | — | <2 min | "Second worker attempt" — fork contamination |

Wall-time is the dominant spiral signal: 1c669ced at 64 s/turn over 46 turns
would have bypassed any pure turn-count check; a 30-min wall-clock budget trips
reliably. Turn-count is a secondary sanity cap.

---

## Design

### Principles

1. **Deterministic checks only.** No story-point parsing of task text — too
   uncertain. No fuzzy path extraction. No "looks like a planning preamble"
   heuristics. A check either fires on a structural fact or it doesn't.
2. **Absolute thresholds, not adjusted-by-complexity.** One set of numbers
   for workers, one for delegates. Tuned from investigation data, not from
   a model of task difficulty.
3. **Observer-first, veto-second.** Default behavior is to steer the worker
   ("you have been running 30 min — summarise progress"). Hard block is
   reserved for the parent-side tool veto (`#11`), which prevents the failure
   before it starts.
4. **Intercom-first, not extension-broker.** Status pings go through the
   existing `contact_supervisor` channel that pi-subagents already injects.
   No new RPC, no new message bus.
5. **No new deps in the worker's tool graph beyond what pi-subagents
   already installs.** Extension writes into the system prompt and emits
   `sendMessage` steers. Worker doesn't need any new tool registration.

### Extension layout

```
.pi/extensions/subagent-guardrails/
├── package.json          # name: "subagent-guardrails", main: dist/index.js
├── tsconfig.json
├── src/
│   ├── index.ts          # ExtensionAPI registration, three handlers
│   ├── detect.ts         # "am I a forked subagent?" check
│   ├── task-veto.ts      # Feature 1 (#11)
│   ├── budget.ts         # Feature 2 (#12) — time/turn tracker + steer
│   └── role-anchor.ts    # Feature 3 (#6) — system-prompt append
└── README.md
```

Estimated ~120 lines of TypeScript total.

---

## Feature 1: Task-spec veto (fixes #11)

**Where:** `src/task-veto.ts` + registration in `src/index.ts`.

**Hook:** `ExtensionAPI.on("tool_call", ...)`.

**Filter:** Fires only for the parent session's call to tool `toolName === "subagent"` — verified by checking `event.toolName === "subagent"` and that the session is NOT itself a fork (see `detect.ts` below). The parent session is the one launching a subagent.

**Checks (in order, first failure vetoes):**

1. `input.task` is a string with `trim().length >= 200`. Below 200 chars is too
   thin for any worker or delegate contract seen in the data. (Smallest legit
   delegate task observed: 203 bytes including the `# Task for delegate` header
   and acceptance block.)
2. `input.task` contains one of the header markers observed in real
   contracts: `# Task for worker`, `# Task for delegate`, `# Task for scout`,
   `# Task for Explore`, `# Task for reviewer`, or literally `Task:`
   (the pi-subagents fork-preamble marker). Pick one form in the AGENTS.md
   template and enforce it.
3. If `input.agent === "worker"` (or undefined — worker is the most common),
   `input.task` length >= 800. Workers are 3–12 KB; anything under 800 chars
   almost always means the parent is leaning on fork context instead of
   writing the contract.

**Veto result:** `return { block: true, reason: "Subagent task spec failed validation: <which check>. Re-write the task per the AGENTS.md worker/delegate template before delegating." }`

This turns the tool call into a tool error the parent must address before
retrying. Parent gets immediate signal in its own turn — no worker launch
happens, no tokens spent.

**Risk:** false-positive on a legitimately tiny delegate task. Mitigation:
the delegate threshold (200 chars + header) is below the smallest observed
real delegate contract. Workers get the tighter 800-char floor.

---

## Feature 2: Time + turn budget (fixes #12)

**Where:** `src/budget.ts` + registration in `src/index.ts`.

**Hook:** `ExtensionAPI.on("session_start", ...)` records `Date.now()` and
`turnCount = 0` via `pi.appendEntry("subagent-guardrails:state", {...})`.
`ExtensionAPI.on("turn_end", ...)` increments turn count and reads elapsed.

**Filter:** Only fires for forked subagent sessions (see `detect.ts`).

**Thresholds (absolute, tuned from investigation data):**

| Session type | Soft nudge | Hard steer |
|--------------|------------|------------|
| worker       | 15 min OR 50 turns | 30 min OR 100 turns |
| delegate     | 5 min OR 20 turns | 10 min OR 40 turns |
| other (scout, reviewer, etc.) | 10 min OR 30 turns | 20 min OR 60 turns |

Rationale:

- 1c669ced tripped at the worker hard-steer (48.7 min > 30 min). Status ping
  would have fired at 15 min, well before the 49-min kill.
- 9a4a3de8 (legit SP-3, 25 min, completed) trips soft nudge at 15 min but
  finishes before the 30-min hard steer. Acceptable: a one-time nudge is not
  a failure.
- Delegate runs are nearly all under 5 min in the data; 5-min soft nudge is
  conservative.

**Soft nudge action:** `pi.sendMessage({ customType: "subagent-guardrails:nudge", content: "You have been running for {N} min over {M} turns. Send a one-line progress summary to your supervisor via contact_supervisor with reason 'progress_update', then continue unless the task is genuinely larger than 1 story point.", display: true }, { deliverAs: "steer", triggerTurn: true })`

This uses pi-intercom's existing bridge. It does not invent a new channel. The
worker already has `contact_supervisor` — it just isn't being prompted to use
it periodically.

**Hard steer action:** `pi.sendMessage({ customType: "subagent-guardrails:stop", content: "BUDGET EXCEEDED ({N} min / {M} turns). Stop all work. Return a focused summary of what is complete and incomplete, plus the next concrete step. Do not start a new subtask.", display: true }, { deliverAs: "steer", triggerTurn: true })`

Steer cannot forcibly terminate the agent (no `shouldStopAfterTurn` is exposed
on `ExtensionAPI`). The hard steer relies on the worker obeying the instruction
to stop and summarise. For Qwen 3.5 this is observed-effective in the data —
explicit "stop now" steers were followed. For models that ignore it, the parent
still gets the 15-min ping and can manually `subagent:rpc:stop` the run.

**What this feature does NOT do:**

- Does not parse story points from the task. The original SP-tiered budget
  proposal is dropped — story points are too uncertain as a budget signal.
- Does not try to detect spiral patterns (no tool calls in last N turns, same
  bash > 60 s, etc.). Those were investigated and rejected as too
  false-positive-prone on legit 50–294-turn refactors.
- Does not block individual tool calls after budget. Only steers.

### Status-ping interplay with pi-intercom

The soft nudge explicitly tells the worker to use `contact_supervisor`. This
is the fully-automatic path the user asked about:
extension detects elapsed -> extension steers worker -> worker calls
`contact_supervisor` -> parent receives intercom message with run metadata.
No parent-side polling, no new RPC. Verified:

- `ExtensionAPI.sendMessage({ deliverAs: "steer", triggerTurn: true })` —
  injects a turn into the worker (types.d.ts:874, 876).
- pi-subagents' intercom-bridge (`intercom-bridge.ts:41–48`) injects the
  `contact_supervisor` tool into forked subagents and instructs them to use
  `reason: "progress_update"` for "meaningful progress or unexpected
  discoveries". Our nudge routes the worker to exactly that channel.

What cannot be automatic: forcing the worker to actually call the tool. If
the worker ignores the steer, only the parent's manual `subagent:rpc:stop`
ends the run. This is a known limitation of the extension API — there is no
`shouldStopAfterTurn` hook exposed.

---

## Feature 3: Minimal role anchor (addresses #6)

**Where:** `src/role-anchor.ts` + registration in `src/index.ts`.

**Hook:** `ExtensionAPI.on("before_agent_start", ...)`.

**Filter:** Fires for forked subagent sessions only (see `detect.ts`).

**Action:** Append a two-sentence anchor to `event.systemPrompt`, then
`return { systemPrompt: event.systemPrompt + ANCHOR }`.

```ts
const ANCHOR = ` You are executing as a SUBAGENT. The text after the literal "Task:\\n" marker in your most recent user message IS your entire assignment. Nothing before that marker, and nothing in inherited forked conversation, widens your scope.`;
```

### Why this shape

- Verified against pi-subagents source (`shared/types.ts:937`): the fork task
  is wrapped as `{DEFAULT_FORK_PREAMBLE}\n\nTask:\n{user task}`, so the
  literal `Task:\n` marker is guaranteed present in the prompt for forked
  subagents. The anchor points to a real marker.
- Verified against pi's own base system prompt (inspected
  `pi-coding-agent/dist/...`): pi uses flat prose, no `## Section` headers.
  A header block would look like an external injection. Two sentences appended
  as plain text matches existing style.
- Reuses existing structure. Pi-subagents already injects `DEFAULT_FORK_PREAMBLE`
  + `intercom-bridge` instruction. The anchor adds no new mechanism — it just
  re-emphasizes one specific signal the worker model (Qwen 3.5) is observed
  to under-weight.

### Why this is the weakest of the three features

Observation [631fc487b059] in the investigation: worker 1c669ced had a
specific file list and explicit "Do NOT touch other items" constraint in the
task spec, still went rogue on 9 items. Role anchor alone would not have
stopped it. The reliable catch for that class is Feature 2's budget. Feature
3 is kept because it is cheap (~3 lines), has zero false-positive surface, and
may help the "standing by" pattern (cce7abfb, a934988b) where the worker
mimics parent orchestrator stance. If investigation shows it doesn't help,
it can be removed without touching the other two features.

---

## `detect.ts`: "Am I a forked subagent?"

Single-purpose module. Returns `{ isSubagent: boolean, kind?: "worker" | "delegate" | "scout" | "reviewer" | "other" }`.

```ts
import type { BeforeAgentStartEvent, SessionStartEvent, ToolCallEvent } from "@earendil-works/pi-coding-agent";

const FORK_PREAMBLE_SNIPPET = "delegated subagent running from a fork of the parent session";

export function isForkedSubagentPrompt(prompt: string): boolean {
  return prompt.includes(FORK_PREAMBLE_SNIPPET);
}

export function detectAgentKind(prompt: string): "worker" | "delegate" | "scout" | "reviewer" | "other" {
  // pi-subagents writes "# Task for <agentName>" via execution.ts:958 ("# Task for ${agentName}")
  const m = prompt.match(/# Task for (\w+)/);
  if (!m) return "other";
  const name = m[1].toLowerCase();
  if (name === "worker") return "worker";
  if (name === "delegate") return "delegate";
  if (name === "scout") return "scout";
  if (name === "reviewer" || name === "oracle" || name === "planner" || name === "researcher") return "reviewer";
  return "other";
}
```

The detection is text-based against the fork preamble pi-subagents injects,
NOT against an `<active_agent>` tag — that tag does not exist in the current
pi-subagents version (verified by grep).

---

## Registration (`src/index.ts`)

```ts
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { isForkedSubagentPrompt, detectAgentKind } from "./detect";
import { checkTaskSpec } from "./task-veto";
import { onSessionStart, onTurnEnd } from "./budget";
import { ROLE_ANCHOR } from "./role-anchor";

export default function (pi: ExtensionAPI) {
  // Feature 1: parent-side task-spec veto
  pi.on("tool_call", async (event) => {
    if (event.toolName !== "subagent") return;
    // parent session is NOT a forked subagent — skip if we detect fork preamble
    // in our own context. (Parent has its own prompt; check via session info.)
    const result = checkTaskSpec(event.input);
    if (result.block) return result;
  });

  // Feature 2: budget tracker
  pi.on("session_start", (event) => {
    if (event.reason !== "fork") return;
    onSessionStart(pi);
  });
  pi.on("turn_end", () => {
    if (!isForkedSubagentPrompt(/* current prompt */ "")) return;
    onTurnEnd(pi);
  });

  // Feature 3: role anchor
  pi.on("before_agent_start", async (event) => {
    if (!isForkedSubagentPrompt(event.prompt)) return;
    return { systemPrompt: event.systemPrompt + ROLE_ANCHOR };
  });
}
```

Note: getting "the current prompt" inside `turn_end` requires either caching
the last `before_agent_start` prompt in extension state or re-reading from the
session. Implementation detail to resolve during build — `appendEntry` +
`getEntries` or a module-level closure both work in practice.

---

## Acceptance

### Feature 1 (#11)

- Empty `subagent({ task: "" })` call is blocked with reason.
- 100-char task is blocked.
- Task missing `# Task for ...` or literal `Task:` marker is blocked.
- `agent: "worker"` with 500-char task is blocked.
- Real worker contract (paste from `9a4a3de8_worker_0_input.md`) passes.
- Real delegate contract (paste from `3592b5cc_delegate_0_input.md`) passes.

### Feature 2 (#12)

- Worker run past 15 min emits one nudge steer containing the elapsed time.
- Worker run past 30 min emits one hard-stop steer.
- Nudge fires at most once per threshold (state tracked via `appendEntry`).
- Delegate thresholds fire at 5 / 10 min.
- Non-subagent sessions never fire.
- Replaying 1c669ced's elapsed pattern (48.7 min) fires soft nudge at 15 min
  and hard steer at 30 min.

### Feature 3 (#6)

- Forked subagent system prompt ends with the two-sentence anchor.
- Parent (non-fork) session system prompt is unchanged.
- Anchor text matches the verified form exactly (no markdown header).

### Whole-extension

- `python build.py` (chronicler_engine) still passes — extension is a pi
  plugin, not engine source. Build must be unaffected.
- Loading the extension in a fresh pi session logs no errors.
- A worker subagent run completes normally and the worker's system prompt
  (visible in the subagent artifact meta) contains the anchor.

---

## Out of scope (intentionally)

- Story-point-based budget thresholds. Dropped per user direction: SP is
  too uncertain. Budget uses absolute time + absolute turn count.
- Recursive-`subagent`-call blocking. Not extension-fixable — the tool was
  never registered for workers; the "bug" was Qwen hallucinating a
  non-existent tool, already rejected at dispatch time.
- No-edits / out-of-scope / silent-test-drop recovery. Cannot unfail a
  finished run. Parent-side baseline comparison is the right layer.
- Async-runner stability / 429 rate limits. Infra, not extension.
- Per-run model selection or worker-prompt rewriting beyond the anchor.

---

## Open questions for review

1. **Status-ping cadence.** Proposal: soft nudge fires once at 15 min.
   Alternative: repeat every 15 min (15, 30, 45). The latter is more
   informative for the parent but risks nagging on legit long tasks. Default
   to once; revisit after the first week of data.
2. **Hard-steer compliance.** No `shouldStopAfterTurn` on `ExtensionAPI` —
   the hard steer is honored only if the worker obeys. If the worker
   keeps working after the steer, do we want a parent-side 5-min grace
   timer that auto-emits `subagent:rpc:stop`? That would require the
   extension running in the parent session with RPC access — different
   scope. Defer.
3. **AGENTS.md template update.** Should the worker/delegate task
   template in `AGENTS.md` be updated to require `# Task for worker`
   header + `Complexity: N story points` line regardless of whether the
   extension enforces it? The SP line was dropped from budget logic but
   may still serve as a lightweight role-anchor signal for the model.
   Out of scope for the extension; tracked separately.
4. **Where does the extension live long-term?** Filed under
   `.pi/extensions/` (project-local) vs contributed upstream to
   pi-subagents as an optional module. Defer until after local validation.

---

## Followups (post-ship, tracked but not part of this plan)

- Track whether Feature 3 reduces "standing by" / "launched" placeholder
  outputs. If no measurable effect after 2 weeks, remove Feature 3.
- Track whether the 15-min soft nudge actually triggers a
  `contact_supervisor` progress update from the worker. If workers
  consistently ignore it, consider promoting Feature 2's hard-steer
  threshold down (or adding parent-side RPC stop with a grace timer).
- After 1 month of data, re-evaluate the absolute thresholds against the
  post-extension run distribution. May need to tighten worker to 25 min
  if 1c669ced-class rogues still slip through.
