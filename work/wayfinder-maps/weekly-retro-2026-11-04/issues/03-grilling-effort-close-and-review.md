# Decide how an effort closes and what review owns

Type: grilling (HITL)
Status: open
Blocked by: 01

## Question

What must happen before an effort's branch merges into `main`? Which defect classes stop being review work and become a check? Where do the standing rules of a build ticket live?

## Context

- **Candidates** of [the report](../assets/README.md):
  - the defects that span tickets and reach `main`, and the close-out that the owner types by hand (R2)
  - tests that cannot fail come back in every effort (R3)
  - standing rules are copied from map to map and go stale (R14)
  - grilling rounds are too large to answer well (R15)
  - unused skills, and skill families that overlap (R22)
  - the hex-comparison lesson is not in the wayfinder skill (B5)
- **The agenda** is the section for this ticket in `../assets/dispositions.md`, from [Propose a disposition for each candidate](01-research-propose-dispositions.md). Ask its questions, at most 3 in each round (map, Notes).
- **Facts the owner may need.**
  - On prompt-flexibility, a review of the whole branch ran after the ticket reviews. Its findings needed an 8-phase fix plan (`old-docs/archived-plans/prompt-flexibility-review-fixes.md`).
  - Layout-trials merged on 2026-11-03 with per-ticket reviews only (inferred: no record of a whole-branch review exists).
  - The owner corrected a fast-forward merge on 2026-10-30. The next 3 merges were fast-forwards, and the owner did not object to them.
- **Not decided here.** Whether a mutation run enters the gate (R3) belongs to the open grilling tickets of [Unclebob quality tools](../../unclebob-quality-tools/map.md) on the value and the shared-tree safety of mutation testing. This ticket decides only the route of the R3 evidence, and whether anything is needed until those tickets close.

## Answer

Record each decision. For each adopted fix of one session or less, create a task ticket in this map. Route each larger fix, and record where it went.
