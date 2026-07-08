# Block `git checkout` and `git restore` (entire commands) in both extensions

## Summary

The actual wipe on 2026-07-08T14:55:20.765Z was `git checkout -- .pi/extensions/subagent-guardrails/` run by worker session 019f4235 (L2918). Per user direction: block ALL `git checkout` and ALL `git restore` (no surgical carve-outs) in both extensions. Layered defense: pi-permissions covers primary + subagent sessions, subagent-guardrails covers subagents only.

Side-effect: existing `git-checkout-branch` rule becomes redundant (always shadowed by `git-checkout`). Removed for cleanliness.

## Key Changes

| Extension | File | Change |
|-----------|------|--------|
| pi-permissions | `src/rules.ts` | Add `git-checkout`, `git-restore`; remove `git-checkout-branch` (redundant) |
| pi-permissions | `.pi/permissions.json` | Add 3 keys (`git-checkout`, `git-restore`, `git-rm` drift fix); remove `git-checkout-branch` |
| pi-permissions | `src/matcher.test.ts` | Add positives + dedicated tests; remove `git checkout feature` from benign; remove `git-checkout-branch` positive |
| pi-permissions | `README.md` | Replace `git-checkout-branch` row with `git-checkout` and `git-restore` rows |
| subagent-guardrails | `src/commit-veto.ts` | Add `checkout\|restore` to `BLOCKED_GIT` verb list |
| subagent-guardrails | `src/commit-veto.test.ts` | Extend `BLOCKED` const; add explicit variant tests (branch switch, create, discard, staged, source, hub variants) |
| subagent-guardrails | `README.md` | Update Feature 4 row + verb-set code block |

**Commit strategy: 1 combined commit** covering both extensions and the existing uncommitted subagent-guardrails carve-out work.

## Implementation

### Phase 1: pi-permissions (3 SP)

- [ ] #### Task 1.1: Add base-command rules + tests + README + sync permissions.json (3 SP)
  - In `src/rules.ts` `DEFAULT_RULES`:
    - **Remove**: `{ name: "git-checkout-branch", pattern: "\\bgit\\s+checkout\\s+-B\\b" }` (now redundant)
    - **Add**: `{ name: "git-checkout", pattern: "\\bgit\\s+checkout\\b" }`
    - **Add**: `{ name: "git-restore", pattern: "\\bgit\\s+restore\\b" }`
  - In `.pi/permissions.json`:
    - **Remove**: `"git-checkout-branch"` key
    - **Add**: `"git-rm": "\\bgit\\s+rm\\b"` (fix existing drift — DEFAULT_RULES has it but file does not)
    - **Add**: `"git-checkout": "\\bgit\\s+checkout\\b"`
    - **Add**: `"git-restore": "\\bgit\\s+restore\\b"`
  - In `src/matcher.test.ts`:
    - **Modify** the `positives` array in "matcher: each default rule matches its positive case":
      - Remove `["git-checkout-branch", "git checkout -B main feature"]` (rule removed)
      - Add 5 `git-checkout` positives: `git checkout feature`, `git checkout -b new-branch`, `git checkout -B main feature`, `git checkout -- file.txt`, `git checkout HEAD -- file.txt`
      - Add 3 `git-restore` positives: `git restore file.txt`, `git restore --staged file.txt`, `git restore --source=HEAD file.txt`
    - **Modify** the `benign` array: remove `"git checkout feature"` (now blocked by `git-checkout`). All other benign entries stay (none reference `git restore`).
    - **Add** 2 new dedicated tests:
      - `"git-checkout blocks every variant (switch, create, force-create, discard)"` asserting: `git checkout feature` → "git-checkout", `git checkout -b new` → "git-checkout", `git checkout -B main feature` → "git-checkout", `git checkout -- file.txt` → "git-checkout", `git checkout HEAD -- file.txt` → "git-checkout"
      - `"git-restore blocks every variant (working tree, staged, source)"` asserting: `git restore file.txt` → "git-restore", `git restore --staged file.txt` → "git-restore", `git restore --source=HEAD file.txt` → "git-restore", `git restore .` → "git-restore"
  - In `README.md`:
    - Remove the `git-checkout-branch` row from the default-rules table
    - Add 2 rows:
      - `git-checkout` | `\bgit\s+checkout\b` | Any checkout (branch switch, create, force-create, working-tree discard). Use `/permissions grant git-checkout` for session-level bypass.
      - `git-restore` | `\bgit\s+restore\b` | Any restore (working tree, staged, source). Use `/permissions grant git-restore` for session-level bypass.

