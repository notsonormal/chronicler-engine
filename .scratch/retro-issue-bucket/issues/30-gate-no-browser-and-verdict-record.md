# Add the `--no-browser` gate tier and a checkable gate verdict

Type: task
Status: resolved
Blocked by: —

## Scope

Implements the answers of [03 — Run the full build gate once per change set](03-run-the-full-gate-once.md) and [22 — Record which gate run covers the committed tree](22-record-gate-provenance.md).

- `python build.py --no-browser`: the full gate minus the browser tier. A step subcommand rejects the flag in both argument orders, and `--coverage --no-browser` exits 2. Bare `python build.py` stays the full gate.
- The journal `logs/build_history.txt` gains two columns: `tree` (short HEAD, plus a digest of uncommitted and untracked content when the tree is dirty) and `tests` (test-set size for each tier that ran).
- Each `nextest:` epilogue line for a known tier prints the count change against that tier's newest journal record, with that record's timestamp and tree.
- `AGENTS.md`: the Final Validation ladder and the `integration` comment. `tests/AGENTS.md:21`: "non-LLM suite". `ENVIRONMENT.md`: a measured `--no-browser` row.

## Context

- The journal shows 114 full-gate runs since 2026-09-25, 317.5 min, 63% of all build wall time. The browser tier is about two thirds of a warm gate.
- Session `01a11db3` (2026-10-08 22:47) got a green gate on its first run, then distrusted it: per-tier counts had changed since an earlier gate (guardrails 165 → 162, integration 1570 → 1563), and nothing recorded which tree each count came from.
- The integration step already runs the 1220 lib unit tests (`test-pattern` on a lib test name: `1 passed, 0 failed, 1805 skipped`).

## Answer

- **Measured.** Warm default target dir, no Rust change: `--no-browser` 34 s of step time (`logs/build_20261008_232127.log`); full gate 170 s, browser 135 s of it (`logs/build_20261008_231827.log`). Both green on tree `1d8f342250+4d7f6727`. Counts: architecture 1, guardrails 162, integration 1565, browser 69.
- **Review (Spec).** Faithful, no scope creep. Fixed: the `ENVIRONMENT.md` row names the target dir. Answered: the suffix-order rejection test exits through argparse, as the existing `--coverage` test does.
- **Review (Standards).** Fixed: a failed `git hash-object` (file removed by a concurrent agent, newline in a path) no longer makes the tree `unknown`; the journal read takes a shared lock; `_append_history` has no defaults; the heading is `## Answer`. Declined: the docstring/help repeat (different readers), the fixture duplication, the tier-map test (it pins the tier names).
