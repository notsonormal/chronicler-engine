# Storage `require_*` Helpers

**Parent Plan:** none (standalone, orthogonal to error-model work in `t1-error-model-unification.md`)
**Status:** Planning — not started
**Date:** 2026-07-12
**Depends on:** none
**Blocks:** none
**Priority:** P3 (cosmetic / readability)
**Findings owned:** n/a

---

## Summary

`pipeline.rs::run_from_input` (lines 56-89) loads a bundle of `game + world + persona + npcs` from `Storage`. Each `get_*` call returns `Result<Option<T>, EngineError>`, forcing the call site to chain `.ok_or_else(|| EngineError::Variant(...))?` after every `?`. Three lookups, three `ok_or_else` blocks, three error-ctor expressions.

The `Option` return is correct for the storage layer (absence is a legitimate state for catalogue / existence checks — `get_world` is also called from `world_catalogue/gate.rs` and `application_service.rs`). The noise lives at the **call site** where absence is not legitimate: `run_from_input` requires all three rows to exist or the pipeline cannot start.

Add `require_*` convenience methods on `Storage` that fold `None → EngineError` internally, leaving `get_*` untouched for callers that need `Option` semantics.

## Current Shape (pipeline.rs:56-89)

```rust
let (world, map, persona, npcs) = match (|| {
    let game = app.storage().get_game(started_for)?.ok_or_else(|| {
        EngineError::Internal(internal_error(format!(
            "current_game_id {started_for} not found"
        )))
    })?;
    let world_with_map = app
        .storage()
        .get_world(&game.world_key)?
        .ok_or_else(|| EngineError::WorldNotFound(game.world_key.clone()))?;
    let persona_card = app
        .storage()
        .get_persona(&game.persona_key)?
        .ok_or_else(|| EngineError::NpcNotFound(game.persona_key.clone()))?;
    let npcs: HashMap<String, NpcCard> = app
        .storage()
        .list_characters(world_with_map.world_id)?
        .into_iter()
        .map(|n| (n.id.clone(), n))
        .collect();
    Ok::<_, EngineError>((
        Arc::new(world_with_map.world_card),
        Arc::new(world_with_map.map),
        Arc::new(persona_card),
        npcs,
    ))
})() {
    Ok(bundle) => bundle,
    Err(e) => {
        tracing::error!("run_from_input: {e}");
        state.narrative.input_buffer.status = GenerationStatus::Error(e.to_string());
        run.phase_finalize(&mut state);
        return Ok(());
    }
};
```

## Proposed Shape

### 1. Add `require_*` helpers in `backend/{games,worlds,personas}.rs`

```rust
// backend/games.rs
pub fn require_game(&self, id: u64) -> Result<Game, EngineError> {
    self.get_game(id)?
        .ok_or_else(|| {
            EngineError::Internal(internal_error(format!(
                "current_game_id {id} not found"
            )))
        })
}

// backend/worlds.rs
pub fn require_world(&self, key: &str) -> Result<WorldWithMap, EngineError> {
    self.get_world(key)?
        .ok_or_else(|| EngineError::WorldNotFound(key.to_string()))
}

// backend/personas.rs
pub fn require_persona(&self, key: &str) -> Result<PersonaCard, EngineError> {
    self.get_persona(key)?
        .ok_or_else(|| EngineError::NpcNotFound(key.to_string()))
}
```

### 2. Refactor `pipeline.rs::run_from_input` call site

```rust
let (world, map, persona, npcs) = match (|| {
    let game = app.storage().require_game(started_for)?;
    let world_with_map = app.storage().require_world(&game.world_key)?;
    let persona_card = app.storage().require_persona(&game.persona_key)?;
    let npcs: HashMap<String, NpcCard> = app
        .storage()
        .list_characters(world_with_map.world_id)?
        .into_iter()
        .map(|n| (n.id.clone(), n))
        .collect();
    Ok::<_, EngineError>((
        Arc::new(world_with_map.world_card),
        Arc::new(world_with_map.map),
        Arc::new(persona_card),
        npcs,
    ))
})() {
    Ok(bundle) => bundle,
    Err(e) => {
        tracing::error!("run_from_input: {e}");
        state.narrative.input_buffer.status = GenerationStatus::Error(e.to_string());
        run.phase_finalize(&mut state);
        return Ok(());
    }
};
```

Gone: 3 `ok_or_else(...)` blocks + their `.clone()` / `format!` noise. Each lookup = single `?`.

## Decisions to Lock

- **`EngineError::NpcNotFound` for missing persona is semantically wrong.** Existing code (pipeline.rs:72) already misuses this variant for persona. Options:
  - **(A)** Add `EngineError::PersonaNotFound(String)` variant; update `require_persona` + fix pipeline.rs:72 misuse.
  - **(B)** Keep status quo — `NpcNotFound` overloaded for both NPC and persona lookups.
  - **(C)** Rename `NpcNotFound` → `CharacterNotFound` (covers NPC + persona via one variant) — broader blast radius, touches all `NpcNotFound` call sites.
  - Recommend **(A)**: local fix, doesn't touch NPC call sites.

## Out of Scope

- Storage trait changes (no trait here — `Storage` is a concrete struct at `adapters/driven/storage/backend/`). The `get_* → Option` return shape stays for other callers.
- T1 error-model unification (`t1-error-model-unification.md`) — that plan reshapes `PipelineResult` / `GenerationStatus::Error` control flow. This plan only touches the bundle-load section. They compose cleanly: the IIFE wrapper around `require_*` calls can stay or be replaced when T1 lands.
- `list_characters` returns `Result<Vec<NpcCard>, EngineError>` already — no `Option` to fold, unchanged.

## Blast Radius

- 3 files in `adapters/driven/storage/backend/` (games.rs, worlds.rs, personas.rs) — add 3 small methods.
- 1 file: `application/action_pipeline/pipeline.rs` — one call site.
- (Optional) `src/error.rs` — add `PersonaNotFound` variant if Decision A chosen.
- No behaviour change. Error types and strings preserved verbatim.

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean.
- Grep for `.ok_or_else(|| EngineError::` in `pipeline.rs` — should drop to zero at the bundle-load site.
- Existing error-message strings must not change (some tests may assert on substring).
- If Decision A taken: grep `NpcNotFound` — pipeline.rs:72 misuse should be gone; NPC call sites unchanged.

## Pre-Implementation Checklist

- [ ] Decide A / B / C for `PersonaNotFound` variant before touching `error.rs`.
- [ ] Scan `query_handlers.rs:34` and `retry.rs:123` for similar `get_*.ok_or_else(...)?` chains — if 3+ sites share the same shape, `require_*` helpers pay off across the codebase, not just in pipeline.rs. (Preliminary grep shows likely candidates but not yet read.)
- [ ] Audit tests that assert on the exact "current_game_id {id} not found" / "world '{}' not found" / "persona '{}' not found" message strings — if any exist, keep messages byte-for-byte identical in `require_*`.
- [ ] Confirm this plan does not collide with `t1-error-model-unification.md` Phase 6.1 Issue 9 constraint (the IIFE bundle-load wrapper is downstream of T1's pipeline-error-shape work).