**Verification per task**:
- `cd .pi/extensions/pi-permissions && npx tsc --noEmit` → exit 0
- `cd .pi/extensions/pi-permissions && npm test` → 13/13 pass (11 existing - 0 [array modifications don't change test count] + 2 new dedicated tests).
- `python3 -c "import json; json.load(open('.pi/permissions.json'))"` → no error.

### Phase 2: subagent-guardrails (3 SP)

- [ ] #### Task 2.1: Extend commit-veto + tests + README (3 SP)
  - In `src/commit-veto.ts`:
    - **Modify** `BLOCKED_GIT`: change `\b(?:git|hub)\s+(commit|push|tag|merge|rebase|reset|rm)\b` → `\b(?:git|hub)\s+(commit|push|tag|merge|rebase|reset|rm|checkout|restore)\b`
    - **Modify** `REASON_SUFFIX`: change `"Workers must not mutate repository history or push."` → `"Workers must not mutate repository history, change branches, restore working tree, or push."`
    - **No new constants or new if-branches needed.** The existing single verb-regex handles all 9 verbs uniformly.
  - In `src/commit-veto.test.ts`:
    - **Modify** `BLOCKED` const: change `["commit", "push", "tag", "merge", "rebase", "reset", "rm"]` → `["commit", "push", "tag", "merge", "rebase", "reset", "rm", "checkout", "restore"]`. The auto-generated loop adds 2 new tests (one per verb) with no other change.
    - **Add** explicit variant tests (8 total):
      - 4 `git checkout` variants: `git checkout feature`, `git checkout -b foo`, `git checkout -B foo bar`, `git checkout -- file.txt` — all blocked
      - 3 `git restore` variants: `git restore file.txt`, `git restore --staged file.txt`, `git restore --source=HEAD file.txt` — all blocked (no carve-outs)
      - 2 hub variants: `hub checkout feature`, `hub restore file.txt` — both blocked (hub parity)
    - **Add** 1 reason-text test: assert that `git checkout feature`, `git restore file.txt` both produce a reason matching `/change branches/` or `/restore working tree/` (proves the new REASON_SUFFIX text).
    - **Total new tests: 11** (76 → 87 post-fix)
  - In `README.md`:
    - Update Feature 4 row: "Blocks `git commit`, `push`, `tag`, `merge`, `rebase`, `reset`, `rm`, `checkout`, `restore` (any form, incl. `--hard` for reset, `git checkout --` for working-tree discard, and `git restore` for staged/working-tree/source variants)" → "Blocks `git commit`, `push`, `tag`, `merge`, `rebase`, `reset` (any form, incl. `--hard`), `git stash` (any subcommand except `stash list`), `git checkout` (any variant), and `git restore` (any variant) in `bash` calls. `hub <same>` is also blocked."
    - Update the verb-set code block to include `checkout|restore` in the BLOCKED_GIT regex.

**Verification per task**:
- `cd .pi/extensions/subagent-guardrails && npx tsc --noEmit` → exit 0
- `cd .pi/extensions/subagent-guardrails && npm test` → 87/87 pass (76 existing + 11 new).
- Manual smoke check: `cd .pi/extensions/subagent-guardrails && node --import tsx -e "import('./src/commit-veto.ts').then(m => console.log(m.checkGitVeto('git checkout feature')))"` → must return `{ block: true, reason: "git checkout blocked in subagent context. Workers must not mutate repository history, change branches, restore working tree, or push. Return a summary of staged or local changes and let the parent session commit via the commit-and-push skill." }`.

### Phase 3: Commit + push (1 SP)

- [ ] #### Task 3.1: Single combined commit + push (1 SP)
  - All changes from Phases 1 and 2 in ONE commit.
  - Commit message format: `fix(extensions): block git checkout + git restore (whole commands) to prevent working-tree wipe`
  - Body must reference the wipe incident (worker session 019f4235, 2026-07-08T14:55:20, root cause `git checkout -- .pi/extensions/subagent-guardrails/` not blocked) and list both extensions + the `git-rm` drift fix + the `git-checkout-branch` rule removal.
  - Run `commit-and-push` skill (per AGENTS.md / .agents/skills).

## Test Plan

| Extension | Pre-fix | Post-fix | New tests |
|-----------|---------|----------|-----------|
| pi-permissions | 11 | 13 | +2 dedicated; in-place modifications to existing arrays |
| subagent-guardrails | 76 | 87 | +11 (2 auto from BLOCKED loop, 9 explicit variants) |

Both extensions use `npm test` (node:test + tsx). Both must pass before commit.

## Per Task Validation Steps

After each task:
1. `npx tsc --noEmit` in extension dir → exit 0
2. `npm test` in extension dir → all tests pass
3. (Phase 2) Manual smoke check via `node --import tsx -e "..."` confirms new block fires

After Phase 3:
4. `git log -1` shows expected commit message and author
5. `git status` clean (no uncommitted leftovers)
6. `git diff HEAD~1 --stat` shows ~7 files modified across both extensions

## Assumptions

- **Worker task force = `worker` subagent (3 SP each, sequential per phase).** Tasks could be parallel (no shared files), but 3 SP cost is small enough to do sequentially. Total = 1 commit at end (per Phase 3).
- **Strict blocking chosen over surgical** per user direction (skill review #5). All `git checkout` and `git restore` invocations are blocked, no carve-outs. Workers cannot branch-switch, create branches, or do any restore operation; parent owns all branch ops and un-staging.
- **Existing `git-checkout-branch` rule removed** as redundant (always shadowed by the new `git-checkout` rule's first-match-wins insertion order). Removal keeps the rule list clean.
- **Existing uncommitted subagent-guardrails work (cf6999e carve-out)** gets included in the single Phase 3 commit. Its content is already verified (76/76 tests pass per prior observation aa407188fcae). No additional verification beyond `npm test` at commit time.
- **`.pi/permissions.json` drift fix for `git-rm`** is in scope. One-line addition; rule already in DEFAULT_RULES.
- **No CHANGELOG / ADR** for this change. Pure addition of deny-list rules; no API change.
- **For primary users:** `git checkout feature` and `git restore --staged file.txt` will trigger the confirm dialog each time. Users can `/permissions grant git-checkout` or `/permissions grant git-restore` once to bypass for the session. This is intentional friction — confirms the design choice.

## NOT in scope

- `.pi/permissions.json` drift in OTHER files (only `git-rm` drift in this file is fixed)
- Hub variant of `git restore --staged` carve-out (no carve-outs in strict mode)
- Audit log persistence (pi-permissions already has in-session ring buffer only)
- Pre-commit hooks / CI integration
- Carve-out for legitimate `git restore --staged` (per user direction: strict in both)
- Carve-out for legitimate `git checkout feature` branch switch (per user direction: strict in both)

## What already exists (reuse, don't reinvent)

- `pi-permissions/src/matcher.ts:firstMatchingRule` — reused as-is. New rules just register patterns.
- `pi-permissions/src/index.ts` tool_call handler — already wired, no handler changes needed.
- `subagent-guardrails/src/commit-veto.ts:checkGitVeto` — already supports arbitrary verb list in `BLOCKED_GIT`. Adding `checkout|restore` is a 9-character change.
- `subagent-guardrails/src/index.ts` bash tool_call handler — already wired.
- README table conventions in both extensions — extend/replace existing tables.
- Test patterns: `node:test` + `assert/strict` — reuse existing import style.
- BLOCKED verb loop in commit-veto.test.ts — adding verbs to BLOCKED auto-generates tests.

## Failure modes

| Codepath | Failure | Handling |
|----------|---------|----------|
| New rule fires false positive (legitimate cmd blocked) | User has to confirm via `ctx.ui.confirm` for primary; hard block for subagent | Standard UX; session grant bypasses for primary |
| New rule has regex bug (destructive cmd slips through) | Tests catch it at npm test time | Pre-commit validation |
| `.pi/permissions.json` becomes malformed | matcher.ts falls back to in-memory defaults, notifies via ctx.ui | Existing fallback, not regressed |
| Existing user runs pi without reloading file | New keys absent from active session until restart | Documented limitation; not new |
| Worker dispatch mid-implementation wipes new files (recurrence) | Same as the original incident | This very plan prevents it |

## Unresolved decisions

None. All review decisions answered (strict blocking chosen, git-rm drift fixed, git-checkout-branch removed, 1 combined commit).
