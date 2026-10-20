# Make a red gate name the failing test and its message

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

When a nextest step fails, the gate prints the step, the counts and the log path, but not the failing test. In this run `fix-code` needed two more calls to find `guardrails_long_comment_runs` and its message (`gate.rs:338 [WARN] Long comment run`). nextest already prints the test id. Should the epilogue print it?

- **Print each `FAIL` line and its first stderr line** in the epilogue.
- **Print a `grep` command** that finds them in the log.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Implementer `eaa0aa4a` entries 213 to 218; `logs/build_20261019_215814.log`.
