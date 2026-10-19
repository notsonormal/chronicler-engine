# Run every non-browser binary as the focused check, not a hand-picked list

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

C4 moved the persona check of the boot behind the Game lookup. The C4 reduced gate and the C5 focused checks named a list of test binaries (`http`, `lib`, `storage`) without `bootstrap`, so two `run_branches::test_run_persona_not_found_*` failures stayed hidden until the C5 full gate. That cost one red full gate (~200 s) and a fix. From C6 on, briefs asked for `python build.py integration` (every non-browser binary, ~30 s), and no binary was missed.

Where should the rule live?

- **`AGENTS.md`, Development Loop.** One sentence: before you report work as done, run `python build.py integration`; a `--test-target` list misses tests of other binaries.
- **`.agents/skills/simple-after-plan/SKILL.md`.** The brief template asks for `integration`, not a list.
- **Leave it.** The full gate catches it.

## Context

- Source: `/retro` 2026-10-19 on the core step of the Rust rebuild (ticket 07, "C5 commit gate" note of 2026-10-18 ~23:05).
- Evidence: `logs/build_history.txt`, the C5 full gates of 2026-10-18 ~23:00; the fix in `src/bootstrap/run.rs` `prepare_data` calls `storage.require_persona` before `resolve_boot_game`.
