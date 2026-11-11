# Keep the world id stable when SQLite re-seeds a world

Type: task (AFK)
Status: resolved
Blocked by: 29

## Question

SQLite `seed_world` writes with `INSERT OR REPLACE` on `worlds.key`. With `PRAGMA foreign_keys=ON`, replacing a row deletes it and inserts a new one: the world gets a new id, and its `maps` and `characters` rows are cascade-deleted. InMemory `seed_world` keeps the id. Should re-seeding keep the world id and its dependents, and what change makes both backends agree?

## Context

- Surfaced by [Stop world creation from silently overwriting an existing key](29-world-key-silent-overwrite.md) (implementer finding and code review). [reported, not reproduced by the coordinator]
- Latent today: bootstrap skips worlds that already exist (`get_world` check in `src/bootstrap/load.rs`), so no shipped path re-seeds an existing key. [reported]
- Cascades: `src/adapters/driven/storage/utils/plumbing.rs` (~lines 166, 196). The ticket 29 seed tests assert card and map, never the id.
- Likely fix: `INSERT … ON CONFLICT(key) DO UPDATE`, which keeps the row id and does not cascade.

## Done when

- Re-seeding an existing key keeps the world id and its maps and characters on both backends, or the ticket answer records why replace-with-new-id is intended.
- A driven-adapter test asserts the id across a re-seed on both backends.
- `python build.py` is green. Commit after review.

## Answer

**Bug confirmed first.** A new InMemory/SQLite test pair seeds a world and a character, re-seeds the world, and asserts the id, card, map and characters survive. Before the fix the SQLite test failed (`left: 2, right: 1`, "re-seeding must keep the world id"); InMemory passed. `INSERT OR REPLACE INTO worlds` deleted the conflicting row and inserted a new one, and the `ON DELETE CASCADE` foreign keys on `maps` and `characters` wiped the dependents.

**Fix** (`src/adapters/driven/storage/worlds.rs`). `seed_world` uses `INSERT INTO worlds … ON CONFLICT(key) DO UPDATE SET …`, which updates the row in place: same id, no cascade. The map row is updated by `world_id` and inserted only when absent. The write is split into `write_world` / `write_world_row` (a `WorldWriteMode` of `Insert` or `Upsert`) / `write_map_row`. InMemory already kept the id; both backends now agree. `created_at` now survives a re-seed; nothing orders by it.

**UNIQUE item from [Follow up on small issues found during review](36-follow-up-small-review-issues.md) folded in.** `create_world` is a plain `INSERT INTO worlds`. A `SQLITE_CONSTRAINT_UNIQUE` violation maps to `EngineError::WorldAlreadyExists`, so the `worlds.key UNIQUE` constraint is the authority across processes too. `world_key_exists` was removed. InMemory still refuses by scanning for the key. The reviewer checked the mapping cannot misclassify another constraint: in Insert mode the only statement is the worlds insert, whose only UNIQUE is `key`.

**Tests (driven adapter):** `tests/storage/world_storage.rs` — `test_seed_world_keeps_id_and_dependents` / `test_sqlite_seed_world_keeps_id_and_dependents`, identical except storage construction. The ticket 29 refusal tests still pass.

**Gate:** worktree on `d71af65c`: `nextest: 1641 passed, 0 failed, 2 skipped`, browser 24 passed (`build_20260930_203630.log`).

**Code review** (`/code-review`, verdict CLEAN). Judgement calls went to ticket 36.

