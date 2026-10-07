# Put coordinator conventions in the map Notes before fan-out

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Parallel implementation tickets each re-authored the coordinator's conventions, so the briefs contradicted each other and each other's state. Two tickets both lowered `REQUIRES_MIGRATION_TEST_COUNT` ([Take the quarantine count out of parallel-ticket conflicts](02-quarantine-pin-parallel-conflicts.md)). The t68 session ran the full gate five times and t70 twice ([Run the full build gate once per change set](03-run-the-full-gate-once.md)). The briefs said "Do NOT commit, do NOT stage", but the settled policy became "You can commit onto worktrees, they will be deleted afterwards. But the changes should sit uncommited in dashboard-ui-issues-2 after everything is done."

Which conventions belong once in the map Notes?

- **Record all three before fan-out.** Shared counters/files only one ticket may touch; the gate cadence; whether worktree commits are allowed.
- **Only the collision-prone ones.** Shared counters and gate cadence; leave commit policy to the brief.
- **Neither.** Keep per-brief authoring; rely on the coordinator to reconcile.

## Context

- Source: `/reflect` 2026-10-07, sessions `01a11775`, `01a1177a`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 9).
- `.agents/skills/wayfinder/SKILL.md:13` — an effort may override the plan-only default in its **Notes**.
- `:36` — the `## Notes` block.
- Sibling tickets: [02](02-quarantine-pin-parallel-conflicts.md), [03](03-run-the-full-gate-once.md).
