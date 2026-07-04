# Fix Boot Path: Restore Auto-Create Game with `--persona` CLI Flag

## Context

ADR-026 relocated persona binding from world to game. The original plan (Sub-decision B) made `resolve_game_id` return `Option<u64>` — when no game exists, server boots to "limbo" (no active game). This broke 22 browser tests that rely on auto-active-play at startup, and 4 HTTP tests that submit forms missing the new required `persona_key` field.

Root cause: old `resolve_game_id` auto-created a game when none existed, using `world.player_key` for the persona. The plan removed that auto-create path entirely, leaving no game on first boot.

The v13 migration also deviated from plan: impl backfills `persona_key`/`persona_name` from `worlds.player_key` instead of wiping `games`. Since tests always start with fresh DBs (`server.rs::cleanup_stale_db` wipes before each browser test; `build.py --cleanup` wipes target dir), the backfill logic is dead code that adds complexity for no benefit.

## Approach

Restore auto-create-game behavior. Add `--persona` CLI flag (default `julian`) mirroring `--world` (default `redmist_estate`). When no game exists for the requested world, `resolve_game_id` auto-creates one using the CLI persona. Simplify v13 migration to schema-only (no backfill). Fix the 4 HTTP tests that submit forms without `persona_key`.

**Why `--persona julian` default is consistent with ADR-026:** The ADR rejects world-level `default_persona` as a *runtime concept*. A CLI bootstrap flag is not a runtime default — it's an explicit startup parameter for auto-creating the initial game, same as `--world` auto-selects the initial world. The Games-tab New Game form still requires an explicit persona pick; no runtime fallback exists.

## Files to modify

1. `chronicler_engine/src/cli.rs` — add `--persona` flag
2. `chronicler_engine/src/bootstrap/init_game.rs` — restore auto-create, take `persona_key`
3. `chronicler_engine/src/bootstrap/run.rs` — revert limbo path, pass `args.persona`, restore linear boot
4. `chronicler_engine/src/storage/db.rs` — simplify v13 migration (drop backfill)
5. `chronicler_engine/tests/http/fragment.rs` — add `persona_key` to 4 form submissions
6. `chronicler_engine/tests/test_utils/server.rs` — add `persona` param to server spawn
7. `chronicler_engine/tests/test_utils/browser.rs` — pass persona through
8. `chronicler_engine/tests/browser/*.rs` — pass `test_player` as persona (4 files, constant only)
9. `chronicler_engine/docs/adr/adr-026-persona-relocation-to-game.md` — amend: replace Sub-decision B with CLI flag
10. `chronicler_engine/CONTEXT.md` — update "Non-terms" to reflect CLI `--persona` flag
11. `chronicler_engine/docs/CHANGELOG.md` — update entry (remove "wipes games" claim, add `--persona` flag)

## Reuse

- `resolve_game_id` pattern from git HEAD (`init_game.rs:25-50`): auto-create game via raw INSERT. Reuse exactly, add `persona_key` + `persona_name` to the INSERT.
- `storage::get_persona` (`personas.rs:28`): already used by `run.rs` to resolve persona. No change needed.
- `list_game_names_for_world` (`run.rs:233`): used by auto-create to generate unique game name. Currently `#[allow(dead_code)]` — remove the allow, it's used again.
- `TestServer` / `start_server_with_env` (`tests/test_utils/server.rs`): add `persona: &str` param, thread through to CLI args.

## Steps

### Step 1 — CLI: add `--persona` flag

**`src/cli.rs`** — add field to `Args` struct after `pub world: String`:

```rust
/// Specify which persona to use when auto-creating a game
#[arg(long, default_value = "julian")]
pub persona: String,
```

### Step 2 — `resolve_game_id`: restore auto-create with persona

**`src/bootstrap/init_game.rs`** — change signature back to `Result<u64>` (not `Option`), take `persona_key` and `persona_name`:

```rust
pub(crate) fn resolve_game_id(
    db_pool: &crate::storage::db::DbPool,
    world: &WorldCard,
    persona_key: &str,
    persona_name: &str,
) -> crate::error::Result<u64> {
    match find_latest_game_for_world(db_pool, &world.key)? {
        Some((id, name)) => {
            tracing::info!("Loaded existing game '{name}' (id={id})");
            Ok(id)
        }
        None => {
            let existing_names = list_game_names_for_world(db_pool, &world.key)?;
            let name = crate::model::game::generate_game_name(&world.name, &existing_names);
            let conn = db_pool.conn();
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                rusqlite::params![&world.name, &world.key, persona_key, persona_name, &name, &now],
            )
            .map_err(|e| crate::error::EngineError::Config(format!("Failed to create game: {e}")))?;
            let id = conn.last_insert_rowid() as u64;
            tracing::info!("Created new game '{name}' (id={id}) with persona '{persona_key}'");
            Ok(id)
        }
    }
}
```

