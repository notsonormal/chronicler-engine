# Put formatting into the narrow dev loop

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`clippy` and `test-pattern` do not check formatting. In this run, trailing whitespace from heredoc writes showed up only in a 60 s `--no-browser` gate, which then had to run again. Should the narrow loop catch it?

- **Name `python build.py fmt` in the Development Loop table** as the step before the gate.
- **Make `clippy` run the fmt check** (`cargo fmt --check`) first.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Implementer `63ee340e` entries 404, 405 (`left behind trailing whitespace --> tests/http/settings.rs:568`), 408.
- Related: [Run every non-browser binary as the focused check](53-focused-checks-miss-a-binary.md).
