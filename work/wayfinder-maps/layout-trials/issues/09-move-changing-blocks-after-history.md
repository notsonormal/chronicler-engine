# Move the changing blocks after the history

Type: task (AFK)
Status: resolved
Blocked by: 03

## Question

Make the kept candidate of [Try the changing blocks after the history](03-changing-blocks-after-history.md) a part of the engine, on the branch `layout-trials`, test first.

## Context

- The trial code is in `tmp/lt03/candidate.diff` (2 files, no tests). Use it as a guide, not as the change.
- The new order of the user message: `<KnownCharacters>` (with no presence marks), `<Persona>`, `<WorldLore>`, `<ConversationHistory>`, `<GameState>`, `<CharactersPresent>`, `post_history_prompt` (`<writing_style>`, `<output_format>`), then the tail (`<PlayerInput>`, `<Guide>`).
- Split `render_character_cards_layer` in `src/application/prompting/assembler.rs` into a KnownCharacters layer and a CharactersPresent layer. Remove the `(in room)` / `(elsewhere)` marks in `render_known_character_entry` (`src/application/prompting/builders/sections.rs`).
- Keep the two moved layers in the fixed list (`fixed_user_layers`), not in the tail. Then the history space counts them.
- The change applies to every Narrator-role call (Narration, Event Narration, the regenerate of a user Input, Impersonate). The Quantifier and Options calls do not change.
- Update the tests that pin the old order or the marks.
- Change the docs that the change makes wrong (`docs/AGENTS.md`, "Direction of edits"): at least caching rule 3 and the order in `docs/diataxis/explanation/prompt_system_design.md`, and the layer table in `docs/diataxis/reference/narrative/prompt_system.md`.
- End with `/simple-after-plan` (map Notes, "Decide, then build").

## Plan review (2026-11-01)

### Facts checked against the code

- **known** `fixed_user_layers` (`assembler.rs:213`) sets the order. `tail_layers` (`<PlayerInput>`, `<Guide>`) comes after it. `render_cut_user_prompt` subtracts `fixed_user_layers(history_slot(""))` from the history space.
- **known** `render_known_character_entry` is called only from `render_character_cards_layer`. The Quantifier and Options prompts write no marks, so they do not change.
- **known** No test pins the order of the data layers today. `narrator-prompt-layers` checks only that each layer renders. `narrator-guide-layer` checks `<Guide>` after `<PlayerInput>`.
- **known** Two tests pin the marks: `narrator-character-summary` (`"Alice (in room)"`) and `narrator-character-description-lines` (`"Bob (elsewhere)"`), in `src/application/narration/narrator_tests.rs`. It is a seam contract: each new test needs a `CASE` tag (`scripts/validate_seam_cases.py`).
- **known** "Caching rule 3" is not in any doc. It is only in `work/wayfinder-maps/prompt-flexibility/issues/12-prompt-caching.md` and its asset. So the docs have no written reason for the order. The docs to change are `prompt_system_design.md` (line 16: the marks; line 44: the tag order) and `prompt_system.md` (the layer table and the text above it).
- **known** `layout-trials` is one commit behind `main` (`7e28f54d`, scratch and agent files only). The checkout is on `main`.

### Decisions (owner, 2026-11-01)

1. The work runs in the main checkout on `layout-trials`. Done: `layout-trials` is fast-forwarded to `main` (`7e28f54d`) and checked out.
2. The probe check runs on a second probe server on its own port, with a copy of the main probe DB, as in ticket 03. The main probe server on port 3001 stays untouched.

### Steps

1. Test first, in `narrator_tests.rs`:
   - New `CASE narrator-layer-order`: one Narration with a Guide. Assert the order `<KnownCharacters>`, `<Persona>`, `<WorldLore>`, `<ConversationHistory>`, `<GameState>`, `<CharactersPresent>`, `<writing_style>`, `<output_format>`, `<PlayerInput>`, `<Guide>`. This test also pins that the two moved layers are in the fixed list: `tail_layers` follows `post_history_prompt`, so only a fixed layer can come before `<writing_style>`.
   - New `CASE narrator-known-characters-no-marks`: present and absent Characters both render as `- <name>` with no mark. Or change the two mark tests to assert the bare `- Alice` / `- Bob` line and no `(in room)` / `(elsewhere)`. Prefer the change of the two tests: it adds no case.
   - Reorder the list in `narrator-prompt-layers` to the new order (readability only).
