# subagent-guardrails

Pi extension. Adds deterministic guardrails to subagent runs in this workspace.

## What it does

| # | Feature | Effect |
|---|---------|--------|
| 1 | Task-spec veto | Parent-side. Blocks `subagent` tool calls whose `task` is empty, too short, or missing a required header before any worker launches. |
| 2 | Time + turn budget | Forked subagents only. Steers the worker at 15 min / 50 turns (soft nudge) and 30 min / 100 turns (hard steer). |
| 3 | Role anchor | Forked subagents only. Appends two sentences to the system prompt pointing the worker at the literal `Task:\n` marker. |
| 4 | Git commit/push veto | Forked subagents only. Blocks `git commit`, `push`, `tag`, `merge`, `rebase` (and `hub <same>`) in `bash` calls. Read-only and staging ops (`add`, `status`, `diff`, `log`, `stash`, `fetch`, `pull`) remain allowed. |

## Install / dev

```bash
cd .pi/extensions/subagent-guardrails
npm install
npm test        # node:test + tsx
npm run build   # tsc --noEmit
```

Auto-discovered by pi (`.pi/extensions/*/index.ts`). No build step needed at runtime — pi uses jiti.

## Configuration

None yet. All thresholds are constants in `src/budget.ts`:

```ts
const SOFT_MINUTES = 15;
const SOFT_TURNS = 50;
const HARD_MINUTES = 30;
const HARD_TURNS = 100;
```

Task-spec floors are constants in `src/task-veto.ts`:

```ts
const WORKER_MIN_LENGTH = 800;
const DELEGATE_MIN_LENGTH = 200;
```

Git-veto verb set is the regex in `src/commit-veto.ts`:

```ts
const BLOCKED_GIT = /\b(?:git|hub)\s+(commit|push|tag|merge|rebase)\b/;
```

Edit the constants, `/reload` to apply.

## How detection works

Single source of truth: the `isFork` flag, set on `session_start` with `reason === "fork"`. Used by Features 2, 3, 4. Feature 1 is parent-side and skips when `isFork` is true. `session_start` always fires before `before_agent_start` / `turn_end` / `tool_call` per pi's lifecycle, so the flag is always set in time.

## Files

```
src/
├── index.ts          # ExtensionAPI registration, four handlers
├── task-veto.ts      # Feature 1
├── budget.ts         # Feature 2
├── role-anchor.ts    # Feature 3
├── commit-veto.ts    # Feature 4
└── *.test.ts          # node:test suite
```

## Background and design rationale

Design report and failure-mode analysis: [`chronicler_engine/docs/plans/subagent-guardrails-extension-plan.md`](../../../chronicler_engine/docs/plans/subagent-guardrails-extension-plan.md).

Key design points:

- **Deterministic checks only.** No story-point parsing, no spiral-pattern heuristics. Each check fires on a structural fact (string length, regex match, elapsed time).
- **Observer-first, veto-second.** Default behavior steers the worker. Hard blocks are reserved for structural failures (empty task, git history mutation).
- **Intercom-first.** Status pings route through the existing `contact_supervisor` channel; no new RPC or message bus.
- **No story-point budgets.** SP is too uncertain. Single tier of absolute time + turn thresholds for all subagent kinds.

## Limitations

- **No `shouldStopAfterTurn` on `ExtensionAPI`.** The hard steer relies on the worker obeying. For models that ignore it, the parent still gets the soft nudge and can manually `subagent:rpc:stop` the run.
- **Git veto is a regex, not a sandbox.** Obfuscated invocations (`git c""mmit`, aliases, `git -c` indirection) bypass it. Goal is to stop the common honest commit pattern, not defeat an adversarial worker.
- **No recursive `subagent` blocking.** The tool is never registered for workers; Qwen hallucinating a non-existent tool is already rejected at dispatch time.
