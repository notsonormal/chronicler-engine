# Never tell a review axis to skip `tooling.patch`

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A review brief told both axes that `tooling.patch` "contains only `.scratch/` ticket-status bookkeeping — IGNORE it". The Standards reviewer pushed back: "Premise mismatch: `tooling.patch` is not only `.scratch/` bookkeeping — it also carries that pin and the generated `AGENTS.md` structure line."

`prepare_review_bundle.py` classifies patch areas by path prefix; `tooling` is the catch-all: `build.py`, `scripts/`, `.agents/`, `.scratch/`, `AGENTS.md`. A skip instruction hides gate-pin edits and regenerated indexes. `--ref` mode also adds `commits.txt`.

Should a brief ever tell a reviewer to ignore a patch area?

- **No.** Drop the skip instruction; the bundle README states what each area holds.
- **Describe, then let the reviewer judge.** The brief lists the area's expected content and flags it as low-priority but never "ignore".
- **Both.**

## Context

- Source: `/reflect` 2026-10-07, session `01a11377` (both axes 2026-10-06 23:10/23:12).
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 14).
- `scripts/prepare_review_bundle.py` → `classify_area`; the `tooling` row is the empty-prefix catch-all.
- `.agents/skills/code-review/SKILL.md` §4 prompt templates.
- Related: [Give the Standards review the clone report](18-standards-review-clone-report.md).
