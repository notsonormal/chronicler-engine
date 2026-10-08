# Record which gate run covers the committed tree

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

`commit-and-push` forbids building ("Do not try to build or tests to validate the that application is working - that's not part of this workflow", `SKILL.md:18`), so the commit report carries a self-reported green with no provenance. Commit `6077371b` landed 33 files including new source, reported "I did not build or test, per the skill workflow". Nothing ties a committed tree to the gate run that covered it.

Should the report record provenance?

- **Record the last green.** The commit report names which gate ran, against which tree, and when.
- **A machine stamp.** `build.py` writes a stamp (tree hash + gate result + time) that the commit report cites.
- **Both.** The stamp is the durable form; the report line is the human-readable one.

## Context

- Source: `/reflect` 2026-10-07, sessions `01a117f6`, `01a112d1`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 15; Backlog "Gate-provenance record").
- `.agents/skills/commit-and-push/SKILL.md:18` — the no-build rule.
- Prior art: `tmp/reflect/final_01a10894-87dd.md` (2026-10-04 Accepted, unlanded) — "record where the last green came from in the report".
- Related: [Stop `commit-and-push` from staging the whole shared tree](13-commit-staging-shared-tree.md).

## Answer

Decided 2026-10-08 with the user: **a machine stamp**. Implemented in [30 — Add the `--no-browser` gate tier and a checkable gate verdict](30-gate-no-browser-and-verdict-record.md).

- **Journal columns.** Every `logs/build_history.txt` record now carries `tree` (short HEAD, plus `+<8-hex digest>` of the uncommitted and untracked content when the tree is dirty) and `tests` (test-set size per tier, e.g. `architecture=1 guardrails=162 integration=1565 browser=69`).
- **Epilogue count change.** Each tier's `nextest:` line prints its count against that tier's newest journal record, e.g. `tests: 1565 (same as 2026-10-08T23:18:17, tree 1d8f342250+4d7f6727)`.
- **Trigger.** Session `01a11db3` got a green gate on its first run, then spent its session hunting deleted tests: counts had changed since an earlier gate (guardrails 165 → 162, integration 1570 → 1563) and nothing recorded which tree each count came from.
- **Known limitation.** In a shared checkout the baseline can be another agent's run on another tree. The suffix shows that the trees differ, but not which tests changed. Test names were declined as too much code for the gain.
- **Out of scope.** The commit report citing the gate run. `commit-and-push` still forbids building; a later ticket can make the report quote the journal line.