2. Run `python build.py test-pattern "narrator" --test-target lib`. The new tests fail.
3. `assembler.rs`: split `render_character_cards_layer` into `render_known_characters_layer` and `render_characters_present_layer`. Only the second needs `in_area_ids` (for `render_present_relationships`). Reorder `fixed_user_layers`.
4. `sections.rs`: remove the `in_area_ids` parameter of `render_known_character_entry`. Do not keep it as `let _ =` (the trial diff does).
5. Docs:
   - `prompt_system_design.md`: remove the marks from "Two tiers of Character awareness". Add a section "The changing blocks after the history": the stable blocks come first, the blocks that change during a story (`<GameState>`, `<CharactersPresent>`) come after the history, so a Room change or a Character move keeps the history in the provider cache. Say how it fits the recency section: the style and format sections and the Input stay last. Update the tag list in "XML sections as content labels".
   - `prompt_system.md`: the new layer table: 0 System, 1 Known Characters, 2 Player, 3 World Info, 4 History, 5 Game State, 6 Characters Present, 7 User Input, 8 Guide. Fix the text above it: "nine layers", "Layers 2, 7 and 8 can be absent", the splice sits between Layer 6 and Layer 7, "Layers 1 and 6 each sort their list". The History section stays true: the moved layers still count in the history space.
6. `python build.py test-pattern "narrator" --test-target lib`, then `python build.py clippy`, then `python build.py validate-docs` and `python build.py vale`.
7. Probe: one Narration on a second probe server on its own port (decision 2), built from this code. Read the user prompt of the LLM record and check the order and the bare names.
8. `/simple-after-plan`. It ends with the full `python build.py`.

### Not in scope

- The Quantifier and Options prompts.
- The size effect: a Room change still changes the size of the fixed part, so it can still move the history start by a block. That is true today too.
- An output-quality judgement of the cards near the end of the prompt (ticket 03 recorded it as "does not decide").
- The OpenRouter `session_id` and the other cache rules of ticket 12.

### What already exists

- `tmp/lt03/candidate.diff`: the split and the order. Reuse it, without the `let _ =` line.
- `prompt_block` and `sent_prompt` in `narrator_tests.rs`: the test helpers for one layer and one sent prompt.

### Failure modes

| Failure | Effect | Handled by |
|---|---|---|
| A moved layer goes into `tail_layers` | The history space does not count it; the overflow guard cuts on every call and the cache breaks | `narrator-layer-order` (order before `<writing_style>`) |
| A mark stays in `<KnownCharacters>` | The prefix breaks when a Character moves | the changed mark tests |
| No Character present | `<CharactersPresent>` writes its fallback sentence after the history | unchanged behaviour; `narrator-prompt-layers` |
| Impersonate (no `<Persona>`) or Event Narration (no `<PlayerInput>`) | Empty layers drop in `join_layers`; the order of the rest holds | `narrator-impersonate-layer`, `narrator-blank-player-input-renders-nothing` |
| A doc keeps the old order | A reader moves `<GameState>` back before the history | step 5, the new design section |

Done when the engine renders the new order, the tests pin it, the docs match it, and the full `python build.py` passes. One Narration on a probe server shows the new order.

## Answer

Resolved on 2026-11-01. The engine renders the new order, the tests pin it, the docs match it, and the full gate passes. Nothing is committed. The work is on branch `layout-trials`, in the main checkout.

### Files changed

- **known** `src/application/prompting/assembler.rs`: `render_character_cards_layer` is split into `render_known_characters_layer` and `render_characters_present_layer`. A small helper `in_area_ids` serves the second one. `fixed_user_layers` now runs KnownCharacters, Persona, WorldLore, history, GameState, CharactersPresent, `post_history_prompt`. `tail_layers` is unchanged.
- **known** `src/application/prompting/builders/sections.rs`: `render_known_character_entry` has no `in_area_ids` parameter and no mark. No `let _ =`.
- **known** `src/application/narration/narrator_tests.rs`: tests, see below.
- **known** `docs/diataxis/explanation/prompt_system_design.md`: no marks in "Two tiers of Character awareness"; the new section "The changing blocks after the history" (with the 55.2 % to 78.8 % gain and the link to the recency section); the tag list and the order in "XML sections as content labels"; the recency section says "after `<CharactersPresent>`".
- **known** `docs/diataxis/reference/narrative/prompt_system.md`: nine layers (0 to 8), the layer table in the new order, and the text above it fixed (absent Layers 2, 7 and 8; splice between Layer 6 and Layer 7; Layers 1 and 6 sort their list).
- **known** I did not touch `work/wayfinder-maps/layout-trials/issues/10-separate-session-per-call-kind.md`. It was already modified before I started.

### Tests

