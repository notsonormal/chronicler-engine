# Make the test standards docs match the code

Type: task (AFK)
Status: open
Blocked by: —

## Question

Which claims in the test standards docs are false today, and which parts only copy what the code already states?

## Context

- Scope: `docs/diataxis/reference/coding_standards/{testing,integration_test_standards,unit_test_standards}.md` and the non-generated parts of `tests/AGENTS.md`.
- Drift confirmed in the audit session [known]:
  - `SqliteTestAppBuilder` does not exist, yet integration Pattern 1 and Cross-cutting 6 are built on it. The guardrail message at `tests/infrastructure/guardrails/structure.rs:214` tells people to use it. Find the builder that tests really use, and fix both places.
  - `tests/helpers/sqlite_test_app_builder.rs`, `tests/infrastructure/invariant_contract.rs`, and `src/application/narrative_prompt/` do not exist. The last one is probably `src/application/prompting/`.
  - The integration Pattern 6 example of `Args` lacks `host` (`src/utils/cli.rs:35`).
  - `testing.md` says "nine-pattern", but `unit_test_standards.md` has eight patterns.
- Unverified claims from a scout report: the file counts ("~10 files" is really 15, "14 files" is really 27). Check them before you change them. [docs_drift.md](../assets/test-audit/docs_drift.md) has a known error: it treats `tests/storage/` as untiered, but `tests/STRATEGY.md:15` gives it a row.
- `integration_test_standards.md` is 352 lines. Most of it copies fixture signatures, helper names, and counts that the code states. Those copies are what drifted. Per the writing-for-agents skill ("Pruning"), keep what the code cannot say, such as why the port lock lives in `/tmp` and why mocks are injected over HTTP. Replace the rest with pointers.
- This ticket does not touch `tests/STRATEGY.md`. That is [Rewrite the test strategy around one checkable tier-1 rule](32-rewrite-test-strategy.md).

## Done when

- Every path, type, and function named in the scoped docs exists.
- Every count is correct or removed.
- The `structure.rs:214` message names a real builder.
- `python build.py` is green. The user reviews the diff. Commit after approval.
