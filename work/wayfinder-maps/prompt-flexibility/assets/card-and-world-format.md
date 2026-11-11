# Card and World format: ST, ME and ours

Ticket: [Compare our card and World format with ST and ME](../issues/04-card-and-world-format.md).
Sources: the five V2 cards in [`source_cards/`](source_cards/), SillyTavern at `06bde939`, Marinara Engine at `457e827a`, and our `data/`, `data/schemas/` and `src/domain/model/`.

## Answer in short

- **known** Our Character card keeps 4 text fields of the ST card: `name`, `description`, `personality`, `scenario`. It has no field for `mes_example`, `alternate_greetings`, `system_prompt`, `post_history_instructions`, `extensions.depth_prompt`, `character_book`, `creator_notes`, `tags`, `creator` or `character_version`.
- **known** Our `example_dialogue` field holds the card's `first_mes`, not its `mes_example`. No part of the prompt reads `example_dialogue` (`src/application/prompting/` has no reference to it). So the dialogue examples and the greeting of each card do not reach the model.
- **known** The conversion cut each card description to about half (3077–3526 characters down to 1529–1821). It kept the intro line, `Personality`, `Appearance`, `Background` and `Goals`. It dropped `Speech`, `Behaviours`, `Notes`, `Preferences` and `Sex Info` / `Sexual Info` in all five cards.
- **known** The rule "{{char}} will never speak for {{user}} under any circumstances" is in all five source descriptions. Only Carla keeps it.
- **known** We have no lorebook. The five embedded `character_book`s became one hand-written World `description` and six always-on `global_rules`. The keyword trigger and the per-Character view of each entry are gone.
- **known** We add data that ST and ME do not have: `summary`, `relationships` (with `static` and `dynamic` text), `triggers`, `inventory`, two image paths, the map, and the World's scenarios. All `summary` and `relationships` text is new; no source card contains it.
- **known** The "father" error is in our Persona, in two fields. The source cards and our World say "grandfather" / "grandson".

## 1. Character card fields

"Prompt" says whether the field reaches our prompt today (from [What reaches the model](what-reaches-the-model.md)).

| ST V2 field | ST use | ST V3 | ME | Ours | Prompt |
|---|---|---|---|---|---|
| `name` | `{{char}}`, sender label | same | same | `name` (kept) | yes, as the KnownCharacters line and the card header |
| `description` | the description marker | same | same; also the default first field of the character marker | `description` (kept, cut: only the intro line and `Appearance`) | yes, `Description:` |
| `personality` | the personality marker, through `personality_format` | same | same | `personality` (kept; filled from the `Personality:` line of the description, because the source field is empty) | yes, `Personality:` |
| `scenario` | the scenario marker, through `scenario_format` | same | same; a group scenario can replace it | `scenario` (renamed in use: filled from `Background:` + `Goals:`; the prompt labels it `Context:`) | yes, `Context:` |
| `first_mes` | the first chat message | same | same | `example_dialogue` (renamed) | no |
| `alternate_greetings` | swipes of the first message | same | same | dropped | no |
| `mes_example` | the dialogue examples marker, split at `<START>` | same | same; `dialogue_examples` marker, or a field of the character marker | dropped | no |
| `creator_notes` | UI only | same; plus `creator_notes_multilingual` | same; a macro field | dropped | no |
| `system_prompt` | replaces the `main` prompt when `prefer_character_prompt` is on | same | same; a default field of the character marker | dropped (empty in all five) | no |
| `post_history_instructions` | replaces the `jailbreak` prompt when `prefer_character_jailbreak` is on | same | same; injected after the history | dropped (empty in all five) | no |
| `tags`, `creator`, `character_version` | UI only | same | same; `tags` is a macro field | dropped | no |
| `extensions.depth_prompt` | in-chat injection at a depth (prompt, depth, role) | same | same | dropped (empty prompt, depth 3, in all five) | no |
| `extensions.talkativeness` | group chats | same | same | dropped (absent in the five cards) | no |
| `extensions.chub` | Chub metadata | same | kept as an extension | dropped (Jezebel only) | no |
| `character_book` | embedded World Info, imported as a World Info file linked through `extensions.world` | same; entries add `use_regex` and `@@` decorators | imported as a separate lorebook linked to the Character (`st-character.importer.ts`) | dropped; see section 2 | no |
| `avatar` | the image | `assets` list | `avatarPath` | `profile_image`, `headshot_image` (ours, not from the card) | no |
| — | — | `nickname`, `group_only_greetings`, `source`, `creation_date`, `modification_date` | the importer keeps the first four and `assets`, `creator_notes_multilingual` as pass-through (`V3_CHARACTER_DATA_FIELDS`) | absent | — |
| — | — | — | `extensions.backstory`, `extensions.appearance` (separate fields, default fields of the character marker), and many ME-only extensions (`rpgStats`, `nameAliases`, `onboarding`, colours, Conversation-mode fields) | absent; ours puts `Appearance:` inside `description` and the backstory in `scenario` | — |
| — | — | — | `summary` (library preview, "Not sent to the model") | `summary` (ours: sent to the model as the KnownCharacters line) | yes |
| — | — | — | — | `id`, `inventory`, `triggers`, `relationships` (ours only) | `relationships` yes; `triggers` drive Event Narration |

