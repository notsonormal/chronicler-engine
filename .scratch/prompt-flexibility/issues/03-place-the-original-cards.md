# Put the original ST cards in the repo

Type: task
Status: open
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