Restore `list_game_names_for_world` import (remove `#[allow(dead_code)]` from `run.rs:229`).

### Step 3 — `run.rs`: revert limbo, restore linear boot

**`src/bootstrap/run.rs`** — replace the limbo branch (lines 62-130) with linear flow:

```rust
let active_game_id = super::init_game::resolve_game_id(
    &db_pool,
    &world_with_map.world_card,
    &args.persona,
    "",  // persona_name resolved below from storage lookup
)?;
```

Wait — `resolve_game_id` needs `persona_name` for denormalization. But `run.rs` doesn't have it until it looks up the persona. Two options:

- **A)** `resolve_game_id` looks up persona_name via `storage.get_persona(persona_key)` before INSERT. But `resolve_game_id` only has `db_pool`, not a `Storage` (it uses raw SQL).
- **B)** `run.rs` resolves persona_name before calling `resolve_game_id`, passes it in.

Go with **B** — `run.rs` already has `lookup_storage`:

```rust
let world_id = world_with_map.world_id;
let world_card = world_with_map.world_card.clone();
let map = world_with_map.map;

// Resolve persona for auto-create and runtime
let player = lookup_storage
    .get_persona(&args.persona)?
    .ok_or_else(|| {
        crate::error::EngineError::Config(format!("Persona '{}' not found", args.persona))
    })?;
let persona_name = &player.sheet.name;

let active_game_id = super::init_game::resolve_game_id(
    &db_pool,
    &world_with_map.world_card,
    &args.persona,
    persona_name,
)?;
let storage = Arc::new(crate::storage::Storage::new_sqlite(
    db_pool.clone(),
    active_game_id,
));

let npcs = lookup_storage.list_characters(world_id)?;
// ... rest of boot continues linearly (no limbo branch) ...
```

Delete the entire `if active_game_id == 0 { ... return ... }` block. Delete the `let game = storage.get_game(active_game_id)?...` block (persona already resolved above). The rest of run.rs (load_game_state, spawn_arrival, settings, server) stays.

### Step 4 — Simplify v13 migration

**`src/storage/db.rs`** — replace the `if version < 13` block with schema-only changes (no backfill):

```rust
if version < 13 {
    let exec = |sql: &str| {
        conn.execute(sql, [])
            .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
    };

    // Add persona columns to games (idempotent guard).
    let has_persona_key: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('games') WHERE name='persona_key'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if has_persona_key == 0 {
        exec("ALTER TABLE games ADD COLUMN persona_key TEXT NOT NULL DEFAULT ''")?;
    }
    let has_persona_name: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('games') WHERE name='persona_name'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if has_persona_name == 0 {
        exec("ALTER TABLE games ADD COLUMN persona_name TEXT NOT NULL DEFAULT ''")?;
    }

    // Drop world.player_key if it still exists (upgrading DBs).
    let has_player_key: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('worlds') WHERE name='player_key'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if has_player_key > 0 {
        exec("ALTER TABLE worlds DROP COLUMN player_key")?;
    }

    conn.pragma_update(None, "user_version", 13).map_err(|e| {
        crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
    })?;
}
```

No `DELETE FROM games`. No backfill UPDATE. Fresh DBs never hit this path meaningfully (v9/v10 create tables without `player_key`, v13 adds persona columns with DEFAULT ''). Existing DBs get schema changes only; existing game rows get empty `persona_key`/`persona_name` — `resolve_game_id` auto-creates a new game with the CLI persona if no game matches the world.

### Step 5 — Fix 4 HTTP tests: add `persona_key` to form bodies

**`tests/http/fragment.rs`** — 4 tests submit `POST /games` without `persona_key`. Add `&persona_key=<value>` to each form body. Each test already seeds a persona via `storage.seed_persona(&player.key, &player)`, so use `&player.key`:

