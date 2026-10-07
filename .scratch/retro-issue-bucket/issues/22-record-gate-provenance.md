# Record which gate run covers the committed tree

Type: grilling (HITL)
Status: open
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
