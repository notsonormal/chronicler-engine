# `trial.py compare` overwrites the reference trial's `compare.md`

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`trial.py compare <a> <b>` writes `compare.md` beside the first pack, and names each column by its folder name only. Where must it write, and how must it name its columns?

## Context

Source: `/retro` 2026-11-22 of [Build the scene entries](../../long-term-memory-and-state/issues/17-build-scene-entries.md), retro session `e91f4262`.

- **known** `scripts/trial.py:1499` sets the default path (help at :1542). The column name is the folder name (:1468), so two packs both read `trial-1`.
- **known** The trial of ticket 17 put the reference first, as the map Note says, and overwrote `tmp/ltm09/redmist_julian/compare.md` at 18:59.

## Options to decide

- **A.** Write beside the last pack, and label each column with the shortest path ending that is unique. Add a test in `scripts/tests/test_trial.py`.
- **B.** Refuse to overwrite a file that `--out` did not name, as well as the labels of A.
