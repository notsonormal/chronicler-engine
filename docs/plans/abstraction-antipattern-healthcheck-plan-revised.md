# Plan: Advisory clippy smell detection in healthcheck

**Date:** 2026-08-12  
**Status:** Planned  
**Goal:** Surface `clippy::too_many_arguments` and `clippy::dead_code` findings as an advisory report in `scripts/healthcheck.py`, without blocking CI.

## Why
`scripts/healthcheck.py` currently only runs a duplicate-code check. `cargo clippy` already emits useful smell warnings, but they are buried in build noise. Two deliberate suppressions already exist in the codebase that should be reviewed periodically:

- `src/application/arrival_service.rs` — two `#[allow(clippy::too_many_arguments)]` suppressions.
- `src/adapters/driven/llm/providers/deepseek.rs` — `#[allow(dead_code)]` on a field stored for future implementation.

## Scope

1. Add a new `@register("clippy_smells")` check to `scripts/healthcheck.py`.
2. Run `cargo clippy --all-targets --message-format=short`.
3. Parse the short-format output (`file:line:col: level: message [lint]`).
4. Collect:
   - `clippy::too_many_arguments` — include all hits, even if `#[allow(...)]` is present.
   - `clippy::dead_code` — exclude hits that have an adjacent `#[allow(dead_code)]` or `#[allow(clippy::dead_code)]` attribute (scan previous 5 lines).
5. Exit code 0 when findings exist (advisory). Exit code 1 only if `cargo clippy` itself fails.
6. Add a `--out` flag to write the markdown report and `--verbose` for progress.
7. Update the module docstring `Usage:` block.

## Out of scope
- Blocking enforcement.
- `arch-lint.toml`, `tests/guardrails.rs`, or `src/lib.rs` lint changes.
- Any cleanup of the existing suppressions.

## Acceptance criteria

- `python scripts/healthcheck.py clippy_smells` runs without error.
- Output is markdown with sections for `too_many_arguments` and `dead_code`.
- Each finding has `file:line` + lint name.
- Exit code 0 even when findings exist.
- `python scripts/healthcheck.py all` includes `clippy_smells` automatically.
- Report lists the two known suppressions above as examples and does not flag allowed dead code.

## Verification

- `cargo clippy --all-targets -- -D warnings` clean (baseline).
- `python scripts/healthcheck.py clippy_smells --out report/smells.md` produces `report/smells.md`.
- `python scripts/healthcheck.py all` runs the new check.
