# 06 — Refactor Db* mappers to module-per-type

Type: task
Status: ready-for-agent
Blocked by: (none)

## Question

Refactor the `Db*` row-model types so their inherent impls satisfy `guardrails_inherent_impl_locality`.

Current state (per scan, post-`backend/`-flatten commit `6cb2049`):
- `DbCharacter` defined in `adapters/driven/storage/models/character.rs`, impl split across `models/character.rs` + `storage/characters.rs` (was `backend/characters.rs`)
- `DbGame` — same shape (`models/game.rs` + `storage/games.rs`)
- `DbPersona` — `models/persona.rs` + `storage/personas.rs`
- `DbPromptPreset` — `models/prompt_preset.rs` + `storage/presets.rs`
- `DbSettings` — `models/settings.rs` + `storage/settings.rs`
- `DbWorld` — `models/world.rs` + `storage/worlds.rs`

Each split is across two files, neither named after the type.

Target shape — single-file per type, named after the type:
```text
adapters/driven/storage/models/
  db_character.rs      # struct DbCharacter + impl DbCharacter (all methods)
  db_game.rs           # struct DbGame + impl DbGame
  db_persona.rs        # struct DbPersona + impl DbPersona
  db_prompt_preset.rs # struct DbPromptPreset + impl DbPromptPreset
  db_settings.rs      # struct DbSettings + impl DbSettings
  db_world.rs          # struct DbWorld + impl DbWorld
```

The `storage/Xs.rs` impl blocks (character persistence behavior on `Storage` is NOT what's moving — that stays with `Storage` per ticket 03, now resolved). What moves are the `impl DbX { from_row, into_domain, ... }` methods currently living in the `storage/` files.

Wait — verify which impl blocks actually live in `storage/`. The scan showed `impl DbCharacter` in `storage/characters.rs` AND in `models/character.rs`. Before refactoring, the agent must `rg -n "^impl Db" src/adapters/driven/storage/` to enumerate the actual impl blocks and their locations, because the `storage/Xs.rs` files mostly contain `impl Storage` (not `impl DbX`) — the manual scan may have over-counted.

Constraints:
- `build.py` green at every landed step.
- Preserve all call sites (adjust imports).
- Preserve `guardrails_mod_purity` (models/mod.rs contains declarations only).
- Do NOT touch `Storage` or `InMemoryData` (those are ticket 03).
- Do NOT touch trait impls.
- File rename `models/character.rs` → `models/db_character.rs` (file stem now matches type).

Acceptance:
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `Db*` violations.
- Full `build.py` green.
- No new `guardrails_*` failures.
- `models/mod.rs` updated with new `pub mod db_X;` declarations and old ones removed.
