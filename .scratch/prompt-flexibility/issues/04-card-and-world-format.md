# Compare our card and World format with ST and ME

Type: research
Status: open
Blocked by: 03

## Question

Which fields of the SillyTavern card spec (V2 and V3) and of ME's character, persona and lorebook formats does our data model drop, merge or rename? What did our conversion change in the text of each card?

## Context

- Our format: `data/schemas/*.json`, `src/domain/model/character.rs`, `world.rs`, `scenario.rs`, the Persona model, and `data/worlds/redmist_estate/`, `data/characters/redmist_estate/`, `data/personas/`.
- The conversion scripts: `scripts/extract_sillytavern_png.py`, `scripts/refine_character_json.py` (splits the description into personality, scenario and description fields).
- The "Character Card Format" section of `docs/external_applications/sillytavern_prompt_system.md` lists only 5 fields; it is incomplete.
- ST card fields to check: `description`, `personality`, `scenario`, `first_mes`, `mes_example`, `alternate_greetings`, `system_prompt`, `post_history_instructions`, `character_book` (lorebook), `creator_notes`, depth prompt.
- A known text change: the Persona card says "never knowing his father Bernard Redmist", and the World's `global_rules` say "a grandson he never knew" (see [the stored Impersonate prompt](../assets/impersonate-prompt-157.txt)). Check whether the original card has the same error.

## Output

An asset `assets/card-and-world-format.md`: a field table (ST V2/V3, ME, ours: kept, dropped, merged, renamed), and for each card a short list of what the conversion changed. This ticket is about the format. Fixing the text belongs to the authoring effort.
