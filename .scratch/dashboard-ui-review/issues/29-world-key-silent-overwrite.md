# Stop world creation from silently overwriting an existing key

Type: task (AFK)
Status: resolved
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

## Answer

**Refuse, no confirm step.** A confirm would need a two-request htmx flow and a force flag, a new UI pattern that belongs to [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md). Refusing is deterministic and stops the loss outright.

**Two storage methods separate the paths.**
- `Storage::seed_world` stays the upsert. Bootstrap (`src/bootstrap/load.rs::process_world_dir`) still calls it and keeps replace-on-key.
- `Storage::create_world` no longer delegates to `seed_world`. It returns `EngineError::WorldAlreadyExists(key)` when the key exists. The check runs inside one `with_backend_mut`, so there is no check-then-write gap in one process.
- `WorldCatalogue::create_world` maps that to `ApplicationError::Validation("A world with key '…' already exists")`. The handler is unchanged: it returns `400 "Failed to create world: …"`, which the shell's `htmx:beforeSwap` handler shows as a toast. The panel is not wiped. Inline errors stay with [Stop server errors from wiping whole panels](25-stop-errors-wiping-panels.md).

**Backends aligned.** InMemory `seed_world` used to ignore a duplicate key. It now replaces card and map like SQLite `INSERT OR REPLACE`. Both backends refuse the same way on `create_world`.

**Tests.**
- Tier 1 (HTTP): `tests/http/worlds.rs::test_world_create_existing_key_is_refused_http`, new scenario `docs/specs/worlds.md` 25.7.
- Driven adapter: `tests/storage/world_storage.rs` — `test_create_world_duplicate_key_refused`, `test_sqlite_create_world_duplicate_key_refused`, `test_seed_world_replaces_existing_key_in_memory`, `test_sqlite_seed_world_replaces_existing_key`. These replace `test_create_world_duplicate_key_idempotent`, which asserted the silent overwrite.
- Unit: `src/application/world_catalogue_tests.rs::test_create_world_duplicate_key_is_refused`.

**Gate:** `nextest: 1639 passed, 0 failed, 2 skipped`, browser 23 passed (worktree log `build_20260930_191141.log`).

**Code review** (`/code-review`, verdict CLEAN; judgement calls only, not fixed here):
- The SQLite refusal is a pre-check before `INSERT OR REPLACE`. It is atomic in one process, not across two processes on one DB file. Letting the `key UNIQUE` constraint refuse would make it authoritative.
- Scenario 25.7 asserts only name and key are unchanged, not description, map or scenarios. Its When omits the endpoint, unlike 25.1–25.5.
- The new backend test pairs vary literals between InMemory and SQLite (`unit_test_standards.md` Pattern 2); the file already deviates the same way.
- `tests/http/worlds.rs` header does not mention create.

**New ticket:** [Keep the world id stable when SQLite re-seeds a world](35-sqlite-reseed-keeps-world-id.md).

