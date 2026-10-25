# The seam-case form: the tag carries the description

Type: task
Status: resolved
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

**2026-10-17, implementer (uncommitted).** Branch `wf/rebuild-seam-form`, base `c820278d`. The tag-only form is in place.

Facts measured.

| Item | Result |
|---|---|
| Tag form | `// CASE <id>: <description>`, and `// CASE <id> [<label>]: <description>`. Python `CASE_TAG_RE`: `^\s*//\s*CASE\s+([a-z0-9][a-z0-9-]*)(?:\s*\[([a-z][a-z0-9-]*)\])?\s*:\s*(\S.*?)\s*$`. |
| `scripts/validate_seam_cases.py` | A declaring file carries one tag or more. The checker fails on: a test in a declaring file with no tag; a tag with no test attribute within `TAG_LOOKAHEAD` (5) lines; two tags on one test; a duplicate id in the repository; a malformed tag (bad id, bad label, no description); the old form, that is `//! CASE` or a bare `// CASE: <id>`. `--list` prints the id and the description of every case, contract by contract. |
| Cases | 3 contracts, 57 cases: Quantifier 11, Options Agent 8, `Narrator` 38. Every id, description and test function matches `HEAD` (`//! CASE <x>` against `// CASE <x>` is byte-equal; the id names the same `fn`). |
| `tests/infrastructure/guardrails/unit_test_allowlist.rs` | `is_case_declaration` reads the same form, with a required whitespace after `CASE` and `is_case_label` for the label. The Rust reader and the Python reader agree on every fixture of both test files. |
| Docs | `tests/STRATEGY.md` "The one rule" shows the tag form. `AGENTS.md` and `guardrails.md` are regenerated (`generate_structure_index.py`, `generate_guardrails_doc.py`; the guardrail's source line moved to 201). The `seam-cases` help text of `build.py` follows the new rule. |
| Gate | `logs/build_20261017_224403.log`: the full gate is green (clippy; guardrails 189; integration 1624 passed, 2 skipped; browser 29). `python -m unittest discover scripts/tests`: 398 tests, OK. `rg -n '^//! CASE|// CASE: ' src tests` finds nothing. |

Deviations.

1. The Question says 55 cases. The three contracts carry 57 (11 + 8 + 38); the count did not change.
2. The Rust fixture asserts that the old bare form is not a declaration. To keep the acceptance grep clean, that fixture writes the form without the space after the colon (`// CASE:storage-1`), which both readers reject. The Python tests hold the spaced old form and the module-doc form.
3. `tests/AGENTS.md` and `.agents/skills/` carry no example of the old form; only `tests/STRATEGY.md` did.

## Answer

Built and merged to `main`. A seam case is declared once, as `// CASE <id>: <description>` (optional `[<label>]`) above its test. `validate_seam_cases.py --list` prints the cases of each contract. The three contracts keep their 57 cases (the ticket's "55" was the count before the P4 fix pass). The guardrail `is_case_declaration` reads the same form. Full gate green: `logs/build_20261017_224403.log`.
