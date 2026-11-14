# A step run of the tests stops at the first failure and does not say how to run the rest

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Must the "N not run" count of `build.py` say how to run the tests that did not run?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass), with the build session `a09ca82b`.

- **known** The agent wrote "478 tests did not run. I run it again", ran again without `--no-fail-fast`, and the second run stopped early too (43 s lost).
- **known** The fail-fast default is at `build.py:242-245`; the count is printed by `scripts/build_report.py:70`. `AGENTS.md` says the `integration` step covers "Every test binary".

## Options to decide

- **A.** Append "(add `--no-fail-fast` to run them)" to the count.
- **B.** Nothing.
