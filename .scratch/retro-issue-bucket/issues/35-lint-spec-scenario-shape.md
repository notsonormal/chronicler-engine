# Lint a spec scenario's shape and vocabulary

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

An implementer added six spec scenarios. Two broke `tests/STRATEGY.md` rules that only a review checks:

- **Spec vocabulary.** Scenario 2.5 in `docs/specs/actions.md` had the Then lines `message_service.load_or_fresh().narrative.input_buffer.status is Idle` and `...scene.npcs_in_area still contains "npc_1"`. Scenario 24.17 in `docs/specs/options.md` used the field name `options_always_on`. STRATEGY §Spec vocabulary forbids Rust identifiers and field paths.
- **One scenario, one behaviour.** Scenario 24.17 held two independent Given/When/Then blocks: a second `Given` came after a `Then`. Two tests carried the one tag.

The document-review subagent found both, and the lead fixed them. Both patterns are mechanical: a `.`-joined snake_case path or a `::` inside a gherkin block, and a `Given` line that follows a `Then` line in the same scenario.

Where should the check live?

- **`scripts/validate_feature_spec.py`.** It already parses every scenario. It could fail on a `Given` after a `Then` and on an identifier pattern. It runs in the gate as `spec-coverage`.
- **A ratchet.** `actions.md` already names `message_service.…` in most scenarios. The check could cover only new scenarios, or carry an allow-list that only shrinks.
- **Review only, as now.** The document-review brief already caught both problems.

## Context

- Source: `/retro` 2026-10-11, lead session for `docs/plans/doc-spec-boundary-follow-ups.md`. Implementer session `01a12ac6-5809`, document-review session `01a12adb-ddca`.
- `tests/STRATEGY.md` §Spec vocabulary, and its rule that an edit to a spec file triggers a rewording of the scenarios that name identifiers. That rule now makes all of `docs/specs/actions.md` due for a rewording.
- A related tag check: `validate_feature_spec.py` could also warn when two tests share a tag. Scenario 2.5 does this on purpose: one tag, two branches with one Then.
