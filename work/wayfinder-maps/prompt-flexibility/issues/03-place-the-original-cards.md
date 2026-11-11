# Put the original ST cards in the repo

Type: task
Status: resolved
Blocked by: —

## Question

The owner has the original SillyTavern character cards that the World data came from. They must be in the repo so that later tickets and the authoring effort can read them.

## Checklist (owner)

1. Put each original card (PNG, and JSON if you have it) in `data/source_cards/` (owner, charting: committed in the repo).
2. Add any original World or lorebook file that you have.
3. Tell the agent which card belongs to which character and Persona in `data/characters/redmist_estate/` and `data/personas/`.

Known: `src/bootstrap/load.rs` seeds only `worlds/`, `characters/`, `personas/`, `prompt_presets/` and `settings.json`, and `scripts/validate_data.py` checks only those paths. So the gate and the seed do not read `data/source_cards/`.

## Agent part

Extract the embedded JSON of each PNG next to it (the card JSON is in a PNG text chunk; `scripts/extract_sillytavern_png.py` handles the images of these cards). Record the file list and the card spec version of each card in the answer.

## Answer

Done. The owner added five PNG cards to `data/source_cards/`. The owner then moved them to [`assets/source_cards/`](../assets/source_cards/), so they are scratch assets and not engine data. The agent extracted the raw embedded JSON (`chara` text chunk, base64) next to each PNG, without changes. `scripts/extract_sillytavern_png.py` was not used: it converts the card, and this ticket needs the original.

| Card file (`assets/source_cards/`) | Our Character | Spec | Lorebook entries | `alternate_greetings` |
|---|---|---|---|---|
| `main_carla-682a3640394b_spec_v2.png` / `.json` | `data/characters/redmist_estate/carla.json` | `chara_card_v2` 2.0 | 4 | 5 |
| `main_gabriella-244da5164d7b_spec_v2.png` / `.json` | `.../gabriella.json` | `chara_card_v2` 2.0 | 3 | 7 |
| `main_jezebel-8b603341a57a_spec_v2.png` / `.json` | `.../jezebel.json` | `chara_card_v2` 2.0 | 5 | 4 |
| `main_lisette-3edd9af5acee_spec_v2.png` / `.json` | `.../lisette.json` | `chara_card_v2` 2.0 | 5 | 5 |
| `main_louise-f72bef94a194_spec_v2.png` / `.json` | `.../louise.json` | `chara_card_v2` 2.0 | 5 | 5 |

Facts for later tickets:

- **No V3 card.** No PNG has a `ccv3` chunk. Ticket 04 can compare against V2 only for the source; V3 stays a spec reference.
- **Mapping is by name.** The card `name` of each file matches one Character file.
- **No Persona card and no separate World or lorebook file.** The owner has none. The cards write the player as `{{user}}`. So `data/personas/julian.json` has no source card; it was written for the engine.
- **The World lore is in the cards.** Each card has an embedded `character_book`. The entries overlap but differ per card: Redmist Family, Redmist Estate, Bernard Redmist, Gabriella Redmist, and (in three cards) Carla Jess. Gabriella's book has no name and no Gabriella entry. All cards are by `LukeyPoo488`. Each card has `extensions.depth_prompt`; Jezebel also has `extensions.chub`.
- **Non-empty fields of every card:** `name`, `description`, `first_mes`, `mes_example`, `alternate_greetings`, `character_book`, `tags`, `creator`, `extensions`, `avatar`; `creator_notes` in all but Jezebel; `character_version` in Jezebel only. `personality`, `scenario`, `system_prompt` and `post_history_instructions` are empty.
- **The "father" error is ours (known).** Four cards say "never knew you had a grandfather". The Persona card's "never knowing his father Bernard Redmist" came from our conversion, not from the source. Ticket 04 records it with the other text changes.
- **A conversion fact for ticket 04 (known).** `scripts/extract_sillytavern_png.py` writes the card's `first_mes` into our `example_dialogue` field and drops `mes_example`.
- **Gate, seed and package.** The cards are under `.scratch/`, so `scripts/validate_data.py`, `src/bootstrap/load.rs` and the packaging step of `build.py` (which copies all of `data/`) do not read them. `git check-ignore` reports nothing, so the files will be committed.
