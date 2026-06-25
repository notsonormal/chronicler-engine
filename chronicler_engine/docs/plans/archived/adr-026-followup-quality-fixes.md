# ADR-026 Follow-up: Thermo-Nuclear Review Quality Fixes

**Date:** 2026-06-24
**Status:** Approved
**Scope:** Address follow-up findings from thermo-nuclear code quality review of the ADR-026 (persona relocation) diff. Skip #6 (run.rs sequence reshuffle) — user explicitly deferred.

## Context

ADR-026 moved persona binding from `World` to `Game` (as `persona_key` + denormalized `persona_name`), dropped `WorldManifest.player_file`, and scans `data/personas/` directly. The implementation is functionally correct (build + clippy clean) but surfaced code-quality regressions identified in the thermo-nuclear review.

This plan addresses findings #1, #2, #3, #4, #5, #7, #8. Finding #9 is deferred — `create_game_handler` error-code splitting adds branching without clear value over the existing `app_err_to_response` downstream mapping. Finding #6 is explicitly skipped (user direction).

## Findings Addressed

| # | Finding | Action |
|---|---------|--------|
| 1 | Dead `ctx` fetch in `new_world_form_handler` | Delete the block; render form directly |
| 2 | Duplicated games INSERT across `resolve_game_id` + `Storage::create_game` | Extract `DbPool::insert_game` helper; both call sites reuse |
| 3 | `GamesPanelTemplate` leaks `PlayerCard` via `Vec<PlayerCard>` + `.sheet.name` reach-through | Add `PersonaRowView { key, name }`; handler maps |
| 4 | `run_migrations` bumped to `pub(crate)` solely for test-support use | Revert to private `fn` |
| 5 | v13 migration idempotency guards are dead weight on versioned DB | Simplify to 3 plain ALTERs + DROP + `pragma_update` |
| 7 | Two persona-key conventions (`test-player` hyphen vs `test_player` underscore) | Standardize on `test_player`; replace literals with `TEST_PERSONA` const |
| 8 | `seed_default_game_row` silently swallows errors via `let _ =` | Switch to `.expect()` (module already allows `expect_used`) |
| 9 | `create_game_handler` collapses distinct errors to 400 | **Deferred** — not a regression; downstream `app_err_to_response` already maps `ApplicationError::Engine` → 500 |
| 6 | `run.rs` unrelated sequence reshuffle | **Skipped per user direction** |

## Files to Modify

### Production code (3 files)

1. **`src/server/worlds_fragment/handlers.rs`** — delete dead `ctx` fetch block in `new_world_form_handler`
2. **`src/storage/db.rs`** — (a) revert `pub(crate) fn run_migrations` → `fn`; (b) simplify v13 migration block; (c) add `DbPool::insert_game(...)` helper
3. **`src/bootstrap/init_game.rs`** — `resolve_game_id` calls `DbPool::insert_game` instead of raw INSERT

### Template + handler (3 files)

1. **`src/server/games_fragment/template.rs`** — add `PersonaRowView { key, name }`; change `personas: Vec<PlayerCard>` → `Vec<PersonaRowView>`; drop `PlayerCard` import
2. **`src/server/games_fragment/handlers.rs`** — map `PlayerCard` → `PersonaRowView` when building `GamesPanelTemplate`; no `.sheet.name` in template
3. **`src/storage/backend/games.rs`** — `Storage::create_game` sqlite branch calls `DbPool::insert_game` instead of raw INSERT

### Test code (3 files)

1. **`src/test_support/test_app_builder.rs`** — change player key from `"test-player"` to `"test_player"`
2. **`tests/http/fragment.rs`** — replace 3 `"persona_key=test-player"` literals with `TEST_PERSONA` const (import from `test_utils`)
3. **`src/test_support/fixtures.rs`** — `seed_default_game_row`: replace `let _ =` with `.expect("seed_default_game_row")`

## Detailed Steps

### Step 1 — Delete dead ctx fetch (#1)

**`src/server/worlds_fragment/handlers.rs::new_world_form_handler`** — current code fetches `as_game_service_context()` and immediately discards with `let _ = ctx;`. The form render only needs `render_world_edit_form(None, None, &[])`. Delete the entire `let ctx = match ...` block.

Successfully deleting this block also removes the latent 500 bug: `as_game_service_context()` errors when no active game exists, which would prevent the "New World" form from rendering on a fresh DB.

**Verify:** `cargo build` clean; `cargo nextest run worlds_fragment` passes.

### Step 2 — Extract `DbPool::insert_game` (#2)

**`src/storage/db.rs`** — add a helper next to `DbPool::impl`:

