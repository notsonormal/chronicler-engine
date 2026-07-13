# Storage `require_*` Helpers

**Parent Plan:** none (standalone, orthogonal to error-model work in `t1-error-model-unification.md`)
**Status:** In Progress (canonical-all scope approved 2026-07-13)
**Date:** 2026-07-12 (scope revision 2026-07-13)
**Depends on:** none
**Blocks:** none
**Priority:** P3 (cosmetic / readability)

---

## Summary

`Storage::get_game`, `get_world`, `get_persona` return `Result<Option<T>, EngineError>`. The `Option` return is correct for the storage layer: catalogue, fallback, existence, and validation flows legitimately observe absence. But required-read callers — the action pipeline, generation/persistence gates, query handlers, arrival, and bootstrap — embed `.ok_or_else(|| EngineError::<Variant>(...))?` after each `get_*`. Worse, the variant choice is inconsistent: missing-game becomes `EngineError::Internal(...)` in some sites and `EngineError::Config(format!("... not found"))` in others; missing-persona becomes `EngineError::NpcNotFound` (semantically wrong — Persona ≠ Character) in pipeline/retry and `EngineError::Config` elsewhere; missing-world is `WorldNotFound` in pipeline/retry but `Config` in both gates and bootstrap fallback.

Add canonical `Storage::require_game`, `require_world`, `require_persona` helpers that fold `Ok(None)` into canonical typed variants, leave `get_*` unchanged for optional callers, and delete the now-unused `EngineError::NpcNotFound`.

Intentional behavior change:

- Missing game → `EngineError::GameNotFound(u64)` / display `"Game not found: {id}"`
- Missing world → existing `EngineError::WorldNotFound(String)` / `"World not found: {key}"`
- Missing persona → new `EngineError::PersonaNotFound(String)` / `"Persona not found: {key}"`

Backend `Err` values propagate unchanged through `?`.

## Scope (Canonical-All)

Required-read callers migrated:

- `application/action_pipeline/pipeline.rs::run_from_input`
- `application/action_pipeline/retry.rs::retry_event_continuation`
- `application/generation_gate/gate.rs::start_action`
- `application/persistence_gate/gate.rs::fetch_world_data_for_fresh_state`
- `application/query_handlers.rs::get_current_room_view` / `get_npc_headshots`
- `application/arrival_service.rs::run`
- `bootstrap/run.rs` (inner fallback-world + persona; outer requested world keeps `get_world` for fallback)

`get_*` preserved:

- `game_catalogue/gate.rs` validation paths (stay `ApplicationError::Validation`)
- `get_current_game_name` `"Unknown"` fallback
- Bootstrap primary requested-world lookup (None triggers fallback)
- Seed and existence checks
- World catalogue
- Catalogue/fallback/existence/validation flows throughout

Out of scope:

- `require_snapshot`, `require_preset`, `require_character` — absence semantics differ or require side effects (`retry.rs::save_retry_error` + early return; `EngineError::Internal`; validation paths).
- T1 pipeline error-model unification.
- Recreating deleted `fetch_world_bundle` / `fetch_world_bundle_for_retry` helpers (T13 explicitly removed them; locality preferred over shared bundle).
- New guardrail banning `get_*` (would produce false positives).

## Interface

```rust
impl Storage {
    pub fn require_game(&self, id: u64) -> Result<Game, EngineError> {
        self.get_game(id)?.ok_or_else(|| EngineError::GameNotFound(id))
    }

    pub fn require_world(&self, key: &str) -> Result<WorldWithMap, EngineError> {
        self.get_world(key)?
            .ok_or_else(|| EngineError::WorldNotFound(key.to_string()))
    }

    pub fn require_persona(&self, key: &str) -> Result<PersonaCard, EngineError> {
        self.get_persona(key)?
            .ok_or_else(|| EngineError::PersonaNotFound(key.to_string()))
    }
}
```

`GameNotFound(u64)` is intentional type asymmetry: game IDs are numeric throughout the storage interface.

## Migration

- `pipeline.rs`: replace three inline `get_* + ok_or_else` mappings with `require_*`; remove `internal_error` import if no other call site uses it.
- `retry.rs`: same three replacements.
- `generation_gate/gate.rs::start_action`: `get_game`/`get_persona` → `require_*`.
- `persistence_gate/gate.rs::fetch_world_data_for_fresh_state`: `get_game`/`get_world`/`get_persona` → `require_*`. Helper stays; absence semantics become canonical.
- `query_handlers.rs`: `?` converts canonical `EngineError` into `ApplicationError` via existing `From` impl.
- `arrival_service.rs`: collapse each three-arm `match self.app.storage().get_*(...)` into two-arm `match require_*(...)` with a single merged log message covering absence + backend error. Fire-and-forget `fn run(self) -> ()` contract preserved; both `Ok(None)` and `Err(e)` already produced the same control flow.
- `bootstrap/run.rs`: outer requested-world lookup keeps `get_world` (None triggers fallback). Inner fallback-world → `require_world`. Persona lookup → `require_persona`.

## Error Variant Changes

`src/error.rs`:

```rust
#[error("Game not found: {0}")]
GameNotFound(u64),

#[error("Persona not found: {0}")]
PersonaNotFound(String),
```

Retain existing `WorldNotFound(String)`.

Delete `NpcNotFound(String)`. Zero production constructors remain after `require_persona` migration.

## Decisions Locked

- Decision A: add `PersonaNotFound` variant (not overload `NpcNotFound`).
- Decision: delete `NpcNotFound` entirely once migration lands.
- Decision: do not recreate world-bundle helper (T13 explicitly removed it).
- Decision: arrival merges absence + backend-error log paths (accepted).
- Decision: no new guardrail on `get_*`.
- Decision: subagent dispatch sequential across all phases to avoid `EngineError` import-surface conflicts.

## Verification

- New `require_game` / `require_world` / `require_persona` unit tests cover hit + exact typed-payload miss only (backend-error propagation covered by existing pipeline/retry integration tests).
- New display tests in `src/error_tests.rs` lock canonical strings.
- `query_handlers_tests.rs`: new missing-game typed-error assertions for both affected handlers.
- `tests/integration/bootstrap/run_branches.rs`: updated persona-bad-world test expects `PersonaNotFound`.
- New pipeline/retry regression tests lock canonical missing-row status messages.
- Existing injected-backend-error tests on `get_world` / `get_persona` continue to prove non-absence errors flow unchanged.
- Grep `NpcNotFound` across `chronicler_engine/src` and `chronicler_engine/docs` returns zero (only explicit historical CHANGELOG references to a prior plan phase are retained).
- `python build.py` green; `cargo clippy --all-targets -- -D warnings` clean; `graphify update .` run.

## Pre-Implementation Checklist

- [x] Persona variant decided (Decision A).
- [x] Caller set exhaustively enumerated via manual `rg` across `chronicler_engine/src`.
- [x] T1 sequence dependency checked (no collision — this plan does not touch `PipelineResult` shape).
- [x] Arrival log-merge semantics approved.
