# Plan: Remove Old Migrations from `src/storage/db.rs`

## Context
`chronicler_engine/src/storage/db.rs` contains `run_migrations()`, which incrementally upgrades SQLite schema from v1 through v9. The user runs `build.py --cleanup` before builds, ensuring no stale databases exist. All current and future databases will be created fresh, so supporting v1-v8 upgrade paths is unnecessary technical debt.

## Goal
Replace the incremental migration logic with a single idempotent schema initialization that creates the final v9 schema directly.

## Approach
1. **Run `build.py --cleanup`** to kill lingering processes and remove build artifacts (as requested).
2. **Refactor `run_migrations`** in `src/storage/db.rs`:
   - Keep the `run_migrations` function name, `PRAGMA user_version` check, and the `if version < N` migration pattern so future migrations have a clear template to follow.
   - Remove all old incremental migration bodies (v1 through v9) and their helper functions (`merr`, `recreate_prompt_presets_table`).
   - Replace them with a single `if version < 9` block that creates the final schema directly:
     - Create all tables with `IF NOT EXISTS` using the final v9 schema:
       - `games`
       - `game_state_snapshots` (with `ON DELETE CASCADE` FK)
       - `messages` (with `ON DELETE CASCADE` FK)
       - `message_swipes` (with `ON DELETE CASCADE` FK)
       - `llm_messages`
       - `prompt_presets` (v8 schema: no `prompt_text`, has `role`, `instructions`, `writing_style`, `output_format`)
     - Create all indexes with `IF NOT EXISTS`.
     - Insert the default game row only if `games` is empty (preserves idempotency for reopening DBs).
     - Set `user_version = 9`.
   - Leave a concise commented template for future migrations.
3. **Verify tests pass** — `db_tests.rs` covers table existence, default game, cascade delete, and reopen idempotency. No test changes expected since behaviour is identical for new DBs.
4. **Run `python build.py`** to validate fmt + clippy + tests.

## Files to Modify
- `chronicler_engine/src/storage/db.rs`

## Verification
- `cargo test` passes (especially `test_db_reopen_idempotent` and `test_db_cascade_delete_game`).
- `cargo clippy` clean.
