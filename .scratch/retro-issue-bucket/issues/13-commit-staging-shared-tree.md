# Stop `commit-and-push` from staging the whole shared tree

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`commit-and-push/SKILL.md:75` is `git add -A`. This checkout runs several sessions at once, so `-A` sweeps in another session's in-flight work. It happened: a 2026-10-04 commit report says "A concurrent session's work is in commit 3. The `.scratch/architecture-deepening/` edits (18 min old, another session was doing t[hem])". A later session hand-excluded `.scratch/agent-workflow-friction/issues/03-yes-no-fact-placement.md` because another session had claimed it.

The 2026-10-04 reflect accepted "stage explicit paths (never `-A`)" and it never landed.

How should staging work on a shared checkout?

- **Stage explicit paths.** Name every path; never `-A`. Name any file deliberately left out.
- **Split by workstream.** Check `.scratch` claim state, separate mixed workstreams into separate commits.
- **Both, written into Step 3** (and the Pre-commit Hook Behavior section).

## Context

- Source: `/reflect` 2026-10-07, sessions `01a108fe`, `01a10d53`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 5).
- `.agents/skills/commit-and-push/SKILL.md:75` — `git add -A`.
- Prior art: `tmp/reflect/final_01a1089d-5ec4.md` (2026-10-04 Accepted, unlanded) — "Stage explicit paths (never `-A`) and prescribe the shared-tree hook-collision path."
- Related: [Add a GitHub-side push-failure path](19-github-side-push-failure.md) and [Record which gate run covers the committed tree](22-record-gate-provenance.md).
