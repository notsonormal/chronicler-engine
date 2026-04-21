# chronicler_engine/scripts/

## Responsibility

Python helper scripts for game data extraction, image processing, character JSON refinement, and coverage reporting.

## Design

**Scripts:**
- `extract_sillytavern_png.py` — Extracts SillyTavern V2 PNG character cards (base64 `chara` metadata). Produces `NpcCard` JSON files and aggregates `character_book` entries into a `WorldCard`. Uses Pillow to read PNG metadata.
- `extract_images.py` — Extracts character images from SillyTavern PNGs. Saves full image + cropped portrait (top 40%) to `data/images/`.
- `refine_character_json.py` — Post-processes character JSON files in `data/characters/`. Parses structured sections (Personality, Appearance, Background, Goals) from the `description` field and redistributes them into proper `personality` and `scenario` fields.
- `coverage_summary.py` — Reads `tmp/coverage/coverage.json` (or fallback `coverage.json`), prints combined coverage stats, and lists files below 80% line coverage.

## Flow

1. SillyTavern PNG → `extract_sillytavern_png.py` → character JSON + world JSON
2. Character PNG → `extract_images.py` → full + cropped images
3. Character JSON → `refine_character_json.py` → refined JSON with structured fields
4. Test run → `coverage_summary.py` → coverage report

## Integration

- **Input**: SillyTavern PNG character cards, JSON character files, coverage JSON
- **Output**: Chronicler Engine data files (`data/characters/`, `data/images/`, `data/world/`)
