# subagent-guardrails

Pi extension. Adds deterministic guardrails to subagent runs in this workspace.

## What it does

| # | Feature | Effect |
|---|---------|--------|
| 1 | Task-spec veto | Parent-side. Blocks `subagent` tool calls whose `task` is empty, too short, or missing a required header before any worker launches. |
| 2 | Time + turn budget | Subagent sessions only. Steers the worker at 15 min / 50 turns (soft nudge) and 30 min / 100 turns (hard steer). |
| 3 | Role anchor | Subagent sessions only. Appends two sentences to the system prompt pointing the worker at the literal `Task:\n` marker. |
| 4 | Git commit/push/reset/stash veto | Subagent sessions only. Blocks `git commit`, `push`, `tag`, `merge`, `rebase`, `reset` (any form, incl. `--hard`), and `git stash` (any subcommand except `stash list`) in `bash` calls. `hub <same>` is also blocked. Read-only and staging ops (`add`, `status`, `diff`, `log`, `stash list`, `fetch`, `pull`) remain allowed. |

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

Git-veto verb set and stash carve-out are the regexes in `src/commit-veto.ts`:

```ts
const BLOCKED_GIT = /\b(?:git|hub)\s+(commit|push|tag|merge|rebase|reset|rm)\b/;
const STASH_ANY = /\b(?:git|hub)\s+stash\b/;
const STASH_LIST = /\b(?:git|hub)\s+stash\s+list\b/;
// stash blocks unless it matches STASH_LIST
```

Edit the constants, `/reload` to apply.

## How detection works

Single source of truth: the `isSubagent` flag, set on `session_start` via
layered detection (see `src/subagent-detection.ts`). Any-true wins:

1. `process.env.PI_SUBAGENT_CHILD === "1"` env var. Set by pi-subagents at
   child spawn. Empirically unreliable as the sole signal (env propagation
   gaps for async workers observed 2026-07-06). Kept for forward compat.
2. Session name starts with `subagent-` (pi-subagents names every spawned
   child session `subagent-{role}-{runId}-{index}`).
3. Session header has a `parentSession` field. Covers `/fork` and `/clone`
   too.

Signals 2 and 3 are read from `ctx.sessionManager` inside the
`session_start` handler, which fires before `before_agent_start` /
`turn_end` / `tool_call` per pi's lifecycle.

`/new` from the top-level `pi` shell (no parent session, no `subagent-`
name, env var unset) stays unguarded by Features 2/3/4 — that's the
interactive parent, by design.

Used by Features 2, 3, 4. Feature 1 is parent-side and skips when
`isSubagent` is true.

## Files

```
src/
├── index.ts                   # ExtensionAPI registration, four handlers
├── task-veto.ts               # Feature 1
├── budget.ts                  # Feature 2
├── role-anchor.ts             # Feature 3
├── commit-veto.ts             # Feature 4 (commit/push/tag/merge/rebase/reset/rm/stash)
├── subagent-detection.ts      # Layered isSubagentSession (env / name / parentSession)
└── *.test.ts                  # node:test suite
```

## Background and design rationale

Key design points:

- **Deterministic checks only.** No story-point parsing, no spiral-pattern heuristics. Each check fires on a structural fact (string length, regex match, elapsed time).
- **Observer-first, veto-second.** Default behavior steers the worker. Hard blocks are reserved for structural failures (empty task, git history mutation).
- **Intercom-first.** Status pings route through the existing `contact_supervisor` channel; no new RPC or message bus.
- **No story-point budgets.** SP is too uncertain. Single tier of absolute time + turn thresholds for all subagent kinds.

## Limitations

- **No `shouldStopAfterTurn` on `ExtensionAPI`.** The hard steer relies on the worker obeying. For models that ignore it, the parent still gets the soft nudge and can manually `subagent:rpc:stop` the run.
- **Git veto is a regex, not a sandbox.** Obfuscated invocations (`git c""mmit`, aliases, `git -c` indirection) bypass it. Goal is to stop the common honest commit pattern, not defeat an adversarial worker.
- **No recursive `subagent` blocking.** The tool is never registered for workers; Qwen hallucinating a non-existent tool is already rejected at dispatch time.
