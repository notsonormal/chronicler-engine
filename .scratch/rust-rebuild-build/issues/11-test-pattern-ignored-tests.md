# Close or fix: `build.py test-pattern` cannot run ignored tests

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`python build.py test-pattern "<pattern>"` passes no `--run-ignored` to nextest. In P5 and C4 the target tests were `#[ignore]`d, so a reader had to call `cargo nextest run --run-ignored all` directly, outside the build slot.

The need has gone: on 2026-10-19 no `#[ignore]` is left in `src/` or `tests/http/`. The only ignored tests are in `tests/llm/`, and `python build.py --include-llm` and `--llm-only` run them.

- **Option A: close it.** No ignored test needs a pattern run today.
- **Option B: add a flag** `--run-ignored` to `test-pattern` (about 10 lines in `build.py` plus a test), for the next time a slice parks red target tests.

Recommendation (inferred): A. Reopen it with the next plan that parks red tests.

## Context

- Source: P5 ([P5: the storage Tx](06-p5-storage-tx.md), owner item 1 of its retro), repeated in the core step's retro ([The core step](07-core-step.md)).
