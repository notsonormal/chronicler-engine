# Plan: Split `src/storage/backend.rs` by Table Domain

## Problem

`src/storage/backend.rs` is 1,248 lines (1,159 non-blank). While it fits under the 2,000-line guardrail, it colocates seven unrelated table domains in one file:

| Section | Lines | Methods |
|---------|-------|---------|
| Types + constructors + helpers | 1–265 | `new_sqlite`, `new_in_memory`, `with_backend_mut`, etc. |
| Game methods | 269–416 | `list_games`, `create_game`, `delete_game`, `get_game` |
| Snapshot methods | 418–560 | `save_snapshot`, `load_latest_snapshot`, `load_snapshot_by_id` |
| Message methods | 561–811 | `insert_message`, `delete_message`, `load_message_rows`, etc. |
| Swipe methods | 812–963 | `insert_swipe`, `update_swipe_text`, `shift_swipe_indices`, etc. |
| Preset methods | 964–1097 | `list_presets`, `get_preset`, `save_preset`, `delete_preset` |
| LLM message methods | 1098–1196 | `save_llm_message`, `list_latest_llm_messages` |
| Free helpers | 1197–1248 | `parse_datetime`, `db_game_to_game`, `db_row_to_preset` |

This makes navigation, code review, and parallel editing harder than necessary. The file has no structural boundaries beyond comment dividers.

## Decision

**Convert `backend.rs` to a directory module and split methods into table-scoped submodule files.**

Rust permits multiple `impl Storage` blocks across modules. Each submodule will contain one `impl Storage` block for its table domain. The core type definitions and shared helpers stay in `backend/mod.rs`.

### Target Structure

```
src/storage/
  mod.rs                         (unchanged: `pub mod backend; pub use backend::*;`)
  backend/
    mod.rs                       — Storage struct, Backend enum, Operation enum, constructors, with_backend_mut
    games.rs                     — impl Storage { list_games, create_game, delete_game, get_game }
    snapshots.rs                 — impl Storage { save_snapshot, load_latest_snapshot, load_snapshot_by_id }
    messages.rs                  — impl Storage { insert_message, delete_message, load_message_rows, get_active_swipe_index, update_active_swipe, soft_delete_message, restore_soft_deleted, purge_soft_deleted }
    swipes.rs                    — impl Storage { insert_swipe, update_swipe_text, shift_swipe_indices, load_swipes_for_messages }
    presets.rs                   — impl Storage { list_presets, get_preset, save_preset, delete_preset }
    llm_messages.rs              — impl Storage { save_llm_message, list_latest_llm_messages }
    helpers.rs                   — parse_datetime, db_game_to_game, db_row_to_preset, from_db
  backend_tests.rs               — stays in src/storage/ (or optionally moved to backend/tests.rs)
```

### Why this approach

- **Aligns with ADR-019/020 intent.** Each file maps 1:1 to a database table. Reasoning about what a file modifies is trivial.
- **Zero runtime cost.** Still a single `Storage` struct with inherent methods — no traits, no indirection.
- **Idiomatic Rust.** Multiple `impl` blocks for the same type across modules is standard practice (e.g. `std::vec::Vec` has `impl` blocks in multiple files inside `alloc/src/vec/`).
- **Preserves test coverage.** No logic changes — pure file movement. All 894 tests continue to pass without modification.

## What stays in `backend/mod.rs`

- `use` statements
- `pub struct Storage { ... }`
- `struct InMemoryData { ... }`
- `enum Backend { ... }`
- `pub enum Operation { ... }`
- `pub struct TestOverride`, `impl TestOverride`, `pub struct TestFailureHandle`
- Constructors: `new_sqlite`, `new_in_memory`, `with_failure`, `with_shared_overrides`, `with_test_failures`
- Private helpers: `with_backend_mut`, `game_id`
- `impl Storage` for `set_game_id`, `current_game_id`

## What moves

| File | Content | Approx lines |
|------|---------|-------------|
| `games.rs` | Game CRUD methods + `db_game_to_game` | ~150 |
| `snapshots.rs` | Snapshot methods + `parse_datetime` | ~145 |
| `messages.rs` | Message methods | ~255 |
| `swipes.rs` | Swipe methods | ~155 |
| `presets.rs` | Preset methods + `db_row_to_preset` + `from_db` | ~135 |
| `llm_messages.rs` | LLM message methods | ~100 |

## Optional cleanups (do only if approved)

Two small cleanups were flagged in code review. They can ride along with this refactor or be deferred.

### 1. Rename `from_db` → `db_preset_to_preset`
The helper at line 1233 is named `from_db`, which is inconsistent with `db_game_to_game` and `db_row_to_preset`. Rename to `db_preset_to_preset` for naming uniformity. Zero risk — private function, one call site.

### 2. `Mutex<Option<Backend>>` to eliminate dummy allocation in `add_failure`
`add_failure` (line 187) uses `mem::replace` with a fully-constructed dummy `Backend::InMemory(Box::new(InMemoryData { ... }))` just to extract the current backend. Changing `backend: Mutex<Backend>` to `backend: Mutex<Option<Backend>>` lets `add_failure` use `backend.take().unwrap()` instead — no dummy allocation, no `std::mem` import. The `unwrap()` is always `Some` in practice (the option is only `None` for the brief moment inside `add_failure`).

**Trade-off:** Every `backend.lock()` access gains `.unwrap()` or `?` on the `Option`. In practice this is a single `.unwrap()` inside `with_backend_mut`, so the noise is contained to one helper. The dummy allocation is eliminated.

## Files to update

1. **Create directory** `src/storage/backend/`
2. **Move** `src/storage/backend.rs` → `src/storage/backend/mod.rs`
3. **Create** `src/storage/backend/{games,snapshots,messages,swipes,presets,llm_messages,helpers}.rs`
4. **Update** `src/storage/mod.rs` — no changes needed (`pub mod backend;` works for both file and directory modules)
5. **Update** `docs/reference/data_layer.md` — change `src/storage/backend.rs` → `src/storage/backend/mod.rs`
6. **Update** `docs/CHANGELOG.md` — same path update
7. **Update** `docs/adr/adr-020-storage-consolidation.md` — same path update

## Verification

- `cargo check` passes
- `cargo clippy --all-targets -- -D warnings` passes
- `cargo test` passes (0 failures)
- `python build.py` passes
- No file exceeds 2,000 non-blank lines (guardrail)

## Risks

| Risk | Mitigation |
|------|------------|
| Git history loss for `backend.rs` | Use `git mv` to preserve blame/history on the rename |
| Import resolution breakage | Each submodule uses `use super::*;` to access `Storage`, `Backend`, `EngineError`, etc. from `mod.rs` |
| Doc anchor links break | Only three `.md` files reference `backend.rs`; all are updated in the plan |
