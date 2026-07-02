# Plan: Phase 2 Thermonuclear Review Fixes

**Date:** 2026-06-30 (decisions locked 2026-07-01)
**Status:** Implemented 2026-07-02 — all 11 steps landed on `hexagon-phase2` (commits `618faf8` → `a45e0b9`). Build green, 1190 tests pass.
**Scope:** `chronicler_engine/`
**Branch target:** stay on current `hexagon-phase2` branch (commit fix-up commits directly)

## Locked decisions (post plan-review)

- **Fix 1:** Option B — delete `LlmCallResult::from_chat_result`; each provider constructs `LlmCallResult` directly in its `complete()` impl. `ChatCompletionResult` stays adapter-internal (`src/adapters/driven/llm/transport/request.rs`). Port file drops `use crate::adapters::driven::llm::transport::ChatCompletionResult;`. Original Option A plan rejected: would have moved `ChatCompletionResult` into port while Fix 12 drops its `system_prompt`/`user_prompt` echo fields → port would own adapter-shaped transport DTO with dead fields. Option B avoids that.
- **Fix 6:** Option A — `QuantifierAgent` holds `Arc<dyn LlmProvider>` directly, drops `Arc<LlmCallRecorder>`. `with_backend` deleted; callers migrate to construct `Arc<dyn LlmProvider>` and pass directly.
- **Fix 7:** Option B — delete dead `text_check_service` field on `DefaultApplicationService` (plus builder + accessor). Accept plan deviation from 2.3; `AppState.text_check_service` stays direct.
- **Fix 12:** Option A — drop `system_prompt` + `user_prompt` from `LlmCallResult`. Recorder builds `LlmMessage` from its own args + chat result fields.
- **Fix 13:** included in this plan (no defer).
- **Fix 14:** Path A, full scope. Move `GameStateSnapshot` to `src/domain/model/state/game_state_snapshot.rs`; update all ~40 usage sites to import from domain (NO re-export shim at adapter path — full retag). Close Storage leaks in `agents/registry.rs`, `agents/quantifier/agent.rs`, `message_editing.rs`.
- **Branch strategy:** stay on current `hexagon-phase2` branch.
- **Scope:** no split. Test plan (`phase2-tests-coverage-fixes.md`) deferred until these fixes land.

## Context

External review of branch `hexagon-phase2` (commits `4b018d3` → `0c87b12` → `923c91c`) returned **"Not approved"** with 14 findings across P0–P3 severities. Every claim in the review was independently verified against the codebase — all confirmed accurate (see Verification Log at end). Two additional structural concerns (test misalignment + missing unit tests for new files) are split into a separate plan: `phase2-tests-coverage-fixes.md`.

This plan covers code-level fixes only. The test-coverage plan covers the new-file unit tests + integration-test reorganization, and is deferred until this plan lands.

**Reviewer verdict recap:** "Not approved. P0 #1 (port file violates the invariant the ADR is built around), P0 #2 (silent Mock fallback in prod drops forensics), and P0 #3 (double-save through MockBackend) are blocking. P1 items are clear code-judo opportunities the plan explicitly asked the reviewer to push for. P2/P3 are cleanup that can land with the fixes."

**Plan-review (improve-ai-plan skill) completed 2026-07-01** — all open decisions locked, Failure modes section added, Implementation handoff section added. See locked decisions block at top.

## Related

