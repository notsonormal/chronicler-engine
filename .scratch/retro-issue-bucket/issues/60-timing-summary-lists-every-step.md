# Give the duplicate step a row in the Step Timing Summary

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The full gate runs `Checking for duplicate code...` (step 14 of 19), and its summary goes to the build log. The Step Timing Summary has no row for it, so it looks as if the step did not run. In this run the lead read the summary and the start of a cut-off grep, then reported that "the step prints no summary". The report was false, and the lead corrected it later. Should every gate step have a timing row?

- **Yes**: report-only steps (`duplicates`, the worktree removal, the asset copy) get a row with `[OK]` or `[WARN]`.
- **Print one line** after the step: "Duplicate summary: N clone pairs, see the log".

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- `logs/build_20261019_213433.log`: `grep -c "Duplicate Summary"` = 1; the Step Timing Summary lists 14 rows and no duplicate row.
- `build.py` `_run_duplicates_report`, the Step Timing Summary print.
