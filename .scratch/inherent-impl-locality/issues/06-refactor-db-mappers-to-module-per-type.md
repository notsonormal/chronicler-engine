# 06 — Refactor Db* mappers to module-per-type

Type: task
Status: resolved
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

## Answer

Resolved by consolidating each stray `impl DbX` block from `storage/Xs.rs` into the type's defining `models/X.rs` file, plus splitting the two multi-type model files so every `Db*` type owns its own file. `build.py` fully green (143s, all 12 steps).

**User decisions (this session):**
1. **No file rename.** The ticket's target shape renamed `models/character.rs` → `models/db_character.rs` (stem matches type). The user ruled this out as unimportant. Files keep the existing `<concept>.rs` convention (`character.rs`, `game.rs`, …). The stem-matches-type nicety is not encoded in any guardrail, and a rename would strand the multi-type files (`world.rs`, `message.rs`) in inconsistent half-renames.
2. **Split the multi-type files.** The user directed splitting `models/world.rs` (held `DbWorld` + `DbMap`) and `models/message.rs` (held `DbMessage` + `DbSwipe`) so each `Db*` type has its own file (module-per-type), even though those three extra types were not violations.

**Part 1 — consolidated the 6 violations** (moved stray `impl DbX` into def file):
- `DbCharacter` — `impl DbCharacter { to_card }` moved `characters.rs` → `models/character.rs`
- `DbGame` — `impl DbGame { to_game }` moved `games.rs` → `models/game.rs` (the `parse_datetime` import stayed in `games.rs` until removal; it is now imported by `models/game.rs`)
- `DbPersona` — `impl DbPersona { to_card }` moved `personas.rs` → `models/persona.rs`
- `DbPromptPreset` — `impl DbPromptPreset { into_preset }` moved `presets.rs` → `models/prompt_preset.rs`
- `DbSettings` — `impl DbSettings { to_settings }` moved `settings.rs` → `models/settings.rs`
- `DbWorld` — `impl DbWorld { to_card }` moved `worlds.rs` → `models/world.rs`

**Part 2 — split multi-type def files** (user-directed, beyond the violation set):
- `models/world.rs` → `world.rs` (keeps `DbWorld`) + new `models/map.rs` (`DbMap`)
- `models/message.rs` → `message.rs` (keeps `DbMessage`) + new `models/swipe.rs` (`DbSwipe`)
- `models/mod.rs` updated: added `pub mod map; pub mod swipe;`, re-exports re-split (`pub use map::DbMap; pub use swipe::DbSwipe; pub use world::DbWorld;` etc.), declarations reordered alphabetically. `mod_purity` preserved (declarations + re-exports only).

**Visibility widening.** The moved methods (`to_card`, `to_game`, `into_preset`, `to_settings`) were private (`fn`) and called within their original `storage/Xs.rs` module. After the move they live in `storage::models::X`, a different module from the `storage::Xs` callers, so each was widened to `pub(crate)`. `from_row` stays `pub`. No call-site changes were needed beyond import path fixes for the `DbMap`/`DbSwipe` splits.

**Import path fixes** for the split: `worlds.rs`, `messages.rs`, `mappers/message.rs`, `models/message_tests.rs` updated (`models::world::DbMap` → `models::map::DbMap`; `models::message::DbSwipe` → `models::swipe::DbSwipe`).

**Acceptance note.** The `guardrails_inherent_impl_locality` rule does not exist yet (ticket 01 is deferred until 04/06/07 land — 06 is now the last of these to land). Zero `Db*` violations is therefore verified **structurally**, not by the rule: a re-scan shows every `impl DbX` block now lives in the same file as its `pub struct DbX` def, with exactly one impl block per type. Once 01 is re-created, this cluster will pass clean.

**Scope note.** `DbMap`, `DbMessage`, `DbSwipe` were not violations (def + impl already shared one file). Their split is module-per-type tidiness the user chose to fold into this ticket because it is the same `models/` cluster. Filed under 06 by user direction.