- **known** New `CASE narrator-layer-order` (`test_the_changing_blocks_follow_the_history`): the full order of ten tags, with a Guide.
- **known** Changed `narrator-character-summary` and `narrator-character-description-lines`: they assert the bare `- Alice\n` / `- Bob\n` line and no `(in room)` / `(elsewhere)`. No new case for the marks.
- **known** Reordered the layer list in `narrator-prompt-layers`.
- **known** Test first: the three changed or new tests failed before the code change (`python build.py test-pattern "narrator" --test-target lib`). After it, 70 of 70 passed.

### Probe evidence

- **known** A second probe server ran on port 3005, from `target/debug` (not `target/ui-investigator`), on a copy of the main probe DB made with the SQLite backup call (`target/debug/chronicler_3005.db`, removed after the check). It ran from the main checkout, so it found `.env`. I read no key.
- **known** One `POST /action` ("I look around the room.", Game 8) made the Narrator call, LLM record 1034 (`narrator`, OpenCode Go, `deepseek-v4.1-flash`). Positions in its user prompt: `<KnownCharacters>` 0, `<Persona>` 1272, `<WorldLore>` 2158, `<ConversationHistory>` 2645, `<GameState>` 55574, `<CharactersPresent>` 55731, `<writing_style>` 59316, `<output_format>` 59453, `<PlayerInput>` 60924, no `<Guide>` (a plain turn). The order matches the plan.
- **known** `<KnownCharacters>` lists `- Carla`, `- Gabriella`, `- Jezebel` and so on. The prompt holds neither `(in room)` nor `(elsewhere)`.
- **known** The Quantifier call of the same turn (record 1035) holds none of the Narrator tags. It did not change.
- **known** I stopped the port 3005 server (`curl` fails). The port 3001 server stayed up and untouched (`curl` gives 200).

### Checks

- **known** `python build.py clippy`, `validate-docs` and `vale` pass.
- **known** Full `python build.py` (log `logs/build_20261101_221753.log`): every step OK. Guardrail tests 309 passed. Integration 1624 passed, 3 skipped. Browser 44 passed.
- **inferred** `/simple-after-plan`: no subagent tool was available in this session, so I did the steps myself. The comment finder showed no new comment in the changed Rust (only the new `CASE` line). I re-read both docs for the order and the numbering. I made one change: the design doc no longer cites "ticket 03" (a scratch ticket); it says "a trial". Step 6 (`/retro`) is not done.

### Open points

- **known** The opening tag `<KnownCharacters>` runs straight into `- Carla` with no newline. This was so before the change. It is not changed.

### `/simple-after-plan` in the top session (coordinator, 2026-11-01)

The implementer could not start subagents, so the coordinator ran the skill. The sections above are the implementer's record. This section gives the state after the reviews.

- **known** The comment pass (session `86b9449e`) fixed one `CASE` line. The doc review (session `f640621a`) found 14 problems. The coordinator fixed 12. The Standards review (session `b2d72e25`) and the Spec review (session `8662fbb9`) found no missing requirement.
- **known** Changes after the reviews:
  - `assembler.rs`: the helper `in_area_ids` is gone ("area" is on the `_Avoid_` list of `CONTEXT.md`, and it had one caller). The set is a local `present_ids` in `render_characters_present_layer`.
  - `narrator_tests.rs`: the "no presence mark" check is one new case, `CASE narrator-known-entry-has-no-presence-mark`. The two summary tests check only the bare name line. `narrator-layer-order` has no `<Guide>` (`narrator-guide-layer` pins it). `narrator-prompt-layers` keeps its old list, because it checks only `contains`.
  - `prompt_system_design.md`: the new section is one paragraph, "Stable blocks before the history, changing blocks after it". It gives the reason only (the owner found the longer text to be sediment). It has no trial figures and no change history. The figures stay in this map: [Try the changing blocks after the history](03-changing-blocks-after-history.md), "The numbers". The tag list in "XML sections as content labels" links to the reference layer table.
  - `prompt_system.md`: only Layers 7 and 8 render nothing for a blank body (a fix of an older error).
- **known** Full `python build.py` after the fixes: `logs/build_20261101_234827.log`. Guardrail tests 309, integration 1625 (3 skipped), browser 44, all passed.
- **known** Retro: new issue [A delegated agent is told to run `/simple-after-plan`, which it cannot run](../../retro-issue-bucket/issues/99-delegated-agent-told-to-run-simple-after-plan.md), and new evidence in [A ticket's doc step dictates sentences that break the doc rules](../../retro-issue-bucket/issues/95-ticket-doc-steps-break-the-doc-rules.md).
