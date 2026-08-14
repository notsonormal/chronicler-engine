# Enable the test-file-pairing guardrail (fix F1)

Type: task
Status: closed
Assignee: assistant
Blocked by: 02, 03, 04, 07

## Resolution

Fixed `guardrails_test_file_location` in `tests/infrastructure/guardrails/mod.rs` to pass the full `src/...` path to `check_test_file_location` (mirroring the existing `guardrails_doc_standards_tests` pattern) instead of the relative path stripped by `check_src_files`. This removes the F1 mismatch that caused the rule to return early on every `src/` file. Verified `cargo nextest run --test guardrails` passes and `python build.py` is green with zero orphan test files.

## Question

The guardrail `check_test_file_pairing` is currently a no-op in the production walker (F1): `check_src_files` strips the `src/` prefix before calling the rule, but the rule returns early unless the path contains `src/`. So every `*_tests.rs` under `src/` is skipped in CI.

Fix the path mismatch so the rule is actually invoked, then confirm the guardrail passes with **zero** orphan test files. This lands **last** — after every orphan is resolved — so CI never goes red:

- [Move parser_tests.rs into utils/](issues/02-move-parser-tests-to-utils.md)
- [Move context_tests.rs into utils/](issues/03-move-context-tests-to-utils.md)
- [Consolidate game_state_*_tests.rs](issues/04-consolidate-game-state-tests.md)
- [Execute pipeline split](issues/07-execute-pipeline-split.md) (makes `retry_tests.rs` comply)

Implementation: either pass a path containing `src/` to the rule from `check_src_files`, or remove the `src/` prefix guard inside `check_test_file_pairing` (the walker already only iterates `src/`). The unit tests in `tests/infrastructure/guardrails/location_tests.rs` already pass `src/`-containing paths; ensure they still pass after the fix. Run `cargo nextest run --test guardrails` and `python build.py`.
