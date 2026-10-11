# Research: map the test coverage of generation states and retry forms

Type: research
Status: claimed
Blocked by: —

## Question

Which tests cover each generation state, each Snapshot pointer, each retry form and each game-state writer? Which tests assert the restored game state, and which assert only the display? Which test doubles hide a defect?

## Scope

1. Integration tests (`tests/`, read `tests/AGENTS.md` first) and unit tests (`*_tests.rs` beside `src/application/pipeline/`, `message_service`, `generation/`) that touch: a normal generation, a Trigger continuation, movement, each retry form, Retrigger, swipe switch, Options rewrite, arrival.
2. For each test: the spec scenario it covers (`validate_feature_spec.py` annotations), and whether it asserts the restored state (room, NPCs, Snapshot id) or only the HTML.
3. Test doubles: the mock LLM provider and the mock Quantifier (`src/test_support/quantifier.rs`, `adapters/driven/llm/providers/mock.rs`). Which outputs are fixed (for example "no movement"), and which seed defects they hide. Seed §2 names scenario 22.1.
4. Harness gaps: arrival narration never runs in `src/test_support/`; list any other path that no test reaches.
5. The tests named in the seed as "would go red": do they exist now?

## Context

- Seed: [three pipeline defects with one root](../assets/seed-evidence.md).
- The repo rule: tests are the source of truth for behaviour (`AGENTS.md`, The Test-First Philosophy).

## Output

An asset `../assets/research-10-test-coverage.md`: a coverage matrix (rows: states, pointers, retry forms, writers; columns: test, kind, asserts state or display), the list of masking test doubles, and the list of gaps. Label each fact known, inferred, or tested. Cite test names, not line numbers.

## Feeds

The map's fog patch "Fix or rewrite", and the plan that the map ends in.
