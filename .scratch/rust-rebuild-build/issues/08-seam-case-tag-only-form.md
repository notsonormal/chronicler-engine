# The seam-case form: the tag carries the description

Type: task
Status: open
Blocked by: 05

## Question

Owner ruling, 2026-10-17 (option B): a seam case is declared once, on the comment line above its test, as `// CASE <id>: <description>` (with the optional `[<label>]` before the colon). The module-doc `//! CASE` list goes. This amends [ticket 26](../../turn-state-model/issues/26-decide-test-placement.md) R4 (owner Q11 = B, which put the list in the module doc and a bare `// CASE: <id>` on each test).

Reason: the checker allows exactly one test per case, so the module-doc list restated the test names and the id appeared twice. Cost accepted: a case can no longer be declared before its test exists.

Change, as one commit on its own branch after P4 merges:

- `scripts/validate_seam_cases.py` and its tests: a declaring file is a file with a `// CASE <id>: …` tag; it fails on a test with no tag in a declaring file, a tag with no test under it, a duplicate id, and a malformed tag. It prints the case list of each contract on request.
- `tests/infrastructure/guardrails/unit_test_allowlist.rs`: `is_case_declaration` reads the new form (it must match the Python reader).
- The three seam contracts (Quantifier, Options Agent, `Narrator`; 55 cases): move each description onto its tag, delete the module-doc list.
- `tests/STRATEGY.md` "The one rule" and `guardrails.md` prose where they show the form.

## Done when

The full gate is green, `validate_seam_cases.py` reports the three contracts with the same 55 cases, and no `//! CASE` line is left in `src/` or `tests/`.

## Comments
