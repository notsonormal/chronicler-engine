# The gate-model hash changes when only a comment changes

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`verify_coverage.py` hashes the raw source of each listed function, so a comment edit raises the verify-gate warning. Must the hash leave out comments?

## Context

Source: `/retro` 2026-11-22 of [Build the scene entries](../../long-term-memory-and-state/issues/17-build-scene-entries.md), retro session `e91f4262`.

- **known** `scripts/verify_coverage.py:210` hashes the raw text. Comments are blanked only to count braces (docstring :25-27).
- **known** A comment-only edit in `Job::options_step` made the coordinator run `check.py` again (about 40 s), undo a line wrap and record new hashes.
- Related to [121](121-verify-gate-warning-asks-each-agent.md), which is a different fix.

## Options to decide

- **A.** Hash the text with comments blanked, with a test in the script's Python tests.
