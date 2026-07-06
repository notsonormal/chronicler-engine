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
const BLOCKED_GIT = /\b(?:git|hub)\s+(commit|push|tag|merge|rebase|reset)\b/;
const STASH_ANY = /\b(?:git|hub)\s+stash\b/;
const STASH_LIST = /\b(?:git|hub)\s+stash\s+list\b/;
// stash blocks unless it matches STASH_LIST
```

Edit the constants, `/reload` to apply.

## How detection works

Single source of truth: the `isSubagent` flag, set on `session_start` from
`process.env.PI_SUBAGENT_CHILD === "1"`. The pi-subagents extension sets this
env var unconditionally in every spawned child process (sync, async, chain,
parallel, fanout — see pi-subagents `runs/shared/pi-args.ts`). Fresh-context
subagents fire `session_start` with `reason: "new"` and forked-context with
`reason: "fork"`; both set the env var, so either kind is detected. Used by
Features 2, 3, 4. Feature 1 is parent-side and skips when `isSubagent` is true.
`session_start` always fires before `before_agent_start` / `turn_end` /
`tool_call` per pi's lifecycle, so the flag is always set in time.

`/fork` and `/clone` slash commands create new sessions in the **same**
process via `sessionManager.newSession(...)` — they do not spawn a child or
set `PI_SUBAGENT_CHILD`. Therefore interactive manual forks remain unguarded
by Features 2/3/4. This matches the original design intent: only pi-subagents
extension children are workers.

`PI_SUBAGENT_PARENT_SESSION` is also set by pi-subagents but is **not** used
for detection — the parent-session field is written into headers by
`/fork` `/` `clone` slash commands too, so it is a weaker signal. The
`PI_SUBAGENT_CHILD` env var is the discriminator.

## Files

```
src/
├── index.ts          # ExtensionAPI registration, four handlers
├── task-veto.ts      # Feature 1
├── budget.ts         # Feature 2
├── role-anchor.ts    # Feature 3
├── commit-veto.ts    # Feature 4 (commit/push/tag/merge/rebase/reset/stash)
└── *.test.ts          # node:test suite
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
