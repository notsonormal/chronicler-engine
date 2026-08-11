# Audit branch coverage across src unit tests

Type: research (AFK)
Status: open
Blocked by: 10

## Question

Graduates from [ticket 10](10-partition-codebase-audit.md).

STRATEGY.md requires unit tests to cover every branch (sync or async,
fakes at driven ports). The map's Not-yet-specified section already
flagged `ArrivalTaskContext::run` (5 uncovered branches, 3 of ~10
covered) and asked whether this is an isolated case or a pattern across
other application-layer classes (`GameViewQuery`, `MessageService`,
`GameCatalogue.reset()`).

This ticket answers that question by mapping branch coverage across
`src/` unit tests (100 files, 816 tests across domain / application /
HTTP-driving / storage-driven / LLM-driven / bootstrap+utils):

- Which classes have uncovered branches?
- Is the `ArrivalTaskContext` case isolated, or a pattern?
- For each gap: branch location, why it's uncovered (unreachable in
  current config? needs a fake seam? just missing test?).

## Output

A single markdown asset (linked from this ticket) with:

1. One row per class with uncovered branches (class, file, uncovered
   branch count, branch locations, reason)
2. Summary: pattern or isolated case
3. If a pattern: suggested cluster boundaries for future fix tickets
   (does NOT create those tickets — out of scope for this map)

This ticket **maps only** — no new tests, no code changes. Fixes
execute on a future effort (out of scope for this map).

## Out of scope

- Writing any test — investigation only on this map.
- Coverage-percentage targets (invariant-based approach preferred per
  map Out-of-scope).
- Coverage tooling changes — same.