- **`test_create_game_handler` (line ~478)**: change `Body::from(format!("world_key={}", world.key))` → `Body::from(format!("world_key={}&persona_key={}", world.key, player.key))`
- **`test_create_game_with_world_key` (line ~862)**: same pattern — `format!("world_key={}&persona_key={}", world_a.key, player.key)`
- **`test_create_game_with_invalid_world_key` (line ~901)**: this test expects an error for nonexistent world. Add `&persona_key={}, player.key` — the world lookup fails first, persona_key irrelevant but required for form parsing.
- **`test_list_games_fragment_populated` (line ~367)**: two `POST /games` calls. Add `&persona_key=test_player` (or `&player.key` if in scope — check if this test has a `player` variable; if not, use `"test_player"`).

Each edit is a single-line change to the `Body::from(...)` string. Verify by reading the test function context before editing.

### Step 6 — Browser test harness: add `persona` param

**`tests/test_utils/server.rs`** — `start_server_with_env` gains `persona: &str` param. Thread into CLI args:

```rust
pub fn start_server_with_env(
    port: u16,
    world: &str,
    persona: &str,
    use_mock: bool,
) -> (Child, Option<std::path::PathBuf>, std::path::PathBuf) {
```

In both binary and cargo-run paths, add `--persona` arg:

```rust
c.args(["--world", world, "--persona", persona, "--port", &port.to_string()]);
```

Update `TestServer::start`, `TestServer::new`, `TestServer::new_with_mock`, `TestServer::with_config`, `TestServer::from_config` to accept and pass `persona`.

**`tests/test_utils/browser.rs`** — `with_test_page` gains `persona: &str` param, passes to `TestServer::new_with_mock(port, world, persona)`.

**`tests/browser/*.rs`** — each file has `const TEST_WORLD: &str = "test";`. Add `const TEST_PERSONA: &str = "test_player";` alongside. Update all `with_test_page(CONFIG_PATH, TEST_WORLD, ...)` calls to `with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, ...)`.

Files: `editing.rs`, `interaction.rs`, `structure.rs`, `trigger.rs`.

### Step 7 — Update ADR-026

**`docs/adr/adr-026-persona-relocation-to-game.md`**:

- **Section 4 (Bootstrap migration v13)**: remove "Wipe existing `games` rows" bullet. Replace with: "Schema-only: add `persona_key`/`persona_name` columns to `games`, drop `player_key` from `worlds`. No data backfill — fresh DBs start clean; existing DBs get empty persona fields on game rows, resolved on next boot via `--persona` CLI flag."
- **Add Section 8: CLI `--persona` flag**: "The CLI gains `--persona <key>` (default `julian`), mirroring `--world`. When `resolve_game_id` finds no existing game for the requested world, it auto-creates one using the CLI-provided persona. This restores the pre-ADR-026 auto-create behavior with explicit persona selection at startup. The Games-tab New Game form remains the primary creation path; the CLI flag is for headless/first-boot convenience."
- **Consequences > Negative**: remove "Migration is destructive" bullet. Replace with: "Migration is schema-only — existing game rows get empty `persona_key`/`persona_name`; a new game is auto-created on next boot with the CLI persona."

### Step 8 — Update CONTEXT.md

**`CONTEXT.md`** — update "Non-terms" section:

Replace:

```
- **"default persona"** — not a concept in the engine. There is no world-level "default" persona and no fallback resolution. When the player creates a game, they choose a persona explicitly; the form does not submit without one.
```

With:

```
- **"default persona"** — not a runtime concept. The engine has no world-level default persona and no runtime fallback resolution. A CLI `--persona <key>` flag (default `julian`) provides the initial persona for auto-creating a game on first boot, mirroring `--world`. Within the UI, the Games-tab New Game form requires an explicit persona pick.
```

### Step 9 — Update CHANGELOG

**`docs/CHANGELOG.md`** — fix the 2026-06-23 entry:

- Remove "wipes existing `games` rows" from migration bullet. Replace with "schema-only: adds persona columns, drops `worlds.player_key`. No data backfill."
- Add to "Bootstrap startup" bullet: "`resolve_game_id` auto-creates a game using the `--persona` CLI flag (default `julian`) when none exists — restores pre-ADR-026 auto-create behavior with explicit persona selection."
- Add `--persona` to CLI args list.

### Step 10 — Unit test for restored auto-create path

**`src/bootstrap/run_tests.rs`** — add `resolve_game_id_auto_creates_with_persona` test:

- Build empty in-memory DB, a `WorldCard` for `redmist_estate`, persona_key=`"julian"`, persona_name=`"Julian"`.
- Call `resolve_game_id(&db_pool, &world, "julian", "Julian")`.
- Assert: returns `Ok(id)` with `id > 0`.
- Query the `games` row: assert `world_key == "redmist_estate"`, `persona_key == "julian"`, `persona_name == "Julian"`.
- Call `resolve_game_id` again with same args; assert it returns the same `id` (idempotent — does not create a duplicate).
- Query `SELECT COUNT(*) FROM games`; assert `1`.