Notes:

- **known** ST reads a `ccv3` PNG chunk before a `chara` chunk, but its data model stays V2. When ST writes a card, it copies the V2 data into a `ccv3` chunk with only `spec` and `spec_version` changed (`src/character-card-parser.js`). ST code at the pinned commit does not use `group_only_greetings`, `nickname` or the other V3-only fields.
- **known** The V3-only field names come from the V3 spec text and ME's importer list. ME's list has no `modification_date`. **inferred** ME drops it.
- **known** Our `{{…}}` substitution has no `{{char}}` (`src/domain/model/template.rs`). So the conversion wrote the Character name in place of every `{{char}}`. `{{user}}` stays, and the engine replaces it with the Persona name.
- **known** `data/schemas/world.schema.json` does not list `narrator_mode`, `narrative_perspective`, `narrative_tense` or `options_always_on`, which `WorldManifest` reads. The schema is behind the struct.

## 2. Lorebook

| ST V2 `character_book` | ME lorebook | Ours |
|---|---|---|
| book `name`, `description` | `name`, `description`, `category`, tags, scope (all, disabled, specific chats) | the World `name` and `description` |
| `scan_depth`, `token_budget`, `recursive_scanning` | same, plus `entryLimit`, `maxRecursionDepth`, vector search settings | absent |
| entry `keys`, `secondary_keys`, `selective`, `case_sensitive`, `constant` | same, plus `selectiveLogic`, `matchWholeWords`, `useRegex`, semantic and "decision" activation | absent: every rule is always on |
| entry `content` | `content`, plus a `description` for the knowledge-router agent | one `global_rules` string each; text rewritten (see section 4) |
| entry `insertion_order`, `priority`, `position` (`before_char` / `after_char`), `extensions.depth`, role | `order`, `position` 0, 1, 2 (at depth with role) or 7 (outlet), `sticky`, `cooldown`, `delay`, `group` | absent: `<global_rules>` has one fixed place in the system half |
| linked to one character | linked to characters, personas or a chat, or global | one World, shared by all Characters |

- **known** All entries of the five books are `position: after_char`, `constant: false`, `probability: 100`, keyword-triggered. All five books have `scan_depth` 50 and `token_budget` 500.
- **known** Each card's book gives the Redmist facts from that Character's point of view, for example Carla's "{{char}} hates Gabriella for how she looks down on everyone" and Gabriella's "He was quite a strange man who liked to have {{char}} dress up". One shared `global_rules` list cannot hold a per-Character view.

## 3. Persona

