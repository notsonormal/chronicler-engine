# Trim the build commands and make the Development Loop section information only

Type: task
Status: resolved
Blocked by: —

## Scope

Implements the answer of [39 — Rethink the Development Loop section of AGENTS.md](39-rethink-the-development-loop-section.md). Read that answer first. It holds every decision and its reason.

- **`build.py`.**
  - Delete the `unit` subcommand.
  - Make `guardrails` run both test binaries (`--test architecture --test guardrails`). Delete the `architecture` subcommand. In `GATE_ORDER`, the two steps become one `guardrails` step. In `_TIERS` and `_tier_for_cmd`, `architecture` goes away, and the `guardrails` tier maps to the new command.
  - Update the comments and `--help` text that name `unit` or `architecture` (for example the `_INTEGRATION_FILTER` comment and the `GATE_ORDER` comment).
  - Keep `fmt`, `--no-fmt` and `check`.
- **Tests.** `scripts/tests/test_build_cli*.py`: the subcommand list, the command map, and the tier tests.
- **`AGENTS.md`, `## Development Loop`.** Information only. One table of the remaining commands (`clippy`, `test-pattern`, `integration`, `guardrails`, `browser`, `--no-browser`, bare full gate, `validate-docs`, `run`), with what each one covers and its warm cost from `logs/build_history.txt`. Keep: the `tmp/` fact, the 1200 s timeout, the `tail` command, and the `ENVIRONMENT.md` pointer (moved in from `## Concurrent Builds`). Remove everything else that ticket 39 lists, and add no definition of "done".
- **`ENVIRONMENT.md`.** Take in the dev-server DB and data paragraph. Update the cost table columns: `architecture` and `guardrails` become one `guardrails` column.
- **Sweep.** `rg` for `build.py unit` and `build.py architecture` outside `old-docs/` and `.scratch/`. Fix each live reference.

## Context

- Before the change: `AGENTS.md:348–394` (the section) and `:396–403` (`## Concurrent Builds`).
- The journal `tests` column uses `key=value` pairs, so old rows with `architecture=1` stay valid.

## Done when

- `python build.py --help` lists neither `unit` nor `architecture`.
- One full gate is green, and its journal row shows no `architecture` key.
- `/code-review` has run, and each finding is fixed or declined with a reason.

## Answer

Done 2026-10-11. Uncommitted; plan in `docs/plans/48-trim-build-commands-and-development-loop.md`.

- **`build.py`.** `unit` and `architecture` subcommands deleted. `guardrails` runs `--test architecture --test guardrails`. `GATE_ORDER` has one `guardrails` step; `_TIERS` is `guardrails, integration, browser`. Comments updated. The `--no-browser` help string and the module docstring lost their "iteration default", "final run" and "before reporting a task done" wording (ticket 39: no orders, no "done" definition; `AGENTS.md` now points at `--help`).
- **Tests.** `scripts/tests/test_build_cli*.py`: command map, gate-plan lengths (19→18, 18→17), tier map, journal tests. One test pins that an old row's `architecture=1` key is skipped.
- **`AGENTS.md`.** `## Development Loop` is one command table (covers, warm cost from `logs/build_history.txt` medians), the `tmp/` fact, the 1200 s timeout, the `tail` command, the `--help` and `ENVIRONMENT.md` pointers. `## Concurrent Builds` removed.
- **`ENVIRONMENT.md`.** Takes the dev-server DB and data paragraph. The cost table has one `guardrails` column (architecture + guardrails).
- **Sweep.** `chronicler-ui-investigator` and `chronicler-after-plan-workflow-plus-review` skills now point at `ENVIRONMENT.md`.
- **Gate.** Full gate green on the final tree: `2026-10-11T21:19:55 | 144.6 | 0 | (full gate) | 9b78b67196+dec2853c | guardrails=163 integration=1627 browser=29`. The first gate after the change showed guardrails "+1" (162→163), as ticket 39 predicted.
- **Review.** `/code-review` both axes. Fixed: wrong ENVIRONMENT.md footnote, cost-figure conditions (a line under the table), incomplete full-gate "Covers" cell, duplicated absence assertions (the `EXPECTED_COMMANDS` pin already covers them), history wording in a test comment, comment wrap, the `--help`/docstring order wording.
- **Left open (2 stale pointers).**
  - `.cargo/config.toml:4–5` cites `AGENTS.md` "Concurrent Builds". Not edited: `scripts/target_seed.py` hashes that file into the seed build signature, so a comment edit makes every seed miss once. Fix it the next time that file changes for another reason.
  - `.pi/extensions/pi-permission-system/config.json:45` reason string says "see AGENTS.md, Development Loop". The permission system denies agent edits to it. The user changes it to "see ENVIRONMENT.md".
