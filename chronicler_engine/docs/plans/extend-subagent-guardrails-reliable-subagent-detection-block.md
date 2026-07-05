# Extend subagent-guardrails: reliable subagent detection + block `git reset` & `git stash`

## Summary

Two fixes in `subagent-guardrails`:

1. **Detection bug.** `isFork` (set on `session_start` with `reason === "fork"`) only catches forked-context subagents. Fresh-context subagents (SINGLE, PARALLEL, async, chain, worktree) launch with `reason: "new"` → `isFork` stays false → Features 2/3/4 silently skipped for them. Replace with reliable signal: `process.env.PI_SUBAGENT_CHILD === "1"`, set by the pi-subagents extension in every spawned child via `runs/shared/pi-args.ts:164`. Top-level pi never sets it; `/fork` and `/clone` slash commands share the parent process so they don't set it either — only true subagent runs match.

2. **Git-veto gap.** Extend `BLOCKED_GIT` verbs to include `reset` (all forms incl. `--hard`, bare). Add `git stash` handling: block every stash subcommand except `git stash list` (the only read-only form). Workers cannot move HEAD or mutate stash; parent retains commit-and-push + history ownership per `AGENTS.md`.

Net effect: every subagent created by pi-subagents — regardless of context mode (fresh/fork), run style (SINGLE/CHAIN/PARALLEL), or sync/async — gets the budget tracker, role anchor, and the full git-veto (commit/push/tag/merge/rebase/**reset**/**stash**, minus `stash list`).

## Key Changes

- `src/index.ts`: rename `isFork` → `isSubagent`. Set in `session_start` from `process.env.PI_SUBAGENT_CHILD === "1"` (independent of `event.reason`). Skip Feature 1 when `isSubagent`. Gate Features 2/3/4 on `isSubagent`.
- `src/commit-veto.ts`: extend `BLOCKED_GIT` to include `reset`. Add `STASH_ANY` / `STASH_LIST` regexes. Block `git stash` (all forms) except `git stash list`.
- `src/commit-veto.test.ts`: new cases for `reset` (incl. `--hard`, `--soft`, bare, `HEAD~N`) and `stash` (bare, `push/pop/apply/drop/clear/save`, `hub stash`); `git stash list` stays in `ALLOWED`.
- `src/budget.ts` (export `onSessionStart`/`onTurnEnd` still pure): no logic change; `index.ts` now calls them for any subagent.
- `src/role-anchor.ts`: no change.
- `README.md`: update Feature 4 row + regex snapshot, replace "forked subagents only" framing with "subagent sessions (any pi-subagents run)" in Features 2/3/4 descriptions, document detection source (`PI_SUBAGENT_CHILD` env var).

## Implementation

Single phase. Reliability fix ships in same edit as veto extension — both touch `commit-veto.ts`/`index.ts` and share a `npm test`/`npm run build` verify gate.

### Phase 1: Detection + veto

- [ ] #### Task 1.1: Switch detection from `isFork` to `isSubagent` (3 SP)
  - File: `.pi/extensions/subagent-guardrails/src/index.ts`.
  - Replace `let isFork = false;` with `let isSubagent = false;`. In `session_start` handler: `isSubagent = process.env.PI_SUBAGENT_CHILD === "1";` (do NOT branch on `event.reason` — log/metric only). `onSessionStart()` (budget reset) still called when `isSubagent`.
  - Feature 1 (task-veto) handler: `if (isSubagent) return;` (was `if (isFork) return;`).
  - Features 2 (`turn_end`), 3 (`before_agent_start`), 4 (git veto): `if (!isSubagent) return;` (was `if (!isFork) return;`).
  - Update header comment block.
  - Verify: `npm run build` clean; `npm test` still green (no tests touch index.ts directly).
  - Note: 5 SP-tier task done by primary agent, not delegated — verification load on primary.

- [ ] #### Task 1.2: Extend `commit-veto.ts` (1 SP)
  - File: `.pi/extensions/subagent-guardrails/src/commit-veto.ts`.
  - Replace `BLOCKED_GIT` with `/\b(?:git|hub)\s+(commit|push|tag|merge|rebase|reset)\b/`.
  - Add `STASH_ANY = /\b(?:git|hub)\s+stash\b/` and `STASH_LIST = /\b(?:git|hub)\s+stash\s+list\b/`.
  - New logic: verb-match → block with `git ${verb}`; else `STASH_ANY.test && !STASH_LIST.test` → block with `verb = "stash"`; else pass.
  - Reason string reused verbatim: `"git ${verb} blocked in subagent context. Workers must not mutate repository history or push. Return a summary of staged or local changes and let the parent session commit via the commit-and-push skill."`
  - Verify: `npm run build` clean.

