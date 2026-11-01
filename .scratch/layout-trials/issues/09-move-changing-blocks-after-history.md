# Move the changing blocks after the history

Type: task (AFK)
Status: open
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

Done when the engine renders the new order, the tests pin it, the docs match it, and the full `python build.py` passes. One Narration on a probe server shows the new order.
