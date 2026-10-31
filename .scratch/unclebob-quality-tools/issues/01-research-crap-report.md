# Research: what a CRAP report shows on this codebase

Type: research (AFK)
Status: resolved
Blocked by:

## Question

What does a CRAP report (CRAP = CC² × (1 − coverage)³ + CC, per function) show on Chronicler Engine, on fresh coverage, and what would the user need to see to judge its value?

## Context

- The first run (see the [research asset](../assets/research-crapper-mutator-dryer.md), section *crapper*) used a two-day-old coverage JSON converted to LCOV by a scratch script, and found two distortions: `?` counted as a branch (`migrate_up_to` CC 200, every `from_row` CC 12–19), and files that look uninstrumented (0% on wiring code).
- `build.py --coverage` writes `coverage.json` through `cargo llvm-cov report --json` (`build.py` ~line 1693). `cargo llvm-cov report --lcov` from the same profile data should give LCOV with no test rerun (inferred; verify).
- crapper joins coverage to a function by file and line range. Its line coverage marks a `?` line covered when only the `Ok` path ran; LLVM branch or region data could tell the two paths apart (inferred; verify).
- crapper is installed in `tmp/unclebob/.venv`; run it on a copy of `src/` with `*_tests.rs` removed (`tmp/unclebob/ce/`), or add the exclusion.

## Work

1. Produce fresh coverage through `build.py --coverage` (it takes 5–8 min and the build slot) and get LCOV from it. Record the exact command.
2. Run the CRAP report with at least two CC rules: `?` counted, `?` not counted. If cheap, a third variant that uses region or branch coverage instead of line coverage.
3. Exclude `*_tests.rs` and `src/test_support/`.
4. For the top 20 of each variant, read the function and annotate: why it ranks there, whether the rank comes from real branching or from a counting artefact, and what a developer would do about it (a test, a split, nothing).

## Expected output

- An asset under `assets/` with the variants side by side and the annotated top 20: written so the user can judge value without reading the tool's source.
- The facts a value decision needs: run time, coverage cost, how stable the ranking is across variants, and how many of the top 20 point at a real risk (the agent's reading, labelled inferred).
- On resolution, graduate a grilling ticket: "Does the CRAP report have value here, and which CC rule?"

## Answer

The report runs in 4 s on fresh coverage, and one rule decides the ranking: whether `?` counts as a branch.

- **`?` is 25.7 % of all CC** (2,836 → 2,106 over 917 functions). 119 functions (13.0 %) have CC entirely from `?`.
- **Counting `?` fills the top 20 with column lists.** 7 of the 20 are `from_row` functions at CC 12–19 with 100 % line coverage; dropping `?` sends each to rank 400+.
- **The list is mostly a complexity list.** 85.5 % of functions have 100 % line coverage, so CRAP = CC for them (66.8 % under region coverage).
- **Fresh coverage removes both distortions of the 2026-10-25 run.** Wiring code now reads 50–100 %; the only 0 % rows are explained by call site, not staleness — `init_logging` is called only from `src/main.rs`.
- **Branch coverage is unavailable**: `cargo llvm-cov report --lcov` emits 0 BRDA records on rustc 1.88 / cargo-llvm-cov 0.9.1. Region counts from the JSON export are the substitute, and they move 1 row of the top 20.
- **Of the top 20 under the upstream rule, 6 point at a testable risk** (inferred reading). Under the `?`-free rule 5 do, and 6 are complexity-only rows.

Cost: coverage 487.6 s gate-reported (522 s wall, 6.1 GiB dedicated target dir); LCOV export 12 s; CRAP 4 s. Reusing an existing coverage file is 500× cheaper than running coverage on demand.

Stability: changing the CC rule keeps 6 of the top 20; changing line → region coverage keeps 19 of the top 20. The CC rule is the decision.

Asset: [01-crap-report.md](../assets/01-crap-report.md) — four variants side by side, the annotated top 20 of each, the traps, and the facts a value decision needs.

## Proposed grilling ticket (for the coordinator to create)

```markdown
# Grilling: does the CRAP report have value here, and which CC rule?

Type: grilling
Status: open
Blocked by:

## Question

Does a CRAP report earn a place in `build.py`, and if so with which CC rule — `?` counted (upstream) or `?` dropped?

## Evidence

[Research asset](../assets/01-crap-report.md).

- Upstream rule: 7 of the top 20 are `from_row` column lists; 9 of 20 are artefacts or unreachable code; 6 point at a coverage gap.
- `?`-free rule: no artefacts, 18 rows at CRAP ≥ 10, of which 6 are complexity-only rows at 100 % coverage.
- 85.5 % of functions sit at 100 % coverage, so CRAP = CC for them.

## What the decision must settle

1. Value: is a list that needs a seven-entry ignore list (`from_row`, `story_row`, `migrate_up_to`, `init_logging`, `start_server`, `source`, `provider_label`) worth 4 s per run?
2. Rule: count `?` with an ignore list, or drop `?` and get a complexity list?
3. Threshold: report-only, a CRAP bar, or nothing in the gate — and at which number (30 passes today with 2 rows; 10 fails today with 18–46)?
4. Coverage source: reuse the last `coverage.json` (12 s + 4 s) or run coverage on demand (8m42s, 6.1 GiB)?
5. Line or region coverage — 1 row of the top 20, at 12 s of export?
```