| ST persona | ME persona | Ours (`PersonaCard`) | Prompt (ours) |
|---|---|---|---|
| name | `name` | `name` | `Name:` and the history sender label |
| `description` | `description` | `description` | `Description:` |
| — | `personality` | `personality` | `Personality:` |
| — | `scenario`, `backstory` | `scenario` | `Background:` |
| — | `appearance` | — (inside `description`) | — |
| `position`, `depth`, `role` (in the prompt, Author's Note top or bottom, at depth, or none) | — (the persona marker has a place in the preset) | — (fixed `<Persona>` layer) | — |
| `lorebook` | linked lorebooks (`personaIds`) | — | — |
| `title` | `comment`, `creator`, `personaVersion`, `creatorNotes`, `tags` | `key` | — |
| — | — | `example_dialogue`, `summary`, `inventory`, images | `example_dialogue` and `summary` do not reach the prompt (**inferred** from section 1 of [What reaches the model](what-reaches-the-model.md)) |

- **known** ST keeps only one text field for a persona. ME and ours split it.
- **known** No source card exists for `data/personas/julian.json` (see [Put the original ST cards in the repo](../issues/03-place-the-original-cards.md)).

## 4. What the conversion changed, card by card

The scripts `scripts/extract_sillytavern_png.py` and `scripts/refine_character_json.py` explain most of the change. **inferred** Hand edits came after them: the scripts would keep the Islaport line in `description`, but no stored card has it. Git history does not show the conversion: the oldest commit that `git log --follow` finds for `carla.json` is `d14ee3fa`, a line-end change.

Changes in all five cards (**known**, by text comparison):

- The Islaport intro line ("{{char}} lives in the city of Islaport, …") is gone from `description`. The World `description` has it.
- `Speech`, `Behaviours`, `Notes`, `Preferences` and `Sex Info` / `Sexual Info` are gone.
- The `Personality:` label line moved to `personality`. `Background:` lost its label and moved, with `Goals:`, to `scenario`.
- `first_mes` is in `example_dialogue`. `mes_example` and every `alternate_greetings` entry are gone.
- `{{char}}` is the Character name.
- `summary` and `relationships` are new text. They name the Persona as "Julian", not `{{user}}`, so they bind the card to one Persona.

| Card | Other changes |
|---|---|
| Carla | Keeps "Carla will never speak for {{user}} under any circumstances" (in `scenario`). Keeps the `*action*` asterisks in `example_dialogue`. `scenario` holds `\r\n` line ends from the source. The World's only scenario, "Bodyguard Introduction", is a shortened rewrite of this `first_mes` without asterisks. New relationships: Gabriella "deep suspicion … resents how she treated Bernard" (the book says she hates how Gabriella looks down on everyone), Jezebel "worked together to protect Bernard's interests" (no source), Louise (no source). Lost from the book: she found Bernard easy to protect and "quite perverted". |
| Gabriella | Loses the "never speak for {{user}}" rule. `example_dialogue` loses its asterisks and one double space. Has a `triggers` entry, "Gabriella Introduction" in `entrance_hall` (ours only). Her book has no name and no Gabriella entry; its four Bernard sentences (late husband, "easy mark", dress-up, "the last laugh") are not in our data. New relationships with Jezebel, Carla, Lisette (no source). |
| Jezebel | Loses the "never speak for {{user}}" rule. `example_dialogue` loses its asterisks. Lost `Notes`: "{{char}} will act as an advisor for {{user}}." Lost from the book: Bernard was her mentor; she despises Gabriella. Our relationship text with Gabriella and Carla is new. Only card with `extensions.chub` and a `character_version`. |
| Lisette | Loses the "never speak for {{user}}" rule. `example_dialogue` loses its asterisks. `description` adds a full stop at the end of the intro line. Lost `Speech`: she "will never raise her voice at {{user}}". Lost `Notes`: she "has never been a personal maid". Lost from the book: she never met Bernard; Gabriella mistreated her. |
| Louise | Loses the "never speak for {{user}}" rule. `example_dialogue` loses its asterisks. Lost `Speech`: "a heavy French accent". Lost `Notes`: her focus on her own smell. `summary` says "from Paris", which the source has in `Background`. `summary` mixes "Julian's" with "they". |

## 5. World and Persona text

World (`data/worlds/redmist_estate/world.json`), against the five books (**known**):

- `description` merges the Redmist Family and Redmist Estate entries and the Islaport intro line. It adds "on the eastern seaboard". It drops "second only to the Small Family Estate nearby".
- `global_rules` are not the book entries. They are six new sentences. New facts with no source: Bernard "died two weeks ago"; Gabriella lives in "the estate's guest wing"; the heir's name "Julian Redmist"; Jezebel "manages … Bernard's charitable foundations"; Islaport has "a busy harbour". The first rule says "a grandson he never knew", which agrees with the cards.

Persona (`data/personas/julian.json`):

- **known** The "father" error is in two fields. `scenario`: "never knowing his father Bernard Redmist". `personality`: "much like the father he never knew". The source cards say "You never knew you had a grandfather" (`creator_notes` of four cards) and "widow of {{user}}'s grandfather" (Gabriella `description`). Bernard is the grandfather.
- **inferred** The Persona `description` reuses Carla's `Appearance` words ("relatively attractive", "strong body, muscular", "short black hair", "blue eyes"). Its `personality` reuses Carla's traits ("stubborn and dedicated").

## 6. Facts for later tickets

- For [Decide which parts of a Narrator-role call an author can change](../issues/06-the-layout-model.md): ST and ME treat dialogue examples, a per-card system prompt, a per-card post-history text, a depth prompt and a keyword lorebook as card data with a place in the prompt. We have no field for any of them. Our three fixed labels (`Description:`, `Personality:`, `Context:`) also rename the meaning of `scenario`.
- For the same ticket: `example_dialogue` exists in our model and storage, but no prompt part reads it. Either a layout gives it a place, or it is dead data.
- For the authoring effort (out of scope here): the dropped `Speech`, `Behaviours`, `Notes` and `Preferences` text, the "never speak for {{user}}" rule in four cards, the "father" error, the Persona text copied from Carla, and the "Julian" name in `summary` and `relationships`.