### Step 11 — Consolidate integration test storage helpers (Option D)

Scope expansion beyond original ADR-026 plan. Pre-ADR-026, migration v9 auto-INSERTed `games id=1` silently. Every test using `Storage::new_sqlite(DbPool::new(":memory:).unwrap(), 1)` got FK satisfaction for free. Removing the v9 INSERT (Step 4) broke ~10 sites across `snapshot_storage.rs`, `pipeline/retry.rs`, `flow/retry_main.rs`, `llm_message_storage.rs` — they relied on the invisible game row to satisfy `game_state_snapshots.game_id` and `messages.game_id` FK constraints.

Rather than surgically fixing each site, consolidate the duplicated test helpers into one canonical location.

**`tests/helpers/fixtures.rs`** (already exposed to integration tests via `tests/integration/mod.rs:5`) — add:

- `create_test_world_with_scenario()` — promoted from `lifecycle.rs` (has `StartingScenario` block; the no-scenario variant is already in fixtures).
- `create_basic_test_state()` — promoted; uses `create_test_world_with_scenario()`. Most integration tests want the scenario variant.
- `create_basic_test_state_no_scenario()` — promoted from `application_service.rs` variant; uses `create_test_world()`.
- `seed_test_world(storage: &Storage)` — promoted (identical copies in `application_service.rs` and `lifecycle.rs`).
- `make_test_ctx(storage: Arc<Storage>, state: GameState) -> GameServiceContext` — promoted (functionally identical copies in `application_service.rs` and `lifecycle.rs`, only differs by import style).
- `create_test_storage(game_id: u64) -> Storage` — new. Returns sqlite `Storage` with the games row pre-seeded so FK constraints pass.
- `create_test_storage_arc(game_id: u64) -> Arc<Storage>` — new. Convenience wrapper.

**Delete per-file copies** in:

- `tests/integration/application_service.rs` — `create_test_world`, `create_test_map`, `create_test_player`, `create_basic_test_state`, `make_test_ctx`, `create_storage`, `seed_test_world`
- `tests/integration/lifecycle.rs` — same set plus `create_test_world_with_scenario`
- `tests/integration/storage/snapshot_storage.rs` — `create_storage`, `create_game_storage` (identical duplicates)
- `tests/integration/storage/llm_message_storage.rs` — `create_storage`

**Replace raw `Storage::new_sqlite(DbPool::new(":memory:).unwrap(), 1)` sites** with `create_test_storage_arc(1)` (or `_` variant for non-Arc) in:

- `tests/integration/pipeline/retry.rs` — 4 sites (lines 154, 240, 322, 405)
- `tests/integration/flow/retry_main.rs` — 1 site (line 382)
- `tests/integration/storage/snapshot_storage.rs` — all raw `Storage::new_sqlite` sites using id=1

**Preserve** existing correct patterns: `Storage::new_in_memory()` (preset_storage, world_storage, ctx preset_storage), explicit `Storage::new_sqlite(pool, n)` where a custom game_id is intentionally used, and lifecycle's `seed_test_world + create_game(ctx)` flow (which creates a real game via service).

Text drift risk: `application_service.rs` used `description="A testing kingdom"` and `scenario="Test"`; `lifecycle.rs` used `description="A small testing kingdom"` and `scenario="Test scenario"`. Verified via grep — no test asserts on these strings. Safe to unify on the lifecycle variant (more descriptive).

## Verification

1. **Build clean**: `cargo build`
2. **Clippy clean**: `cargo clippy --all-targets -- -D warnings`
3. **Format check**: `cargo fmt --check`
4. **Full pipeline**: `python build.py` (or `python build.py --cleanup` for a fresh start)
5. **Targeted HTTP test run**: `cargo nextest run -p chronicler_engine --test http fragment::test_create_game_handler fragment::test_create_game_with_world_key fragment::test_create_game_with_invalid_world_key fragment::test_list_games_fragment_populated`
6. **Browser tests**: all 22 browser tests in `editing`, `interaction`, `structure`, `trigger` modules pass
7. **End-to-end smoke**: `cargo run` → server starts, auto-creates game with `julian` persona for `redmist_estate` world, game tab shows active play
8. **Integration tests**: `cargo nextest run -p chronicler_engine --test integration` — all storage, lifecycle, application_service, pipeline, flow tests pass without FK constraint failures
