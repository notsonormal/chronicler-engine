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

## Further evidence (2026-10-11, tickets 15 and 16 of the dashboard UI review phase 2)

- Ticket 15 edited scenario 9.2 of `docs/specs/swipe_new.md`, so the "reword when edited" rule applied to the whole file. The document review listed 29 lines across about 20 scenarios and properties, for example `message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg)`. The lead reworded all of them.
- The standards review then found one more leftover (`never stuck \`Generating\`` in I.1). A lint would have found all of them in one pass.
- The spec-axis code review reported the rewording as scope creep (finding 6, session `01a12d31-7858-74b1-b842-ce6d120162ef`). The user ruled on 2026-10-11 that reviews must fix older problems in the files a change touches (see Scope the comment pass to the lines the diff adds (ticket 12, closed; see the map)), so that finding was wrong. The gap is in the code-review spec brief, not in this rule.
- `swipe_new.md` now has none of these identifiers, so a ratchet there would start at zero.
- Sessions: document review `01a12d26-c327-71c6-8485-983fc7949bdd`; standards review `01a12d31-6ae2-7628-9e61-cdb59383a0e5`.

## Comments

- 2026-10-25, audit: the user deferred this ticket. Do not implement it yet; it stays open.
