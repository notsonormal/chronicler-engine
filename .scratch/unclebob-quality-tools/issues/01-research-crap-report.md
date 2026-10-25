# Research: what a CRAP report shows on this codebase

Type: research (AFK)
Status: open
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
