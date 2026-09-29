# Stop world creation from silently overwriting an existing key

Type: task (AFK)
Status: open
Blocked by: —

## Question

Creating a world with a key that already exists silently replaces the existing world: `create_world` delegates to `seed_world`, whose SQLite branch is `INSERT OR REPLACE INTO worlds (key, ...)`. No warning, no confirmation, and the existing world's name, description, map and scenarios are gone. I reproduced it on a throwaway key (two creates, same key `qa_t05_world_a`, different data; the DB kept only the second). What change prevents the overwrite (refuse, or confirm) while keeping seeding idempotent for bootstrap?

## Context

- Finding 05.F5 (P1, data loss). Live DOM + DB evidence in the ticket 05 answer; screenshot 53 shows the list after the second create; the earlier partial run saw the same (`40-world-duplicate-key-silent-overwrite.png`).
- `seed_world` is shared with startup seeding of `data/worlds/*`, where replace-on-key is the intended behaviour. The fix must distinguish the bootstrap path from the user-facing create path (for example a separate storage method, or a pre-check in `create_world_handler`).
- The InMemory backend of `seed_world` currently *ignores* a duplicate key instead of replacing it — the two backends disagree, which the fix should also settle.

## Done when

- A user-facing create with an existing key is refused with a clear message, or requires explicit confirmation; bootstrap seeding is unchanged.
- A test covers the duplicate-key create; tier by `tests/STRATEGY.md`.
- `python build.py` is green. Commit after user approval.
