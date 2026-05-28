# Implementation Plan: Eliminate Test Duplication (Revised)

## Overview

Extract duplicated fixture data and builder logic from unit tests and route them through existing `src/test_support/` infrastructure. The codebase was recently refactored to a unified `Storage` struct, so the plan has been revised to match current architecture. No behavioral changes — purely structural refactoring.

## Architecture Context (Current State)

- **Storage:** Unified `Storage` struct with `Backend` enum (`InMemory`, `Sqlite`, `Test`). Failure injection via `Storage::with_test_failures()` — no custom mock structs needed.
- **AppState:** Uses `storage: Arc<Storage>` and `preset_storage: Arc<Storage>`.
- **GameServiceContext:** Uses `storage: Arc<Storage>` and `preset_storage: Arc<Storage>`.
- **TestAppBuilder:** Already exists, returns `Router`. Used by integration tests; unit tests ignore it.

## Current Duplication Heatmap

| Pattern | Count | Files |
|---------|-------|-------|
| `StoredTriggerContext {` | 15 | `retry_tests.rs` (6), `action_processing_tests.rs` (6), `pipeline_tests.rs` (2), `actions_tests.rs` (1) |
| `PromptPreset {` | ~55 | `handlers_tests.rs` (20), `fragments_tests.rs` (11), others scattered |
| `WorldManifest {` | 14 | `bootstrap_tests.rs` (9), others |
| `CharacterSheet {` | ~27 | `bootstrap_tests.rs` (9), others |
| `Room {` | ~20 | `bootstrap_tests.rs` (5), others |
| Manual `AppState` helpers | 3 | `handlers_tests.rs` (2), `fragments_tests.rs` (1) |
| Manual `GameServiceContext` | 4 | `retry_tests.rs` (4) |
| `GameStateSnapshot::from_game_state` | 21 | `retry_tests.rs` only |

---

## Option 1: Focused — Fix the Three Worst Files Only

**Scope:** Extract fixtures for `StoredTriggerContext`, `PromptPreset`, and bootstrap structs. Replace only in the three worst files.

**Tasks:**
1. Add `TestStoredTriggerContext` to `test_support/fixtures.rs`
2. Add `TestPromptPreset` to `test_support/fixtures.rs`
3. Add `TestWorldManifest` and `TestCharacterSheet` to `test_support/fixtures.rs`
4. Replace inline `StoredTriggerContext` in `retry_tests.rs`
5. Replace inline `PromptPreset` in `handlers_tests.rs`
6. Replace inline fixtures in `bootstrap_tests.rs`

**Pros:** Minimal blast radius (~4 files). ~400–500 lines removed quickly. Low risk.
**Cons:** Doesn't fix `fragments_tests.rs`, `pipeline_tests.rs`, `actions_tests.rs`, `action_processing_tests.rs`, or the `AppState`/`GameServiceContext` manual construction.

---

## Option 2: Moderate — Full Cross-File Cleanup (Recommended)

**Scope:** Everything in Option 1, plus extend `TestAppBuilder` for unit-test use and clean up all remaining `StoredTriggerContext` occurrences.

**Tasks:**
1. Add `TestStoredTriggerContext` fixture
2. Add `TestPromptPreset` fixture
3. Add bootstrap fixtures (`TestWorldManifest`, `TestCharacterSheet`)
4. Add `TestAppBuilder::build_app_state()` (extract `AppState` without building `Router`)
5. Replace `StoredTriggerContext` in all 5 files
6. Replace `PromptPreset` in `handlers_tests.rs` and `fragments_tests.rs`
7. Replace manual `AppState` in `handlers_tests.rs` and `fragments_tests.rs` with `TestAppBuilder`
8. Replace manual `GameServiceContext` in `retry_tests.rs` with `make_test_context_without_snapshot()`
9. Replace bootstrap fixtures in `bootstrap_tests.rs`
10. Add `save_snapshot helper` for `retry_tests.rs` to deduplicate `GameStateSnapshot::from_game_state`

**Pros:** Fixes all major duplication. ~600–800 lines removed. Sets clear precedent.
**Cons:** Touches ~8 files. Some `AppState` tests may need `build_app_state()` addition.

---

## Option 3: Aggressive — Systematic Audit + `Storage::new_in_memory()` Deduplication

**Scope:** Everything in Option 2, plus replace the ~52 `Storage::new_in_memory()` calls across 11 files with a shared helper.

**Tasks:** All of Option 2, plus:
11. Add `test_storage()` helper to `test_support`
12. Replace `Storage::new_in_memory()` in all test files

**Pros:** Maximum consolidation (~900–1100 lines). Very consistent.
**Cons:** Large blast radius. Many files touched. Risk of subtle behavior changes if tests rely on fresh `Storage` instances.

---

## Recommendation

**Option 2 (Moderate)** — the unified `Storage` architecture actually makes this *easier* than originally planned. No mock extraction needed. The main work is fixture helpers + inline replacement, which is safe and mechanical.

## Verification

```bash
cd chronicler_engine && python build.py
```

All tests must pass. No behavioral changes.