- Prior plan: `docs/plans/hexagonal-reorganization-plan.md` (Phase 2 complete; 5 deviations recorded)
- ADR-027: `docs/adr/adr-027-hexagonal-architecture-migration.md` (contains the "exactly 3 files" claim that is false at merge — P3 #14)
- Sibling plan: `docs/plans/phase2-tests-coverage-fixes.md`
- ADR-012: forensics audit trail (silently dropped by P0 #2)
- ADR-018: application service layer

## Goals

Restore ADR-027's central invariant at the port layer, remove the silent forensics-dropping regression, eliminate the MockBackend double-save, land all P1 structural cleanups, and correct ADR-027's false claim about exemption count.

### Success criteria

- `grep -rn "use crate::adapters" src/application/ports/` → empty
- `grep -rn "get_llm_backend_for" src/ tests/` → empty (deleted)
- `grep -rn "struct NoopForensics" src/ tests/` → exactly 1 (in `test_support/` or `test_utils/`)
- `grep -rn "struct NoopForensics" src/application src/bootstrap` → empty (prod copies gone)
- `grep -rn "sanitize_llm_output" src/application/` → file lives under `src/application/` (not imported from `adapters/driven/llm/providers/sanitize`)
- `grep -n "unwrap_or_else.*MockBackend\|Fallback to mock" src/application/game_service.rs src/bootstrap/init_game.rs` → empty
- `LlmCallRecorder::complete` no longer routes sanitization through `crate::adapters::driven::llm::providers::sanitize::`
- `DefaultApplicationService::with_text_check_service` either has a caller OR is deleted
- `ActionPipeline::new` called from prod code via `GameService::pipeline(&self)` instead of inline `Arc::clone` triple
- `QuantifierAgent::with_backend` either moved to `test_support/` or replaced with `make_test_recorder` usage
- `server_impl.rs:23-25` no-op write-back lines deleted
- `read_lock_or_recover` defined exactly once across `src/`
- ADR-027 "exactly 3 files" claim corrected to match reality
- All tests green; `python build.py` green; clippy 0 warnings

---

## P0 — Must fix (blocking)

### Fix 1. Port file imports adapters + dead `get_llm_backend_for` (review P0 #1, P1 #8)

**Current state:**

`src/application/ports/llm_provider.rs`:
- Line 8: `use crate::adapters::driven::llm::transport::ChatCompletionResult;`
- Line 10: `use crate::adapters::driven::storage::Storage;`
- Lines 80-104: `pub fn get_llm_backend_for(...)` marked `#[deprecated(since = "0.2.1")]` — **zero callers** in src/ or tests/. Imports `LlmBackendType`, `Connection`, all 4 provider structs solely to support this dead function.

`src/application/llm_recorder.rs:35`:
```rust
let sanitized_text =
    crate::adapters::driven::llm::providers::sanitize::sanitize_llm_output(&result.text);
```
Application orchestrator directly imports an adapter module function. Plan 2.1c said postprocessing moves to recorder — worker left the function in the adapter and called into it.

**Invariant broken:** ADR-027 line "Core (`domain/`, `application/`) depends on port traits only" — port file itself imports 2 adapters. arch-lint won't catch it (Phase 1.7 deferred rules).

**Root causes (reviewer's framing):**
1. `LlmCallResult::from_chat_result(...)` takes `ChatCompletionResult` from `adapters/driven/llm/transport` — port return type shaped by adapter transport DTO.
2. `get_llm_backend_for` is a composition-root factory that matches on `LlmBackendType` and instantiates concrete provider structs. Plan 2.1d explicitly said this moves to `bootstrap/llm_factory.rs`. The new `get_llm_recorder_for` lives in bootstrap (good), but the old `get_llm_backend_for` was left behind.

**Fix:**

1. **Delete `get_llm_backend_for`** from `src/application/ports/llm_provider.rs` (lines 80-104 + surrounding `use` lines for `Storage`, `Connection`, `LlmBackendType`, the 4 provider types). Zero callers — safe delete.
2. **Move `sanitize_llm_output`** out of `src/adapters/driven/llm/providers/sanitize.rs` to `src/application/llm_sanitizer.rs` (or inline into `llm_recorder.rs`). It's pure regex postprocessing — no I/O, no adapter concern. Update `llm_recorder.rs:35` call site + any test imports. Stale module doc comment in `sanitize.rs` referencing `LlmBackend::postprocess_response_text` (trait was deleted) gets removed with the move.
3. **`ChatCompletionResult` resolution — Option B (locked):** Drop `LlmCallResult::from_chat_result` entirely. Each provider (`openrouter`, `ollama`, `deepseek`, `mock`) constructs `LlmCallResult` directly in its `complete()` impl using fields from `ChatCompletionResult` (which stays adapter-internal at `src/adapters/driven/llm/transport/request.rs`). `mock.rs::make_result` already uses a literal; the other 3 providers replace `LlmCallResult::from_chat_result(agent_name, name(), model(), chat)` with a `LlmCallResult { text: chat.text, raw_request_json: chat.raw_request_json, raw_response_json: chat.raw_response_json, backend_name: self.name().to_string(), model_name: self.model().to_string(), agent_name: agent_name.to_string() }` literal.

**Interaction with Fix 12:** `LlmCallResult` drops `system_prompt`/`user_prompt` in Fix 12. The literal constructed here must use post-Fix 12 field set. Implement Fix 1 provider-direct-construction + Fix 12 field-drop together (single atomic `LlmCallResult` reshaping pass) to avoid mid-step rebuilds.

**Implementation note:** Plan-review rejected original Option A (move `ChatCompletionResult` into port). Reason: Fix 12 leaves that struct with dead `system_prompt`/`user_prompt` echo fields, so port-ownership would introduce dead fields at the port layer — architecturally worse than the original adapter-internal state.

**Files to change:**
- `src/application/ports/llm_provider.rs` — delete `get_llm_backend_for`, remove adapter imports
- `src/application/llm_recorder.rs` — update sanitization call
- `src/adapters/driven/llm/providers/sanitize.rs` — delete (or move content)
- `src/adapters/driven/llm/transport/request.rs` — relocate `ChatCompletionResult` DTO per Option A/B (currently defined at line 17 of this file)
- `src/adapters/driven/llm/providers/{openrouter,ollama,deepseek,mock}.rs` — update `LlmCallResult` construction per Option A/B
- any tests importing `sanitize_llm_output` — update path

### Fix 2. Silent Mock fallback in prod drops ADR-012 forensics (review P0 #2)

**Current state:**

`src/application/game_service.rs:47`:
```rust
let llm_recorder = crate::bootstrap::llm_factory::get_llm_recorder_for(&connection, Arc::clone(&storage))
    .unwrap_or_else(|e| {
        tracing::error!("Failed to create LLM recorder: {e}");
        // Fallback to mock
        struct NoopForensics;
        impl LlmMessageRepository for NoopForensics { /* ... */ }
        Arc::new(LlmCallRecorder::new(
            Arc::new(MockBackend::new(Some(Arc::clone(&storage)))),
            Arc::new(NoopForensics),
        ))
    });
```

Same pattern at `src/bootstrap/init_game.rs:307-318`.

**Regression:** Pre-refactor `get_llm_backend_for` was infallible (`fn ... -> Box<dyn LlmProvider>`, no Result). Failures deferred to first LLM call where they'd actually error out loudly. Post-refactor `get_llm_recorder_for` returns `Result`, but callers `unwrap_or_else` into Mock + NoopForensics. User configures OpenRouter → factory fails (network, missing key, anything) → system silently runs on Mock with zero forensics saved. ADR-012 audit trail dropped silently.

**Fix:**

Make `get_llm_recorder_for` errors propagate. Two call sites:

1. **`GameService::with_storage` / `with_backends` / `new`** — these need to return `Result<Self, EngineError>`. Precedent: `QuantifierAgent::from_config_with_storage(...) -> Result<Self, EngineError>` (at `src/application/agents/quantifier/agent.rs`) already returns `Result` and propagates `get_llm_recorder_for` errors via `?` (see line `crate::bootstrap::llm_factory::get_llm_recorder_for(...)?`). Same pattern applies here.

   **Note:** `QuantifierAgent` has NO `new` constructor. Its constructors are `from_config`, `from_config_with_storage`, and `with_backend` (test-only). Don't reference a fictional `QuantifierAgent::new`.
2. **`ArrivalTaskContext::run` (init_game.rs)** — the actual signature is `fn run(self)` (sync, returns `()`) at line 150, wrapped by `pub fn run_sync(self)` at line 146 (also returns `()`). NOT async, NOT fallible. The `unwrap_or_else` fallback exists because the fire-and-forget path has no parent to bubble an error to. The caller `tests/integration/flow/arrival_persistence.rs:51` just calls `task_ctx.run_sync()` and discards the (unit) return.

   Replacing the silent Mock fallback requires picking one of two shapes:
   - **Option A (loud skip):** Keep `run(self) -> ()`. On `get_llm_recorder_for` failure, log `error!` and `return` early — skip narration entirely. NOT silent Mock, just loud skip. The arrival task fails to produce narration; downstream state is unchanged.
   - **Option B (propagate Result):** Change `run(self) -> Result<(), EngineError>` and `run_sync(self) -> Result<(), EngineError>`. Update `arrival_persistence.rs:51` test caller to handle the Result. If any production caller exists via `tokio::spawn`, log errors there.

   Plan should pick A — simpler, no signature cascade, and the arrival task is genuinely fire-and-forget so a loud skip matches the operational model. B is more rigorous but adds Result plumbing for a path that already silently logs.

   **Drop the `Mock` + `NoopForensics` fallback entirely from prod paths.** Move Mock fallback to a `#[cfg(test)]`-only feature flag if test paths genuinely need it.

   **Note:** `QuantifierAgent::from_config_with_storage` (NOT `QuantifierAgent::new` — that doesn't exist) already uses `?` propagation against `get_llm_recorder_for`. For `GameService::with_storage`, the same `?` pattern applies once the return type is `Result`. `GameService::new()` (line 23) also delegates to `with_storage` and would inherit the Result return — its 2 callers (`src/adapters/driving/http/mod_tests.rs:56` test + the `Default` impl at `game_service.rs:127`) must handle the new signature. Alternatively, `GameService::new()` can be deleted if its only caller is the `Default` impl and tests can use `with_storage` directly.

**Caller inventory (re-verified during plan review):** `GameService::with_storage` has 10 total callers — 2 prod (`server_impl.rs:38,45`) + 8 test (`tests/poison_recovery.rs:42,77`, `src/test_support/test_app_builder.rs:305`, `settings_fragment/handlers_tests.rs:22,45`, `prompt_presets_fragment/handlers_tests.rs:26,307`). All propagate via `?` (tests use `.expect(...)`). `tests/integration/flow/arrival_persistence.rs:51` calls `task_ctx.run_sync()` (unit return) — Option A loud-skip needs no caller change.

**Failure modes (per plan review):** Loud-skip (Option A) for `ArrivalTaskContext::run` returns before any `is_generating` mutation or state write (factory call at `init_game.rs:305` precedes snapshot load at `run()` line 150); no stale flag, no half-written storage. `server_impl.rs:38,45` propagation: broken LLM config halts bootstrap with logged `EngineError` (acceptable — silent degradation was the bug). Mock connection path stays infallible post-Fix 3 (`MockBackend::new()` cannot fail), so `arrival_persistence.rs:51` test stays green.

**Side benefit:** removes 2 of the 9 `NoopForensics` copies (Fix 4 below).

**Files to change:**
- `src/application/game_service.rs` — `new`/`with_storage` return `Result<Self, EngineError>`; delete inline `NoopForensics` struct (lines 47-65 approx)
- `src/bootstrap/init_game.rs` — propagate error from `get_llm_recorder_for` at line 306; delete inline `NoopForensics` (lines 307-318)
- Callers of `GameService::new` / `with_storage` — propagate or handle error
- `src/adapters/driving/http/server_impl.rs` — update `GameService::with_storage(...)` call site (lines 35-43 and similar)

### Fix 3. MockBackend double-save (review P0 #3)

**Current state:**

`src/adapters/driven/llm/providers/mock.rs:94-96`:
```rust
if let Some(storage) = &self.storage {
    let _ = storage.save_llm_message(&result.to_message());
}
```

MockBackend still holds `storage: Option<Arc<Storage>>` and self-saves. Then `LlmCallRecorder::complete` at `src/application/llm_recorder.rs:42` calls `self.forensics.save_llm_message(&message)` — every call through the recorder with a MockBackend writes 2 rows.

`src/bootstrap/llm_factory.rs:30` passes `Some(Arc::clone(&storage))` into `MockBackend::new(...)` — keeps the duplication path alive in prod wiring too.

**Plan context:** Plan 2.1b explicitly said providers "lose their `storage: Option<Arc<Storage>>` field." Phase 2 Deviation 1 (in `hexagonal-reorganization-plan.md`) granted MockBackend an exception to keep the field — for "test assertions on saved messages." Reviewer's read: that's wrong, the recorder owns that now, and existing `mock_tests::test_mock_backend_logs_to_storage` should migrate to go through the recorder.

**Fix:**

1. **Drop `storage` field from `MockBackend`.** Constructor: `MockBackend::new()` (no storage arg).
2. `src/bootstrap/llm_factory.rs:30` — pass `None` (or remove the arg entirely): `Arc::new(MockBackend::new())`.
3. **Migrate `test_mock_backend_logs_to_storage`** (in `src/adapters/driven/llm/providers/mock_tests.rs` or equivalent) — re-route through `LlmCallRecorder::complete` + assert on the `LlmMessageRepository` the recorder was built with (i.e., the real `Storage` the test owns).
4. Audit any other MockBackend callers asserting on `.storage` directly — port to recorder-based assertion.

**Files to change:**
- `src/adapters/driven/llm/providers/mock.rs` — drop `storage` field, drop self-save block (lines 93-96)
- `src/adapters/driven/llm/providers/mock_tests.rs` — migrate assertions
- `src/bootstrap/llm_factory.rs:30` — drop `Some(...)` from `MockBackend::new(...)`
- `src/application/agents/quantifier/agent.rs` — check ` QuantifierAgent` callers that build MockBackend
- Any test sites building `MockBackend::new(Some(storage))` — switch to `MockBackend::new()` + build recorder with the `Storage` they already hold

**Plan note:** Phase 2 Deviation 1 (MockBackend storage kept) is amended — this fix removes the deviation entirely. Update `hexagonal-reorganization-plan.md` Phase 2 deviations section when this lands.

---

## P1 — Structural regressions

### Fix 4. Extract `NoopForensics` to `test_support` (review P1 #4, P2 #10)

**Current state:** 9 `struct NoopForensics` copies across the codebase.

Prod (2 — both removed by Fix 2 above):
- `src/application/game_service.rs:52`
- `src/bootstrap/init_game.rs:315`

Tests (7):
- `src/application/action_pipeline/retry_tests.rs:24`
- `src/application/action_pipeline/pipeline_tests.rs:19`
- `src/application/action_pipeline/actions_tests.rs:19`
- `src/application/agents/quantifier/agent.rs:72` (inside `#[cfg(test)]`-ish block — check)
- `src/application/agents/quantifier/agent_tests.rs` — does it have one too? (verify on implementation)
- `tests/infrastructure/invariant_contract.rs:37`
- `tests/test_utils/mod.rs:29`
- `tests/integration/flow/retry_event.rs:25`

`tests/test_utils/mod.rs::make_test_recorder` already exists as the canonical helper — ignored by 6 other sites.

**Co-duplication with `make_test_recorder`:** Each of the 7 test sites defines its own local `make_test_recorder` function (in addition to the canonical `pub fn make_test_recorder` at `tests/test_utils/mod.rs:28`). The 6 local copies shadow/duplicate the canonical one. Dedup must treat `NoopForensics` + `make_test_recorder` as one combined refactor — they're co-located in every site:

- `src/application/action_pipeline/retry_tests.rs:23` — local `make_test_recorder` + local `NoopForensics` (line 24)
- `src/application/action_pipeline/pipeline_tests.rs:16` — local `make_test_recorder` + `NoopForensics` (line 19)
- `src/application/action_pipeline/actions_tests.rs:16` — local `make_test_recorder` + `NoopForensics` (line 19)
- `tests/integration/flow/arrival_persistence.rs:11` — local `make_test_recorder`
- `tests/integration/flow/retry_event.rs:24` — local `make_test_recorder` + `NoopForensics` (line 25)
- `tests/infrastructure/invariant_contract.rs:32` — local `make_test_recorder` + `NoopForensics` (line 37)
- `tests/test_utils/mod.rs:28` — canonical `pub fn make_test_recorder` + `NoopForensics` (line 29)

**`agent_tests.rs` does NOT define `NoopForensics` itself** — its 5 callers of `QuantifierAgent::with_backend` (at lines 27, 35, 43, 51, 62) reach the `NoopForensics` inside `agent.rs:72` via that method. Removing `with_backend` (Fix 6) eliminates site `agent.rs:72` entirely and forces migration of these 5 callers.

**Fix:**

1. Move `NoopForensics` struct + `impl LlmMessageRepository for NoopForensics` to `src/test_support/` (new file `src/test_support/noop_forensics.rs` or `mod.rs` addition). Keep `#[cfg(test)]`-gated so it can't leak into prod.
2. Promote `make_test_recorder` to `src/test_support/` as a `#[cfg(test)]` `pub(crate)` fn using the shared `NoopForensics`. Delete the 6 local copies. Have `tests/test_utils/mod.rs` re-export the canonical one (or delegate to it) so integration tests keep their existing import path.
3. Replace all 7 test sites: delete local `NoopForensics` + local `make_test_recorder`, import from `test_support` instead.
4. Remove `QuantifierAgent::with_backend` (Fix 6 below) — eliminates site `agent.rs:72` entirely. Migrate the 5 `agent_tests.rs` callers (see Fix 6).

**Files to change:**
- `src/test_support/noop_forensics.rs` (NEW) — shared `NoopForensics` + `make_test_recorder`
- `src/test_support/mod.rs` — add module declaration
- 4 `src/application/action_pipeline/*_tests.rs` — delete local helpers, import canonical
- `tests/integration/flow/arrival_persistence.rs` — delete local helper
- `tests/integration/flow/retry_event.rs` — delete local helper + struct
- `tests/infrastructure/invariant_contract.rs` — delete local helper + struct
- `tests/test_utils/mod.rs` — re-export or delegate to canonical

### Fix 5. Add `GameService::pipeline(&self) -> ActionPipeline` (review P1 #5)

**Current state:** 3 prod sites with inline `Arc::clone` triples:

- `src/application/action_pipeline/actions.rs:15-18`
- `src/application/action_pipeline/retry.rs:124-127`
- `src/application/action_pipeline/retry.rs:152-155`

Pattern:
```rust
let pipeline = ActionPipeline::new(
    Arc::clone(&service.prompt_assembler),
    Arc::clone(&service.llm_recorder),
    Arc::clone(&service.agent_registry),
);
```

Plus 7+ test sites with the same pattern.

**Fix:**

1. Add `GameService::pipeline(&self) -> ActionPipeline` — returned by value. `ActionPipeline` is a plain struct holding 3 `Arc` fields (`assembler`, `recorder`, `agents`); no lifetime parameter exists on the struct. Returning by value triggers 3 `Arc::clone`s internally — cheap. (Plan should NOT use `ActionPipeline<'_>` — that lifetime doesn't exist on the struct definition.)
2. Replace all 3 prod call sites + test sites to use `service.pipeline()`.
3. Removes Arc::clone noise + makes structural coupling explicit (callers don't need to know the 3 fields).

**Files to change:**
- `src/application/game_service.rs` — add `pipeline()` method
- `src/application/action_pipeline/actions.rs`
- `src/application/action_pipeline/retry.rs` (2 sites)
- `src/application/action_pipeline/pipeline_tests.rs` (7 sites)
- `tests/infrastructure/invariant_contract.rs:187`

### Fix 6. Move `QuantifierAgent::with_backend` to test support (review P1 #6)

**Current state:** `src/application/agents/quantifier/agent.rs:66` (starts at line 66):

```rust
pub fn with_backend(
    name: String,
    recorder_or_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
) -> Self {
    // For tests, create a recorder with the given provider and mock forensics
    use crate::application::ports::llm_message_repository::LlmMessageRepository;
    struct NoopForensics;
    impl LlmMessageRepository for NoopForensics { /* ... */ }
    let recorder = Arc::new(LlmCallRecorder::new(
        recorder_or_provider,
        Arc::new(NoopForensics),
    ));
    Self { /* ... */ }
}
```

10 test callers, zero prod callers (verified). Param name `recorder_or_provider` ambiguous (it's an `LlmProvider`). Constructed recorder over-scopes — QuantifierAgent holds full `Arc<LlmCallRecorder>` just to access `provider()`; Quantifier doesn't use forensics.

**Caller inventory (verified):**
- `src/application/agents/quantifier/agent_tests.rs` — 5 callers at lines 27, 35, 43, 51, 62 (inside local `test_with_backend` helper at line 24)
- `src/application/action_pipeline/pipeline_tests.rs` — 6 callers
- `src/application/action_pipeline/actions_tests.rs` — 3 callers

**Fix:**

Two clean options:
- **Option A (locked):** `QuantifierAgent` holds `Arc<dyn LlmProvider>` directly (not full recorder). Test (or factory) wraps in recorder if needed. Quantifier is pure consumer of `complete()` — doesn't need forensics.
- **Option B:** Move `with_backend` to `test_support/` as `QuantifierAgent::test_with_backend(...)`, keep prod `QuantifierAgent::from_config_with_storage` using `Arc<LlmCallRecorder>`.

Option A locked. Implement per Option A unless existing call sites prove hard to migrate (re-verify during implementation).

**Note:** `QuantifierAgent` has NO `new` constructor. Prod constructors are `from_config` and `from_config_with_storage`. Don't reference a fictional `QuantifierAgent::new`.

**Files to change:**
- `src/application/agents/quantifier/agent.rs` — rework `QuantifierAgent` field type per Option A, delete `with_backend`
- `src/application/game_service.rs` — `with_mock_quantifier` builds QuantifierAgent through new path
- `src/application/action_pipeline/pipeline_tests.rs` — 6 call sites migrated
- `src/application/action_pipeline/actions_tests.rs` — 3 call sites migrated
- `src/application/agents/quantifier/agent_tests.rs` — migrate 5 callers at lines 27, 35, 43, 51, 62 + delete local `test_with_backend` helper at line 24

### Fix 7. Wire or delete dead `text_check_service` field on `DefaultApplicationService` (review P1 #7)

**Current state:** `src/application/application_service.rs`:

- Line 104: `text_check_service: Option<TextCheckService>` field
- Lines 111: initialized `None` in constructor
- Lines 115-117: `with_text_check_service` builder (zero callers verified)
- Lines 124-125: `text_check_service(&self)` accessor (only the struct touches the field)

HTTP layer bypasses `ApplicationService` entirely — `AppState.text_check_service` is built directly in `server_impl.rs:29` via `create_text_check_service(&settings)`.

**Plan context:** Plan 2.3 wanted `check_player_input` routed via `ApplicationService` ("`app.text_check.check_player_input(...)` via ApplicationService"). Worker didn't do this.

**Fix — choose one:**

- **Option A (plan compliance):** Wire through `ApplicationService`. HTTP layer calls `app.application_service.text_check_service().check_player_input(...)` instead of `state.text_check_service().check_player_input(...)`. `ApplicationService::new` takes the `TextCheckService` (no `Option`), `with_text_check_service` builder goes away (becomes required arg in constructor). `AppState` drops its direct `text_check_service` field; routing goes through `ApplicationService`.
- **Option B (delete dead code):** Delete field + builder + accessor on `ApplicationService`. Accept that text-check lives on `AppState` directly. Document deviation from plan 2.3 in `hexagonal-reorganization-plan.md` Phase 2 deviations.

Plan picks B (locked during plan review) — simpler, less surface area, accepts the plan deviation from 2.3. Document deviation in `hexagonal-reorganization-plan.md` Phase 2 deviations when this lands.

**Files to change (Option B):**
- `src/application/application_service.rs` — delete field, builder, accessor, import
- (Option A only): `src/adapters/driving/http/app_state.rs`, `src/adapters/driving/http/server_impl.rs`, `src/adapters/driving/http/fragments/actions.rs:97`, `src/adapters/driving/http/fragments/misc/text_check.rs:38`

### Fix 8. Move `sanitize_llm_output` to `application/` (review P1 #8)

**Subsumed by Fix 1** above — same root issue. `sanitize_llm_output` lives in `src/adapters/driven/llm/providers/sanitize.rs`; `LlmCallRecorder::complete` at `llm_recorder.rs:35` imports it directly into the application layer.

Track under Fix 1 — single fix closes both P0 #1 (port imports adapters) reduction and P1 #8 (recorder imports adapter module).

### Fix 9. Delete server_impl no-op + dedupe `read_lock_or_recover` (review P1 #9)

**Current state:**

`src/adapters/driving/http/server_impl.rs:22-25`:
```rust
let settings = read_lock_or_recover(&resources.settings, "settings");
let text_check_service =
    Arc::new(crate::bootstrap::text_check_factory::create_text_check_service(&settings));
let drop_settings = resources.settings.clone();
let _ = drop_settings.write().map(|mut s| *s = settings);
```

Last 2 lines: read a clone, acquire write lock, write same value back. Pure no-op.

Plus `read_lock_or_recover` defined twice:
- `src/adapters/driving/http/app_state.rs:109`
- `src/adapters/driving/http/server_impl.rs:14`

**Fix:**

1. Delete `let drop_settings = ...; let _ = drop_settings.write()...` lines (no-op write-back).
2. Move `read_lock_or_recover` to a shared location — `src/adapters/driving/http/locks.rs` (new) or `src/adapters/driving/http/mod.rs`. Delete duplicate definition.
3. Update both call sites to use the shared import.

**Files to change:**
- `src/adapters/driving/http/server_impl.rs` — delete no-op lines, drop local `read_lock_or_recover`
- `src/adapters/driving/http/app_state.rs` — drop local `read_lock_or_recover`
- `src/adapters/driving/http/locks.rs` (NEW) — shared `read_lock_or_recover`

---

## P2 — File-size / structural cleanup

### Fix 10. Inline struct in fn (review P2 #10)

**Subsumed by Fix 2** — NoopForensics struct + impl inside `unwrap_or_else` closure in `game_service.rs:52` (and `init_game.rs:315`). Removing the silent fallback removes the inline struct.

### Fix 11. Drop `LlmMessageBuilder` (review P2 #11)

**Current state:** `src/application/ports/llm_message_repository.rs:24-90` — builder with 9 methods. `LlmMessage` has all public fields. Production path `LlmCallResult::to_message` (at `src/application/ports/llm_provider.rs:48`) uses struct literal directly — not the builder.

All 14 builder callers are test files (count verified by grep):
- `src/adapters/driving/http/fragments/renderers/renderers_tests.rs` (1 call, line 140)
- `src/adapters/driven/storage/backend/llm_messages_tests.rs` (9 calls, lines 6, 143, 169, 182, 195, 205, 218, 231, 247)
- `tests/integration/storage/llm_message_storage.rs` (4 calls, lines 13, 42, 64, 89)

Plan note: builder lives on a **port** file — not a test utility — for a struct with public fields.

**Fix:**

1. Delete `LlmMessageBuilder` struct + impl (lines 24-90).
2. Migrate 14 test call sites to `LlmMessage { field: value, ... }` struct literals directly. Defaults: `id: 0`, `created_at: Utc::now()` where needed (builder had these; check `build()` impl for exact defaults).

**Files to change:**
- `src/application/ports/llm_message_repository.rs` — delete builder
- 3 test files above — 14 sites migrated to struct literals

### Fix 12. Drop `system_prompt` + `user_prompt` from `LlmCallResult` (review P2 #12)

**Current state:** `src/application/ports/llm_provider.rs:12-22`:

```rust
pub struct LlmCallResult {
    pub text: String,
    pub system_prompt: String,        // echoes recorder arg back
    pub user_prompt: String,           // echoes recorder arg back
    pub raw_request_json: String,
    pub raw_response_json: String,
    pub backend_name: String,
    pub model_name: String,
    pub agent_name: String,
}
```

Recorder passes `(system_prompt, user_prompt)` into `provider.complete(...)`, then `to_message()` (line 48) reconstructs `LlmMessage` with those fields. Identity round-trip via the adapter.

**Fix — choose one:**

- **Option A (locked):** Build the `LlmMessage` in the recorder from its args + the chat result fields. Drop `system_prompt` + `user_prompt` from `LlmCallResult`. Adapters no longer echo them back.
- **Option B:** Keep the round-trip — document why (e.g. if some adapter needs to mutate the prompts during transport).

Option A locked. Grep confirms no provider mutates `system_prompt`/`user_prompt` during transport today (MockBackend echoes `""`; openrouter/ollama/deepseek echo their input args). Regression risk: zero now, but the contract is tightened — provider loses round-trip channel for prompts. Documented as intentional.

**Implement together with Fix 1 provider-direct-construction (single atomic `LlmCallResult` reshaping pass).**

**Files to change:**
- `src/application/ports/llm_provider.rs` — drop 2 fields + update `to_message` (or delete `to_message` and move construction to recorder)
- `src/application/llm_recorder.rs` — build `LlmMessage` from args + result
- `src/adapters/driven/llm/providers/{openrouter,ollama,deepseek,mock}.rs` — drop field population in `complete()` impls

### Fix 13. Reduce `ctx: &GameServiceContext` threading in `phases.rs` (review P2 #13)

**Current state:** 9 `pub(super) fn` in `src/application/action_pipeline/phases.rs` take `ctx: &GameServiceContext`:

- `persist` (line 37)
- `persist_snapshot_failed` (line 44)
- `error_return` (line 62)
- `phase_narrate` (line 73)
- `phase_post_generation` (line 145)
- `phase_engine_commit` (line 183)
- `phase_trigger_continuation_raw` (line 197)
- `reconcile_post_trigger_npcs` (line 274)
- `load_preset_and_response_length` (line 372)

All eventually thread back from `run_from_input(&ctx, ...)`.

**Fix (included in this plan per plan-review decision — not deferred):**

Introduce a `PipelineRun<'a>` struct borrowing `(pipeline, ctx)` for the duration of `run_from_input`:

```rust
struct PipelineRun<'a> {
    pipeline: &'a ActionPipeline,
    ctx: &'a GameServiceContext,
}

impl<'a> PipelineRun<'a> {
    fn phase_narrate(&self, state: &GameState, inputs: &PipelineInputs) -> ActionOutcome { ... }
    // etc — drops the ctx parameter from every method
}

impl ActionPipeline {
    fn run_from_input(&self, ctx: &GameServiceContext, ...) -> ActionOutcome {
        let run = PipelineRun { pipeline: self, ctx };
        run.phase_narrate(...)
    }
}
```

Drops ~15 `ctx` parameters across `phases.rs`. Not a blocker — accept defer to a Phase 2.x cleanup pass if scope creeps.

**Reviewer flagged as "Acceptable to defer, but call it out as missed cleanup."** Plan-review decision: include in this plan, no defer.

**Files to change:**
- `src/application/action_pipeline/phases.rs` — extract `PipelineRun<'a>`, rewrite 9 phase method signatures
- `src/application/action_pipeline/pipeline.rs` — `run_from_input` constructs `PipelineRun`

---

## P3 — Documentation / metadata

### Fix 14. Correct ADR-027 "exactly 3 files" claim (review P3 #14)

**Current state:** `docs/adr/adr-027-hexagonal-architecture-migration.md` line 74 reads:

> Storage (`Storage` struct with `Backend` enum) is accessed directly by the application layer in **exactly 3 files**:

Line 85: "no other `application/` file may import `Storage` directly" — contradicted by reality.

**Reality (verified by `grep -rn "use crate::adapters" src/application/`):**

Two distinct violation patterns exist — they should not be conflated:

**Pattern 1 — Direct `Storage` import (the ADR-027 claim):**
- `src/application/context.rs:19` — exempted
- `src/application/application_service.rs` — exempted (via sub-modules, e.g. `snapshot_blob`, `worlds`)
- `src/application/game_service.rs:14` — exempted
- `src/application/ports/llm_provider.rs:10` — NOT exempted (closed by Fix 1)
- `src/application/agents/registry.rs:11` — NOT exempted
- `src/application/agents/quantifier/agent.rs:15` — NOT exempted

**Pattern 2 — Storage sub-namespace DTO imports (DTO, not `Storage` struct):**
- `src/application/context.rs:16` — `snapshot_blob::GameStateSnapshot` (exempted)
- `src/application/application_service.rs:22` — `snapshot_blob::GameStateSnapshot` (exempted)
- `src/application/application_service.rs:26` — `worlds::WorldWithMap` (exempted)
- `src/application/message_editing.rs:16` — `snapshot_blob::GameStateSnapshot` (NOT exempted)

**Total outside the 3 exempted files: 4** (3 Storage imports + 1 DTO import), not 5. ADR-027's "exactly 3 files" claim is false — actual count is 7 (3 exempted + 4 violations).

**Fix:**

Two paths:
- **Path A (locked, full scope):** Close the leaks so ADR-027's claim becomes true.
  - `ports/llm_provider.rs` imports — closed by Fix 1.
  - `agents/registry.rs` + `agents/quantifier/agent.rs` — route through `LlmCallRecorder` (or another port) instead of importing `Storage` directly. Plan 2.1 should have closed these per the original "core → ports only" invariant. Investigate why worker left these imports.
  - `message_editing.rs` `GameStateSnapshot` import — this is a DTO import, not a Storage import. `GameStateSnapshot` moves to `src/domain/model/state/game_state_snapshot.rs` (value type). All ~40 usage sites retagged to import from `domain::` — NO re-export shim at the adapter path. Plan-review decision: full retag, no shim.
  - The 3 "exempted" files (`context.rs`, `application_service.rs`) also update their import path to `domain::`.
- **Path B (fallback):** Update ADR-027 to honestly describe the actual exemption list. Add `agents/registry.rs`, `agents/quantifier/agent.rs`, `message_editing.rs` to the exemption list with rationale + markers. Accept the deviation as documented.

Path A locked. **Fallback trigger:** if closing the `agents/registry.rs` + `agents/quantifier/agent.rs` leaks proves load-bearing (constructor signature cascade into T2 reliability plan territory — these constructors take `Storage` to build their own LLM recorder via `get_llm_recorder_for(..., Arc<Storage>)`), fall back to Path B for those 2 files ONLY. `message_editing.rs` leak closes unconditionally via the snapshot move. **Risk flag:** the Storage-arg-to-agents pattern is T2-adjacent; don't let it block the rest of Path A.

**Files to change:**
- `docs/adr/adr-027-hexagonal-architecture-migration.md` — update exemption count + list
- (Path A only): `src/application/agents/registry.rs`, `src/application/agents/quantifier/agent.rs`, `src/application/message_editing.rs`
- Related: `docs/plans/hexagonal-deferred-arch-lint-rules.md` rule #4 leak list — update

---

## Suggested fix ordering (reviewer's)

Reviewer suggested implementation order:

1. **Fix 1 + Fix 8** — delete `get_llm_backend_for`, move `sanitize_llm_output` into `application/`, decide on `ChatCompletionResult` placement.
2. **Fix 2 + Fix 4 (prod copy) + Fix 10** — propagate `get_llm_recorder_for` errors; remove silent `Mock + Noop` fallbacks from `game_service.rs` and `init_game.rs`.
3. **Fix 3** — drop `storage` field from `MockBackend`; migrate the two mock-forensics tests to go through the recorder.
4. **Fix 4 (test copies) + Fix 6** — extract `NoopForensics` to `test_support`; have all sites use `make_test_recorder`; relocate `QuantifierAgent::with_backend`.
5. **Fix 5** — add `GameService::pipeline(&self) -> ActionPipeline`; collapse the 4 inline constructions.
6. **Fix 7** — delete dead `text_check_service` field on `DefaultApplicationService` or wire it through.
7. **Fix 9** — delete the no-op write-back lines in `server_impl.rs`; dedupe `read_lock_or_recover`.
8. **Fix 14** — fix ADR-027 "exactly 3 files" claim to reflect reality.
9. **Fix 11, Fix 12, Fix 13** — P2 cleanup, can land together.

Each step ends with `python build.py` green.

---

## Verification Log (all confirmed)

| Review claim | Severity | Verdict | Evidence |
|---|---|---|---|
| Port file imports adapters (`ChatCompletionResult`, `Storage`) | P0 #1 | ✅ CONFIRMED | `src/application/ports/llm_provider.rs:8,10` |
| Dead `get_llm_backend_for` zero callers | P0 #1 | ✅ CONFIRMED | `grep -rn get_llm_backend_for src/ tests/` → only the definition at line 85 |
| Silent Mock fallback in prod | P0 #2 | ✅ CONFIRMED | `src/application/game_service.rs:47`, `src/bootstrap/init_game.rs:307` |
| Pre-refactor `get_llm_backend_for` was infallible | P0 #2 (regression framing) | ✅ CONFIRMED | `git show 1e5bf6b:.../game_service.rs` — no Result/`?`, direct `Arc::from(get_llm_backend_for(...))` |
| MockBackend double-save | P0 #3 | ✅ CONFIRMED | `mock.rs:95` + `llm_recorder.rs:42`; `llm_factory.rs:30` passes `Some(storage)` sustaining the path |
| 9 `struct NoopForensics` copies | P1 #4 | ✅ CONFIRMED exact count | 2 prod + 7 test sites (matches reviewer exactly) |
| `ActionPipeline::new` Arc::clone triple repeated | P1 #5 | ✅ CONFIRMED | 3 prod + 7+ test sites |
| `QuantifierAgent::with_backend` test-only in prod | P1 #6 | ✅ CONFIRMED | 10 test callers, 0 prod |
| Dead `text_check_service` field/builder | P1 #7 | ✅ CONFIRMED | 0 external callers of `with_text_check_service` |
| Recorder imports adapter `sanitize_llm_output` | P1 #8 | ✅ CONFIRMED | `llm_recorder.rs:35` |
| `server_impl.rs` no-op write-back + dup `read_lock_or_recover` | P1 #9 | ✅ CONFIRMED | `server_impl.rs:23-25`; `read_lock_or_recover` at `app_state.rs:109` + `server_impl.rs:14` |
| Inline struct in fn | P2 #10 | ✅ CONFIRMED | Subsumed by P0 #2 evidence |
| `LlmMessageBuilder` only used by tests | P2 #11 | ✅ CONFIRMED | `LlmCallResult::to_message` (`llm_provider.rs:48`) uses struct literal; 17 builder callers all test files |
| `LlmCallResult` echoes prompts | P2 #12 | ✅ CONFIRMED | `llm_provider.rs:13-23` fields + `to_message` reuse |
| 9 phase methods take `ctx: &GameServiceContext` | P2 #13 | ✅ CONFIRMED | grep returned exactly 9 in `phases.rs` |
| ADR-027 "exactly 3 files" false | P3 #14 | ✅ CONFIRMED | Actual count: 7 prod files importing from `crate::adapters` (3 exempted + 4 violators: `ports/llm_provider.rs:8,10`, `agents/registry.rs:11`, `agents/quantifier/agent.rs:15`, `message_editing.rs:16`). Two distinct patterns: direct `Storage` import (3 violators) + storage sub-namespace DTO import (`message_editing.rs:16` imports `snapshot_blob::GameStateSnapshot`) |

---

## Open decisions (all locked during plan review)

1. **Fix 1 — `ChatCompletionResult` placement:** Option B locked. Providers construct `LlmCallResult` directly; `ChatCompletionResult` stays adapter-internal.
2. **Fix 6 — `QuantifierAgent` shape:** Option A locked. Holds `Arc<dyn LlmProvider>` directly.
3. **Fix 7 — `text_check_service` field:** Option B locked. Delete dead field; document deviation.
4. **Fix 12 — `LlmCallResult` prompts:** Option A locked. Drop fields; recorder builds `LlmMessage` from args.
5. **Fix 13 — `PipelineRun` refactor:** included in this plan, no defer.
6. **Fix 14 — Path A full scope locked.** Path B fallback only for `agents/registry.rs` + `agents/quantifier/agent.rs` IF leak-closing proves T2-load-bearing.
7. **Branch strategy:** new `hexagon-phase2-review-fixes` branch off `hexagon-phase2`.
8. **Phase 2 Deviation 1 amendment:** Fix 3 removes the MockBackend storage-field deviation. Update `hexagonal-reorganization-plan.md` Phase 2 deviations list to reflect amendment.

---

## What this plan does NOT cover

- **Missing unit tests for new Phase 2 files** (`llm_recorder.rs`, `text_check_service.rs`, `llm_factory.rs`, `text_check_factory.rs`, `llm_message_repository.rs`, `text_checker.rs`) — separate plan: `phase2-tests-coverage-fixes.md` (deferred until this plan lands).
- **Integration test folder/file structure misalignment** with `src/` — separate plan: `phase2-tests-coverage-fixes.md` (deferred).
- **T2 reliability plan work** (`ArrivalTaskContext` cancel-token registration + reset race) — `docs/plans/reliability-and-cancellation-plan.md`. Phase 2 Deviation 3 noted T2 not in active window; re-audit when T2 lands.
- **arch-lint rule activation** — Phase 1.7 deviation persists. Out of scope. Marker comments + grep-based acceptance remain status quo.

---

## Failure modes (per plan review)

For each new codepath introduced or behavior change:

- **Fix 2 — `get_llm_recorder_for` returns `Err` in prod:**
  - `GameService::with_storage` propagates `?` → callers panic (`.expect`) or propagate further (server bootstrap). Affects `server_impl.rs:38,45` — server fails to start with logged `EngineError`. Acceptable: broken LLM config should halt bootstrap, not silently degrade.
  - `ArrivalTaskContext::run` loud-skips (Option A): logs `error!` and returns. Early-return happens BEFORE any `is_generating` mutation or state write (factory call at `init_game.rs:305` precedes snapshot load at `run()` line 150). No cleanup needed; no stale `is_generating=true`; no half-written state in storage.
  - Test paths using `with_backends`/`with_mock_quantifier` unaffected — inject explicit recorders.

- **Fix 3 — `MockBackend::new()` no-storage:**
  - Tests previously asserting on MockBackend's self-save now fail loudly. Migration assert: drive through `LlmCallRecorder::complete` + assert on injected repo. If a test silently constructs MockBackend with `None` and then asserts forensics were saved → test fails (correct signal — those tests were testing the double-save behavior, which was the bug).

- **Fix 4 + Fix 6 — shared `NoopForensics` / `make_test_recorder` migration:**
  - If any test site continues to define its own local `NoopForensics` after the canonical import exists → grep-based acceptance (`struct NoopForensics` count = 1) catches it.
  - If a test previously relied on `QuantifierAgent::with_backend` wrapping its injected provider in an ad-hoc recorder with NoopForensics, the migrated version must explicitly construct `Arc<dyn LlmProvider>` and pass directly. Risk: test loses NoopForensics-wrapped recorder behavior. Acceptable — Quantifier was the only consumer of that provider, and forensics for quantifier calls now flow through the actual storage-injected recorder at the orchestrator layer (which is the whole point of Fix 6).

- **Fix 5 — `GameService::pipeline()` returning by value:**
  - 3 `Arc::clone`s per call. Cheap (Arc refcount). No lifetime hazard (verified — `ActionPipeline` has no lifetime param). Failure mode: none — pure refactor.

- **Fix 1 + Fix 12 (atomic) — provider-direct-construction + `LlmCallResult` field-drop:**
  - Each of 4 providers must now build `LlmCallResult` literally instead of via helper. `mock.rs::make_result` already constructs the literal directly (just wraps in helper) — easiest migration. `openrouter/ollama/deepseek` use `call()` → `from_chat_result`. They replace with `LlmCallResult { text: chat.text, raw_request_json: chat.raw_request_json, raw_response_json: chat.raw_response_json, ... }`. Risk: field-set drift between providers if one forgets a field. Mitigation: clippy catches unused fields; tests catch missing field. Acceptable.
  - If a future provider's `complete()` mutates `system_prompt`/`user_prompt` during transport (e.g., token compression), the recorder now uses the ORIGINAL args, not the mutated ones. Grep confirms no provider mutates these today. Regression risk: zero now, but the contract is tightened — provider loses round-trip channel for prompts. Documented as intentional.

- **Fix 13 — `PipelineRun<'a>` refactor:**
  - Pure refactor, no behavior change. Borrow lifetime ties pipeline+ctx to `run_from_input`'s scope. Failure mode: borrow-checker rejection if a phase method escapes with `self` reference leak. Mitigation: phases.rs methods already return `ActionOutcome` (owned), no reference escape.

- **Fix 14 Path A — `GameStateSnapshot` move + leak closing in `agents/registry.rs`,`agents/quantifier/agent.rs`:**
  - If leaks prove load-bearing (constructor signature cascade into T2 territory), plan fallback to Path B (honest exemption list) for those specific files only. `message_editing.rs` leak closes unconditionally via the snapshot move. Don't let it block the rest.

- **`server_impl.rs` no-op write-back deletion (Fix 9):**
  - Removing the write-back is pure no-op removal. The `read_lock_or_recover` produces a clone used only for `text_check_service` construction; the deleted lines wrote that same value back to the lock it was just read from. Zero behavior change.

- **Coverage regression check:**
  - `python build.py --coverage` gate: overall ≥80% threshold must hold post-fix. Pre-fix level is 86.3%. Several Phase 2 files have <50% coverage today and will only be re-tested in the deferred test plan. If coverage drops below 80% due to behavioral-path shifts during these fixes, the gate fails — and since test plan is deferred, the fix plan must not introduce untested new codepaths. Confirmed: every change here is behavior-preserving at the public-API level OR tightens error propagation (Fix 2), so existing tests should remain sufficient. If any existing test breaks due to error-propagation strictness, handle inline — not a plan-level blocker.

---

## Implementation handoff (parent orchestrates subagents) — COMPLETED 2026-07-02

All 11 steps landed on `hexagon-phase2`. Deviations from original routing noted inline.

Sequencing constraint: each step ends with `python build.py` green; steps share files, so implementation MUST be sequential. Workers operate one at a time on the active worktree; parent verifies with `build.py` + targeted grep between steps.

SP estimate + agent routing per step (per AGENTS.md §"PREFER TO USE SUBAGENTS FOR WORK"):

1. **Fix 1 port-cleanup (delete dead `get_llm_backend_for` + adapter imports) + sanitize move** — ~5 SP → `worker`. ✅ landed `618faf8`
2. **Fix 1 finish (provider direct-construction) + Fix 11 (drop builder) + Fix 12 (drop fields)** — ~5 SP → `worker`. Single atomic `LlmCallResult`/`LlmMessage` reshaping pass. ✅ landed `b582aec`
3. **Fix 2 + Fix 10** (silent Mock fallback removal + `GameService::with_storage` → `Result`) — ~8 SP → break into: (a) `GameService`/`init_game` signature change (worker), (b) caller propagation across 10 sites (worker). ✅ landed `145d5cc`
4. **Fix 3 MockBackend storage drop** — ~5 SP → `worker`. Migrate `test_mock_backend_logs_to_storage` + 2 `mock_tests.rs` sites + 20+ `MockBackend::new(Some(...))` test sites. ✅ landed `81b3e75`
5. **Fix 4 NoopForensics extract + `make_test_recorder` dedupe** — ~5 SP → `worker`. ✅ landed `b136688`
6. **Fix 6 QuantifierAgent reshape** — ~8 SP → `worker`. Migrate 5 `agent_tests.rs` callers + 9 `pipeline/actions_tests.rs` callers. ✅ landed `ae4f268` + `0e09491` (worker false-reported on test-site migration; primary agent fixed ~45 integration-test sites directly via scripted sed)
7. **Fix 5 `GameService::pipeline()`** — ~3 SP → `delegate`. ✅ landed `05cebd5`
8. **Fix 7 dead field deletion** — ~1 SP → `delegate`. ✅ landed `6cb53e1`
9. **Fix 9 no-op + dedupe** — ~3 SP → `delegate`. ✅ landed `dac73a3` (+ `6dd34a8` cleanup: untracked `.pi/rag` artifacts; `.pi/` added to `.gitignore`)
10. **Fix 13 `PipelineRun<'a>` refactor** — ~5 SP → `worker`. ✅ landed `ea6e778` (primary agent implemented directly; method visibility + PipelineRun fields reshaping needed careful sequencing not suited to worker subagent)
11. **Fix 14 Path A — `GameStateSnapshot` move + full retag + leak closing + ADR update** — ~13 SP → break into: (a) snapshot move + re-tag ~40 sites (worker), (b) close `agents/registry.rs` + `agents/quantifier/agent.rs` Storage leaks (worker), (c) ADR-027 update (delegate). ✅ landed `a45e0b9` (primary agent implemented directly; sub-task (b) deferred to T2 per Path B fallback trigger — constructor signature cascade is T2-adjacent; ADR-027 exemption list updated to 5 files with deferred-T2 markers)
