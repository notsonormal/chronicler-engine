# Compare our card and World format with ST and ME

Type: research
Status: resolved
Blocked by: 03

## Question

Which fields of the SillyTavern card spec (V2 and V3) and of ME's character, persona and lorebook formats does our data model drop, merge or rename? What did our conversion change in the text of each card?

## Context

- The original cards: `../assets/source_cards/` (five V2 PNGs, with their raw embedded JSON next to each). See the answer of [Put the original ST cards in the repo](03-place-the-original-cards.md).
- Our format: `data/schemas/*.json`, `src/domain/model/character.rs`, `world.rs`, `scenario.rs`, the Persona model, and `data/worlds/redmist_estate/`, `data/characters/redmist_estate/`, `data/personas/`.
- The conversion scripts: `scripts/extract_sillytavern_png.py`, `scripts/refine_character_json.py` (splits the description into personality, scenario and description fields).
- The "Character Card Format" section of `docs/external_applications/sillytavern_prompt_system.md` lists only 5 fields; it is incomplete.
- ST card fields to check: `description`, `personality`, `scenario`, `first_mes`, `mes_example`, `alternate_greetings`, `system_prompt`, `post_history_instructions`, `character_book` (lorebook), `creator_notes`, depth prompt.
- A known text change: the Persona card says "never knowing his father Bernard Redmist", and the World's `global_rules` say "a grandson he never knew" (see [the stored Impersonate prompt](../assets/impersonate-prompt-157.txt)). Check whether the original card has the same error.

## Output

An asset `assets/card-and-world-format.md`: a field table (ST V2/V3, ME, ours: kept, dropped, merged, renamed), and for each card a short list of what the conversion changed. This ticket is about the format. Fixing the text belongs to the authoring effort.

## Answer

Resolved. The asset is [`assets/card-and-world-format.md`](../assets/card-and-world-format.md). It has field tables for the Character card, the lorebook and the Persona (ST V2, ST V3, ME, ours), and the text changes of each card, the World and the Persona.

- **Kept:** `name`, `description`, `personality`, `scenario`. `personality` and `scenario` were empty in the source; the conversion filled them from the `Personality:`, `Background:` and `Goals:` lines of the description. The prompt labels `scenario` as `Context:`.
- **Renamed:** `first_mes` went into `example_dialogue`. No prompt part reads `example_dialogue`, so no greeting and no dialogue example reaches the model.
- **Dropped:** `mes_example`, `alternate_greetings`, `creator_notes`, `system_prompt`, `post_history_instructions`, `extensions.depth_prompt`, `tags`, `creator`, `character_version`, and the whole `character_book`. All V3-only fields are absent too (no source card is V3).
- **Merged:** the five embedded lorebooks became one World `description` and six new, always-on `global_rules`. The keyword trigger, the position and the per-Character view of each entry are gone.
- **Cut from the text (all five cards):** the Islaport line, `Speech`, `Behaviours`, `Notes`, `Preferences` and the sex section. About half of each description is gone. Four of five cards lost "{{char}} will never speak for {{user}}".
- **Added by us, with no source:** `summary`, `relationships`, Gabriella's trigger, and several World facts ("two weeks ago", "guest wing", "Julian"). `summary` and `relationships` name "Julian", not `{{user}}`.
- **The "father" error** is ours, in two Persona fields (`scenario`, `personality`). The cards say "grandfather". The Persona `description` and `personality` also reuse Carla's words (inferred).
- **Format drift:** `world.schema.json` does not list four fields that `WorldManifest` reads.

Text fixes belong to the authoring effort. The missing card fields (dialogue examples, per-card system and post-history text, depth prompt, keyword lore) are input for [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md).
