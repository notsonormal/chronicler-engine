# Options and actions spec cleanup

## Summary
The doc-spec boundary follow-ups (`docs/plans/doc-spec-boundary-follow-ups.md`) added Scenarios 24.15–24.20 to `docs/specs/options.md` and 2.5 to `docs/specs/actions.md`. Their review left three owner-approved follow-ups:

1. Scenarios 24.15 and 24.16 now cover 24.7 and 24.12, so the old two go.
2. The edit to `actions.md` triggers the `tests/STRATEGY.md` §Spec vocabulary rule: "When a spec file is edited, reword its scenarios that name a Rust identifier, a field path, or a module." Most `actions.md` scenarios name `message_service.…` paths.
3. The specs say "message", not "turn". `CONTEXT.md` deprecates **Turn** ("Use Message + Swipe"), but its own **Options** entry and the `options.md` titles still say "turn".

This plan changes specs, tests and `CONTEXT.md` only. It changes no code in `src/`.

## Implementation

### Phase A: Remove the covered Options scenarios

- [x] #### Task A.1: Delete Scenarios 24.7 and 24.12
  - 24.7 ("The always-on toggle never fires after an impersonate turn") starts from an empty set and asserts an empty set. 24.15 starts from set A with set B queued, so it fails if always-on fires or the set is cleared.
  - 24.12 ("A narration turn with the toggle off generates no options") asserts an empty set from an empty set. 24.16 starts from set A, so it fails if the set is kept or regenerated.
  - Delete both scenarios from `docs/specs/options.md` and their tagged tests in `tests/http/options.rs` (the tests tagged `SCENARIO: 24.7` and `SCENARIO: 24.12`). Do not renumber the other scenarios. Delete any helper that only those tests used.
  - Before you delete a test, read it and confirm that 24.15 or 24.16 asserts everything it asserts. If it asserts something more (for example a dock check that the new test lacks), move that assertion into the covering test first.

### Phase B: Reword `docs/specs/actions.md`

- [x] #### Task B.1: Reword every scenario to the spec vocabulary
  - Rule (`tests/STRATEGY.md` §Spec vocabulary): a Given or Then names a domain thing with a `CONTEXT.md` term, and a client-seen thing as a response value or a DOM `id` or `class`. Plain English is fine when its nouns are glossary terms.
  - Replace field paths and identifiers with domain terms. Examples:
    - `message_service.load_or_fresh().narrative.input_buffer.status is Idle` → `the generation status is Idle`
    - `GenerationStatus::Error(msg) where msg contains "empty"` → `the generation status is an error whose text contains "empty"`
    - `message_service.load_messages() contains exactly 2 new entries` → `the message history gained exactly 2 messages`
    - `event_header is Some(...)` → `a Narration carries an event header` (the trigger fired)
    - `scene.npcs_in_area contains "gabriella"` → `the scene's NPCs include "gabriella"`
    - `npc_encounter_log.npcs["gabriella"].times_met is 1` → `the player has met "gabriella" once`
    - `trigger fires when times_met == 0` → `whose Trigger fires on the first meeting`
    - `movement.current_room_id == "non_existent_room"` → `the player stands in a room that does not exist`
  - Mock-backend answers in a Given (for example `{"npcs_in_room": ["npc_1"]}`) are what a backend returns, not a Rust identifier. Keep them.
  - The Properties section (I.1 and later) and the engine-command paragraph (`Action::is_engine_command`) follow the same rule.
  - Keep every scenario's number, meaning and observation surface. A reword must not change what a test has to assert, so no test changes in this phase. If a scenario cannot be reworded without changing its meaning, stop and report it.

### Phase C: "Message", not "turn"

- [x] #### Task C.1: Replace "turn" in the Options spec and glossary
  - `docs/specs/options.md`: every scenario title and step that says "turn" says "message" instead (for example "after a narration turn" → "after a narration message", "an always-on narration turn" → "an always-on narration message").
  - `CONTEXT.md` **Options** entry: "after a non-impersonate narration turn" → "after a non-impersonate narration message".
  - Leave test names and test comments alone. The rule is about the specs and the glossary.
  - Do not change other specs in this plan. `rg -n -i '\bturns?\b' docs/specs` lists them for a later pass.

### Phase D: Close

- [x] #### Task D.1: Validate
  - `python scripts/validate_feature_spec.py` reports 0 gaps and 0 orphans.
  - `python scripts/validate_docs.py --strict` passes.
  - `python build.py` (the full gate, browser tier included) passes.

## Test Plan
- Phase A: `python build.py test-pattern options`.
- Phases B and C change only Markdown. `validate_feature_spec` and `validate_docs` cover them.
- Phase D: the full gate.

## NOT in scope
- Code changes in `src/`.
- Rewording other spec files. The rule applies when a file is edited.
- "Turn" in specs other than `options.md`.

## Failure modes
- A deleted test asserted something that its covering test does not: Task A.1 moves the assertion first.
- A reword changes a scenario's meaning: Task B.1 stops and reports.

## Unresolved decisions
- None.
