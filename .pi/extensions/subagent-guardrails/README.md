# subagent-guardrails

Pi extension. Adds deterministic guardrails to subagent runs in this workspace.

## What it does

| # | Feature | Effect |
|---|---------|--------|
| 1 | Task-spec veto | Parent-side. Blocks `subagent` tool calls whose `task` is empty or too short before any worker launches. Length-only check; header markers were removed (pi-subagents does not naturally produce them in the task field). |
| 2 | Time + turn budget | Subagent sessions only. Steers the worker at 15 min / 50 turns (soft nudge) and 30 min / 100 turns (hard steer). |
| 3 | Role anchor | Subagent sessions only. Appends three rules to the system prompt: scope-rejection marker `[SCOPE_REJECTED]`, parent-only tool boundary, no scope expansion. Anchors on the literal `Task:\n` marker. |
| 4 | Git commit/push/reset/stash/checkout/restore veto | Subagent sessions only. Blocks `git commit`, `push`, `tag`, `merge`, `rebase`, `reset` (any form, incl. `--hard`), `git stash` (any subcommand except `stash list`), `git checkout` (every variant — branch switch, create, force-create, working-tree discard), and `git restore` (every variant — working tree, staged, source) in `bash` calls. `hub <same>` is also blocked. Read-only and staging ops (`add`, `status`, `diff`, `log`, `stash list`, `fetch`, `pull`) remain allowed. |

## Install / dev

```bash
cd .pi/extensions/subagent-guardrails
npm install
npm test        # node:test + tsx
npm run build   # tsc --noEmit
```

Auto-discovered by pi (`.pi/extensions/*/index.ts`). No build step needed at runtime — pi uses jiti.

## Configuration

All thresholds are constants in `src/budget.ts`:

```ts
const SOFT_MINUTES = 15;
const SOFT_TURNS = 50;
const HARD_MINUTES = 30;
const HARD_TURNS = 100;
```

Task-spec floors are constants in `src/task-veto.ts`:

```ts
const WORKER_MIN_LENGTH = 500;
const DELEGATE_MIN_LENGTH = 80;
```

Git-veto verb set and stash carve-out are the regexes in `src/commit-veto.ts`:

```ts
const BLOCKED_GIT = /\b(?:git|hub)\s+(commit|push|tag|merge|rebase|reset|rm|checkout|restore)\b/;
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
2. Session name starts with `subagent-` but is NOT `subagent-chat-*`
   (pi-subagents names spawned workers `subagent-{role}-{runId}-{index}`,
   but interactive parent sessions are also named `subagent-chat-{id}` —
   those are primaries, not subagents, so the `chat` segment is carved out
   to avoid misclassifying them; if a chat session is forked, signal 3
   still triggers correctly).
3. Session header has a `parentSession` field. Covers `/fork` and `/clone`
   too.

Signals 2 and 3 are read from `ctx.sessionManager` inside the
`session_start` handler, which fires before `before_agent_start` /
`turn_end` / `tool_call` per pi's lifecycle.

`/new` from the top-level `pi` shell (no parent session, no `subagent-`
name, env var unset) stays unguarded by Features 2/3/4 — that's the
interactive parent, by design.

Used by Features 2, 3, 4. Feature 1 fires on the parent side only (skips
when `isSubagent` is true).

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
- **Git veto is a regex, not a sandbox.** Flags between `git`/`hub` and the verb (`--no-pager`, `-c name=value`, `-C /path`, `--git-dir=...`) are caught. Still bypasses: shell-quoted command names (`git c""mmit`, `g""it commit`), git aliases (`commit = !./publish.sh`), and `hub` shimmed with a different binary name. Goal is to stop the common honest commit pattern, not defeat an adversarial worker.
- **Recursive `subagent` blocking is now explicit.** If a subagent session tries to call the `subagent` tool, the `tool_call` handler returns an explicit `block: true` with a clear reason ("Subagents cannot spawn sub-subagents. The subagent tool is parent-only. Implement the task directly using your own tools, or report back to the parent session if you are blocked on context or scope.") rather than letting the call fall through to pi-agent-core's generic "Tool subagent not found" error. The role-anchor already states this rule; the block is enforcement, not new policy.
- **No `[SCOPE_REJECTED]` parser.** The role-anchor instructs subagents to emit this marker on scope mismatch, but detection is on the parent's review pass, not automated.