- [ ] #### Task 1.3: Extend `commit-veto.test.ts` (1 SP)
  - Add `reset` to `BLOCKED` iteration.
  - Add explicit `git reset --hard`, `git reset --soft HEAD~1`, bare `git reset`, `git reset HEAD~2` blocked cases.
  - Add stash blocked cases: bare `git stash`, `stash push -m x`, `stash pop`, `stash apply`, `stash drop`, `stash clear`, `stash save`, `hub stash`.
  - Add `git stash list` to `ALLOWED` (remove existing `git stash` from `ALLOWED`).
  - Add explicit assertion: `git stash list` passes, `git stash` (bare) blocks.
  - Add `hub reset --hard` blocked case.
  - Verify: `npm test` all green.

- [ ] #### Task 1.4: Update `README.md` (1 SP)
  - File: `.pi/extensions/subagent-guardrails/README.md`.
  - Feature 4 row: verbs list `commit|push|tag|merge|rebase|reset|stash` (excl. `stash list`).
  - "How detection works": replace `isFork`/`reason === "fork"` explanation with `isSubagent`/`process.env.PI_SUBAGENT_CHILD === "1"`. Note that `/fork` + `/clone` slash commands do **not** set this env var, so manual interactive forks remain unguarded by Features 2/3/4 (matches original intent — only pi-subagents extension children are guarded).
  - "Git-veto verb set" snapshot: show new regex + stash-list carve-out regexes.
  - Features 2/3/4 phrasing: "forked subagents only" → "subagent sessions (any pi-subagents child run)".
  - Limitations section: no change (regex-not-sandbox note still holds).

## Test Plan

- `cd .pi/extensions/subagent-guardrails && npm test` — full suite green (existing BLOCKED/ALLOWED + new reset + stash cases).
- `npm run build` — `tsc --noEmit` clean.
- Manual regression (post-plan, outside Plan Mode):
  - `pi /reload`; launch a `worker` subagent with `context: "fresh"`.
  - In worker, run `git reset --hard` → expect block reason returned.
  - Run `git stash` (bare) → expect block; run `git stash list` → passes.
  - Run `git commit -m x` → still blocks (regression).
  - Launch same `worker` with default forked context; confirm identical blocks.
  - In top-level pi (parent session), run `git reset --hard` → must still pass (parent retains full git ownership). Verifies `isSubagent` discrimination.
  - Optional: confirm `/fork` slash-command session still allows `git commit` (interactive fork isn't a subagent — matches original design intent).

## Assumptions

- `PI_SUBAGENT_CHILD=1` is the canonical subagent marker, used by pi-subagents itself in `doctor.ts:180` and set unconditionally in `runs/shared/pi-args.ts:164` for every spawn path (sync, async, chain, parallel, fanout). No path spawns a subagent without it.
- `/fork` and `/clone` slash commands create new sessions in the **same process** via `sessionManager.newSession(...)` — they do not spawn a child or set `PI_SUBAGENT_CHILD`. Therefore they remain unguarded by Features 2/3/4. This matches the original design's intent (only pi-subagents extension children are workers).
- `PI_SUBAGENT_PARENT_SESSION` is also set but is **not** used for detection — the parent-session field is also written into headers by `/fork`/`/clone` slash commands, so it's a weaker signal. The env var is the discriminator.
- Bare `git reset` and bare `git stash` (= `push`) blocked, per user answer "Block all reset" + "Allow stash list" only.
- `git stash show` (read-only) intentionally blocked — user only carved out `list`. Revisit in separate plan if workers need it.
- Module-scoped mutable flag acceptable — pi rebinds extensions on `session_shutdown`/`session_start` so the flag is freshly set per session (same lifetime model as current `isFork`).
- No config surface added (consistent with existing "thresholds are constants" design note).
- Obfuscated invocations (`git st""ash`, `git -c alias.x=stash x`) still bypass — acceptable per existing "regex, not sandbox" limitation note in README.

## Optional grill

Stress-test this plan with `/grill-me-with-docs` before implementing if desired. Not auto-invoked.
