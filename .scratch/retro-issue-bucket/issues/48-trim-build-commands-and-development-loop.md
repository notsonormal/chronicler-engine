# Trim the build commands and make the Development Loop section information only

Type: task
Status: open
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
