# Keep the world id stable when SQLite re-seeds a world

Type: task (AFK)
Status: open
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