```rust
impl DbPool {
    /// Insert a new games row and return the new rowid.
    /// Single source of truth for the `games` INSERT column list.
    pub fn insert_game(
        &self,
        world_name: &str,
        world_key: &str,
        persona_key: &str,
        persona_name: &str,
        name: &str,
    ) -> Result<u64, crate::error::EngineError> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            rusqlite::params![world_name, world_key, persona_key, persona_name, name, &now],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to create game: {e}")))?;
        Ok(conn.last_insert_rowid() as u64)
    }
}
```

Note: this requires `chrono` import in `db.rs` (was removed in the ADR-026 diff — re-add).

**`src/storage/backend/games.rs::Storage::create_game`** — sqlite branch becomes:

```rust
Backend::Sqlite { pool } => pool.insert_game(world_name, world_key, persona_key, persona_name, name),
```

InMemory branch untouched (test path doesn't use the DB pool).

**`src/bootstrap/init_game.rs::resolve_game_id`** — None arm becomes:

```rust
None => {
    let existing_names = list_game_names_for_world(db_pool, &world.key)?;
    let name = crate::model::game::generate_game_name(&world.name, &existing_names);
    let id = db_pool.insert_game(&world.name, &world.key, persona_key, persona_name, &name)?;
    tracing::info!("Created new game '{name}' (id={id}) with persona '{persona_key}'");
    Ok(id)
}
```

Drops the `now` + `conn.execute` + `last_insert_rowid` duplication. Single source of truth for the INSERT column list.

**Verify:** `cargo build`; `cargo nextest run -p chronicler_engine` passes (especially `games_tests`, `bootstrap::load_tests`, `bootstrap::run_tests`).

### Step 3 — Add `PersonaRowView` (#3)

**`src/server/games_fragment/template.rs`** — add a row-view struct mirroring `GameRowView`:

```rust
pub struct PersonaRowView {
    pub key: String,
    pub name: String,
}
```

Change `GamesPanelTemplate`:

```rust
pub struct GamesPanelTemplate {
    pub active_game: Option<GameRowView>,
    pub saved_games: Vec<GameRowView>,
    pub worlds: Vec<WorldCard>,
    pub personas: Vec<PersonaRowView>,  // was Vec<PlayerCard>
}
```

Template body changes from `{{ p.sheet.name }}` to `{{ p.name }}`:

```html
{% for p in personas %}
<option value="{{ p.key }}">{{ p.name }}</option>
{% endfor %}
```

Drop `use crate::model::character::PlayerCard;` import.

**`src/server/games_fragment/handlers.rs::list_games_fragment`** — map `PlayerCard → PersonaRowView`:

```rust
let personas: Vec<PersonaRowView> = state
    .application_service
    .list_personas(ctx.clone())
    .unwrap_or_else(|e| {
        tracing::warn!("Failed to load personas: {e}");
        Vec::new()
    })
    .into_iter()
    .map(|p| PersonaRowView { key: p.key, name: p.sheet.name })
    .collect();
```

Import `PersonaRowView` from template module.

**Verify:** `cargo nextest run games_fragment` passes; manual eyeball — template module no longer imports `PlayerCard`.

### Step 4 — Revert `run_migrations` visibility (#4)

**`src/storage/db.rs`** — change `pub(crate) fn run_migrations` back to `fn run_migrations`. Only caller is `DbPool::new` in the same module. The `pub(crate)` bump was added solely to support `seed_default_game_row` in test_support, but that function calls `pool.conn()`, not `run_migrations` directly — the visibility bump is unused.

**Verify:** `cargo build` clean (no "function is never used" warning — it's called by `DbPool::new`).

### Step 5 — Simplify v13 migration (#5)

**`src/storage/db.rs`** — v13 block currently ~35 lines of idempotency guards (`pragma_table_info` COUNT checks before each ALTER). On a versioned DB with `user_version < 13` gating, the columns are guaranteed absent; the guards only paper over a partial-v13 crash state that the trailing `pragma_update` prevents anyway.

Replace the entire v13 block with:

```rust
if version < 13 {
    let exec = |sql: &str| {
        conn.execute(sql, [])
            .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
    };

    exec("ALTER TABLE games ADD COLUMN persona_key TEXT NOT NULL DEFAULT ''")?;
    exec("ALTER TABLE games ADD COLUMN persona_name TEXT NOT NULL DEFAULT ''")?;
    exec("ALTER TABLE worlds DROP COLUMN player_key")?;

    conn.pragma_update(None, "user_version", 13).map_err(|e| {
        crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
    })?;
}
```

**Leave v12 block untouched** — out of scope for this plan; it predates ADR-026.

**Verify:** Fresh DB test (`DbPool::new(":memory:")`) succeeds; `cargo nextest run storage::db_tests` passes. No existing test covers upgrade from v12 → v13 (all tests use fresh DBs), so no regression coverage lost.

### Step 6 — Standardize test persona key (#7)

**`src/test_support/test_app_builder.rs:80`** — change `key: "test-player".to_string()` to `key: "test_player".to_string()`. Aligns with `TEST_PERSONA` const in `test_utils/mod.rs` (underscore) and `data/personas/test_player.json`.

**`tests/http/fragment.rs`** — 3 literal occurrences of `persona_key=test-player`:

- Line ~365: `test_list_games_fragment_populated` (1st request)
- Line ~376: `test_list_games_fragment_populated` (2nd request)
- Line ~909: `test_create_game_handler_invalid_world` (nonexistent world case)

Replace each with `persona_key={TEST_PERSONA}`. Import `TEST_PERSONA` from `crate::test_utils` (already exported via `pub use`).

**Verify:** `cargo nextest run http::fragment` passes.

### Step 7 — Fix `seed_default_game_row` error swallowing (#8)

**`src/test_support/fixtures.rs::seed_default_game_row`** — module already declares `#![allow(clippy::expect_used)]`. Replace:

```rust
let _ = conn.execute("INSERT INTO games ...", ...);
```

with:

```rust
conn.execute("INSERT INTO games ...", ...)
    .expect("seed_default_game_row: insert should succeed on fresh in-memory db");
```

Rationale: this function is test-only. A failure here is always a setup bug (wrong schema, missing migration, stale column list), never a runtime condition. Silent `let _ =` hides the actual cause behind a downstream FK error. Per the AGENTS.md bias toward surfacing confusion over hiding it, `.expect()` is clearer.

**Verify:** `cargo clippy --all-targets` clean; `cargo nextest run test_support` passes.

## What Stays Unchanged

- **`src/bootstrap/run.rs`** (finding #6) — sequence reshuffle stays as-is per user direction.
- **`src/server/games_fragment/handlers.rs::create_game_handler`** (finding #9) — error-code splitting deferred. Current collapse to 400 (was 500 pre-ADR-026) is not a regression; downstream `app_err_to_response` maps `ApplicationError::Engine → 500` already. Revisit if/when a UI consumer surfaces a need for distinct codes.
- **v12 migration block** — out of scope; predates ADR-026.
- **`GameRowView.persona_name` + badge span** — clean as implemented.
- **`Game.persona_key`/`persona_name` fields + `DbGame` columns** — clean.
- **CLI `--persona` flag + `resolve_game_id` auto-create** — clean.

## Verification Plan

1. `cd chronicler_engine && cargo fmt`
2. `cd chronicler_engine && cargo clippy --all-targets -- -D warnings`
3. `cd chronicler_engine && cargo nextest run` (or `cargo test`)
4. `python build.py` (full validation: fmt + clippy + tests + coverage)
5. Manual: eyeball `games_fragment/template.rs` — no `PlayerCard` import
6. Manual: eyeball `storage/db.rs` — `run_migrations` is `fn`, v13 block is 3 ALTERs + DROP + pragma_update
7. Manual: eyeball `worlds_fragment/handlers.rs::new_world_form_handler` — no ctx fetch

## Reuse

- `PersonaRowView` mirrors the existing `GameRowView`/`WorldRowView` pattern in the same module — no new convention introduced.
- `DbPool::insert_game` mirrors `DbPool::conn` as a small method on the pool — keeps the INSERT column list in one place.
- `TEST_PERSONA` const already exists in `tests/test_utils/mod.rs` — re-use, don't duplicate.
- `.expect()` pattern in test_support matches the existing `#![allow(clippy::expect_used)]` module-level allow.

## Risk

- **Low:** `DbPool::insert_game` extraction — behavioral equivalence is exact (column list + params identical). Risk: forgetting to re-add `chrono` import in `db.rs`.
- **Low:** `PersonaRowView` — pure type boundary, no behavior change. Risk: missing a `.sheet.name` literal in template (search confirms only one occurrence in `games_fragment/template.rs`).
- **Low:** v13 simplification — all tests use fresh DBs (`server.rs::cleanup_stale_db` wipes before each browser test; `build.py --cleanup` wipes target dir). No test covers v12 → v13 upgrade; no regression coverage lost.
- **None:** `new_world_form_handler` ctx deletion — ctx was discarded; removing dead code.
- **None:** `run_migrations` visibility revert — `pub(crate)` was unused.
- **None:** test persona key standardization — const already exists; just aligning the outlier.
- **None:** `seed_default_game_row` `.expect()` — module already allows it.

## Related

- ADR-026: Relocate Persona Binding from World to Game
- `chronicler_engine/docs/plans/archived/fix-boot-and-default-game.md` — prior plan whose impl is being cleaned up
- `chronicler_engine/CONTEXT.md` — domain terms (unchanged by this plan)
