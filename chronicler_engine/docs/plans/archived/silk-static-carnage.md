# Plan: Eliminate Test Duplication in chronicler_engine

## Problem
Unit tests under `src/` massively duplicate fixture data, mock implementations, and builder logic. Integration tests correctly use `test_support/` — unit tests ignore it. Three files are worst offenders:

| File | Lines | Problem |
|------|-------|---------|
| `src/application/action_pipeline/retry_tests.rs` | 939 | 2 mock storage impls (~170 lines), duplicated `StoredTriggerContext` (7x), custom context builder |
| `src/bootstrap_tests.rs` | 686 | `WorldManifest`/`Room`/`CharacterSheet`/`NpcCard` blocks repeated 7–9x each |
| `src/server/prompt_presets_fragment/handlers_tests.rs` | 574 | Manual `AppState` built 3 ways, inline `FailingPromptPresetStorage`, `PromptPreset` structs ~15x |

Cross-file heatmap: `StoredTriggerContext {` ×21 in 9 files, `PromptPreset {` ×62 in 16 files, `CharacterSheet {` ×30+ in 20+ files.

Existing `src/test_support/` (`fixtures.rs`, `context.rs`, `test_app_builder.rs`, `in_memory_storage.rs`) already solves most of this — unit tests simply don't use it.

---

## Option 1: Conservative — Extract Worst Offenders Only

**Scope:** Add new helpers to `test_support/` and replace only the most duplicated inline patterns.

### Changes
1. **New file:** `src/test_support/failing_storage.rs`
   - Move `FailingSnapshotStorage` (from `retry_tests.rs`)
   - Move `FailingMessageStorage` (from `retry_tests.rs`)
   - Move `FailingPromptPresetStorage` (from `handlers_tests.rs`)
   - Export via `test_support/mod.rs`

2. **Extend `src/test_support/fixtures.rs`**
   - `TestStoredTriggerContext::standard()` → replaces 21 identical 8-field structs
   - `TestPromptPreset::system(id, name)`, `TestPromptPreset::custom(id, name)` → replaces 62 inline presets

3. **Replace in target files only**
   - `retry_tests.rs`: use `test_support::failing_storage::*`, use `TestStoredTriggerContext::standard()`
   - `handlers_tests.rs`: use `test_support::failing_storage::FailingPromptPresetStorage`, use `TestPromptPreset::*`

### Pros
- Minimal blast radius (~4 files touched)
- No API changes to `TestAppBuilder` or existing helpers
- Low risk of breaking existing tests
- ~400–500 lines removed quickly

### Cons
- Doesn't fix bootstrap fixture duplication
- Doesn't fix manual `AppState` / `GameServiceContext` construction
- Other test files still duplicate patterns

---

## Option 2: Moderate — Unify Fixture + Adopt TestAppBuilder (Recommended)

**Scope:** Everything in Option 1, plus extend `TestAppBuilder` for server unit tests and add bootstrap fixtures.

### Changes
1. **All of Option 1**

2. **Extend `TestAppBuilder`**
   - Add `prompt_preset_storage(mut self, Arc<dyn PromptPresetStorage>)` method
   - This lets `handlers_tests.rs` inject `FailingPromptPresetStorage` into a builder instead of manual `AppState { ... }`

3. **Replace manual `AppState` constructions**
   - `handlers_tests.rs`: replace 3 × `make_test_app_state_*` with `TestAppBuilder`
   - `src/server/fragments_tests.rs`: replace its own `make_test_app_state()` with `TestAppBuilder`

4. **Extend `src/test_support/fixtures.rs` for bootstrap**
   - `TestWorldManifest::minimal()` → replaces 9 identical `WorldManifest` blocks
   - `TestCharacterSheet::hero()` → replaces 7 identical `CharacterSheet` blocks in bootstrap
   - `TestRoom::plain(id)` already exists in `TestMap::room(id)` — audit if `bootstrap_tests` can use it

5. **Replace manual `GameServiceContext` construction**
   - `retry_tests.rs::make_empty_context()` → use `test_support::make_test_context_without_snapshot()` (nearly identical)

### Pros
- Standardizes on existing `test_support` patterns
- Fixes all three target files completely
- ~700–900 lines removed
- Sets precedent for future unit tests

### Cons
- Touches ~6–8 files
- `TestAppBuilder` changes need careful review (builder method additions are safe but must not break existing calls)
- `bootstrap_tests.rs` uses `WorldManifest` (not `WorldCard`) — new fixture is bootstrap-specific

---

## Option 3: Aggressive — Full Test Infrastructure Overhaul

**Scope:** Everything in Option 2, plus systematic audit and extraction across ALL test files.

### Changes
1. **All of Option 2**

2. **New file:** `src/test_support/pipeline_helpers.rs`
   - Move generic helpers from `retry_tests.rs`: `insert_message_with_swipe`, `add_input_and_save`, `add_narration_and_save`
   - Deduplicate against `tests/helpers/pipeline_helpers.rs` — unify or clearly separate integration vs unit helper scope

3. **Add `TestLlmCallResult` fixture**
   - `TestLlmCallResult::empty()`, `TestLlmCallResult::with_text(text)` → replaces 10+ inline 8-field `LlmCallResult` structs

4. **Systematic cross-file audit**
   - Search and replace ALL inline `CharacterSheet {` in unit tests with `TestPlayer::standard().sheet` or `TestNpc::named().sheet`
   - Search and replace ALL inline `PlayerCard {` / `NpcCard {` with `TestPlayer` / `TestNpc` fixtures
   - Search and replace ALL inline `Room {` with `TestMap::room()` or `TestMap::room_named()`

5. **Consider file splits**
   - `retry_tests.rs` at 939 lines: split by domain (retry_main_narration, retry_event_continuation, retry_last_response_impl) if logical boundaries exist

### Pros
- Maximum duplication elimination (~1200–1500 lines)
- Establishes enforceable conventions
- Test suite becomes significantly more maintainable

### Cons
- Large blast radius (15+ files)
- High risk of subtle test behavior changes if fixtures aren't perfectly equivalent
- Time-consuming to review and validate
- May be overkill — some duplication is acceptable if tests need slight variations

---

## Recommendation

**Option 2 (Moderate)** offers the best cost/benefit ratio:
- It fixes the three worst files completely
- It extends existing infrastructure rather than inventing new patterns
- It sets a clear precedent without a massive audit
- ~700–900 lines of real savings

If Option 2 succeeds and tests pass, a follow-up task can apply Option 3's systematic audit.

## Verification

After any option:
```bash
cd chronicler_engine && python build.py
```

All tests must pass. No behavioral changes — purely structural refactoring.
