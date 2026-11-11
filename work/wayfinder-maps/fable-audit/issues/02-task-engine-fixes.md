# Engine fixes: one fault of the screen, five small defects, two dead fields

Type: task
Status: open
Blocked by: —

## Question

Put the changes below in place (owner, 2026-11-10, D7, D13 and D17 of [the decisions](../decisions.md)). Each has one obvious fix. They are in three groups, and one agent can take one group.

### Group A: the screen and the forms

1. **A part of the screen that fails renders its own error, and the other parts render (D7 a).**
   - Ran it: a Game whose Room is not on the map makes `GET /screen` answer 500 for all parts (`Failed to render visual-sidebar: Room not found: start`). The page is then blank, and the Games and Worlds tabs are gone too.
   - The site: `src/adapters/driving/http/screen/handlers/screen.rs`.
   - **One test pins the old behaviour.** `tests/http/failure_display.rs:217` (scenario 38.2 of `docs/specs/failure_display.md`) expects a non-2xx answer when the visual sidebar cannot render. Change the scenario and the test with the code.
2. **A form answers 400 for a value that it cannot read (D7 b).**
   - Ran it: `POST /settings/text-check` with `check_mode=spell-grammar` answers 200 and stores `Disabled` (`settings/handlers/settings.rs:190`, `_ => TextCheckMode::Disabled`).
   - Read it: the World create form and update form, and the presets handler, use `parse_or_default` (`worlds/handlers/worlds.rs`, six calls. `prompt_presets/handlers/prompt_presets.rs:278`).
   - The change covers the forms only. A read of a stored row keeps `parse_or_default`.
3. **An edit of the last Narration clears the offered Options (D17).** Ran it: the dock kept three Options after an edit. A delete of the last Message already clears them.
4. **The Autoplay status names the Impersonate step (D17).** Ran it: during an Impersonate the status reads "Generating narration..." (`layout/handlers/endpoints.rs:79`). The engine is writing the Input of the Persona.
5. **The story log scrolls to a new Narration when the window is full (D17).**
   - Ran it in Chrome: after 50 Messages the entry count stays at 50, so `assets/dashboard-story-log.js:239` (`storyLogEntryCount(log) <= start.entries`) never lets the scroll run.
   - The rule to use: follow when the log was at its bottom and the last entry changed (its id or its Swipe). This also covers a Retry.
   - The failing test first: one browser test with a full window. Scenario 30.20 of the browser spec describes the wanted behaviour.

### Group B: the process and the prompt

6. **Ctrl-C stops an Autoplay run (D17).**
   - Ran it: the port closes, the process stays, and the run calls the model until its Budget ends. A second Ctrl-C does nothing.
   - Read it: a Job and an Autoplay run are `spawn_blocking` tasks. `start_server` returns and the drop of the runtime waits for them (`bootstrap/run.rs`).
   - The change: after the server ends, stop every held Game through the gate (the path of `GameWriter::stop`), then end the runtime with `shutdown_timeout`. Handle SIGTERM in the same way.
   - This touches the gate. Run `/verify-gate` if a function of `.agents/skills/verify-gate/coverage.toml` changes.
7. **Story text is escaped where the history enters a prompt (D7 c).**
   - Ran it: a typed Input and a Narration with `</LastMessage>` and `<GameState>` in them were sent verbatim in the next narrator request and in the Quantifier request.
   - Read it: `sanitize_for_prompt` (`prompting/builders/sections.rs:8`) runs on the new user message only.
   - The change: escape the angle brackets of stored story text in one place in the assembler. The stored text and the story log do not change.
   - Check the cached share: the escape must give the same bytes for the same entry in every call, or the prompt cache loses its prefix.

### Group C: removals and the four fixes of the invariant audit

8. **Delete `maker` of a Swipe and `currently_meeting` of an Encounter (D7 d).** Read it: production code writes both and reads neither (`message.rs:36`, `trigger.rs:46`). The delete needs a migration for the Swipe column.
9. **Delete `items` of a Room, and `inventory` of a Character and of a Persona (D13).** Read it: no code reads them, and every shipped value is an empty list (`map.rs:44`, `character.rs:56`, `:65`). Remove them from the Rust types, the storage, the JSON schemas and the data files. `exits` of a Room stays.
10. **Four fixes that the invariant audit holds (D7 e).** The owner let them go ahead outside the ticket order of [that map](../../invariant-audit/map.md). When one is done, write a line in the ticket there.

    | Label | The fix | Its ticket there |
    |---|---|---|
    | world-04 | `evaluate_triggers` (`game_rules.rs:166`) reads the Characters in the order of `CharacterCard::in_id_order`, not in the order of a `HashMap` | Part of the grilling ticket 01. D7 e decides this one point |
    | llm-07 | One helper cuts provider text by characters, not by bytes (`response.rs`, `quantifier/utils/orchestration.rs`) | Ticket 06 |
    | llm-08 | `wiring.rs:152` returns the error of `AgentRunner::from_configs`. It does not replace it with `unwrap_or_default()` | Part of ticket 14 |
    | llm-04 | `GameWriter::edit` refuses an empty text | Ticket 09. Its blocker, ticket 03, is decided for this one point by D7 e |

## Rules for every change

- Write the failing test first. Then make the change.
- Line numbers are from commit `e8d6512a`. Find the site again by its name before you edit.
- One change that turns out not to be local stops. Say so in `## Answer`, and do the other changes.
- Use `python build.py clippy` and `python build.py test-pattern` during the work. Run `python build.py` once at the end.
- Do not commit.

## Not in scope

- The refusal of a World with no Room or no Scenario (world-02). It stays with ticket 02 of the invariant audit. Change 1 above only keeps the screen alive.
- A boot check that reports a stored row with an unreadable value.
- A `<think>` block in model text. The report infers the gap. No model of that kind was run, and no question asked it.
- The other Trigger faults (D10 leaves them open).
- A message after a hard stop, and a mark for a background Game (D8 b, c).

## What already exists

- [The story report](../assets/story-damage-and-false-screen.md): findings 1, 6 and 10, and section 7 (what the tests pin).
- [The engine report](../assets/engine-defects-and-risks.md): findings 2 and 13.
- [The lenses report](../assets/code-lenses.md): the values that become a default, and the two fields with no reader.
- The mutation run in [the environment report](../assets/development-environment.md), section 5: no test fails when the write of `currently_meeting` is inverted.
