---
diataxis: reference
title: Unit Test Standards
---

## Pattern 1 — Pure unit

**Purpose.** Test a pure function or a method on a domain type with no external dependencies. The function under test is the only thing the test constructs (plus optional domain-builder helpers like `TestPersona::default()`).

**The standard.**

```rust
use crate::<module_path>::<thing_under_test>;

#[test]
fn test_<behaviour>_<expected_outcome>() {
    let input = <construct_minimal_input>;
    let result = <function_under_test>(input);
    assert_eq!(result, <expected>);
}
```

If the function is reused across tests, lift the input construction into a `fn make_<thing>() -> <Type>` at the bottom of the file (do not promote to `src/test_support/` unless used by ≥3 test files).

**Exemplar.** `src/error_tests.rs` (8 tests, `Display` + `From` mappings on the `EngineError` enum, no fixture, no async). `src/domain/model/character_tests.rs` (13 tests, JSON serde roundtrip).

**Drift notes.**

- `src/domain/model/message_history_tests.rs` uses local helpers (`append_messages`, `make_history`) — this is legitimate because `MessageHistory` exposes many operations and the helpers are reused across the 20 tests. Do not copy this pattern into single-test files.
- `src/application/ports/text_checker_tests.rs` is scope-guarded to polymorphism only (1–2 tests verifying trait dispatch). Do NOT duplicate Harper implementation tests here — those live in `src/adapters/driven/text_check/harper_text_checker_tests.rs`.
- Tests of `fn parse_action`, `fn generate_game_name`, and other small parsers/generators should NEVER cross 20 tests unless the input-space is genuinely large (e.g., `quantifier/parser_tests.rs` is allowed 27 tests because it enumerates JSON/markdown/text-fallback parser branches).

## Pattern 2 — Storage backend pair

**Purpose.** Exercise a `Storage` method to confirm it works identically against an in-memory backend (`Storage::new_in_memory()`) and the sqlite-backed variant (`sqlite_storage()` from `src/test_support/`). Storage is the system under test.

**The standard.**

```rust
use crate::adapters::driven::storage::backend::Storage;
use crate::test_support::{dummy_<entity>, sqlite_storage};  // as needed

#[test]
fn test_<method>_<expected_outcome>() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    <exercise method, assert result>
}

#[test]
fn test_<method>_sqlite() {
    let storage = sqlite_storage().unwrap();
    storage.set_game_id(1);
    <identical exercise, identical assert>
}
```

The two tests must be character-for-character identical except for the storage construction. This is the rule that makes the pair actually test parity.

**Exemplar.** `src/adapters/driven/storage/backend/messages_tests.rs` (29 tests, every method has `test_<method>` for in-memory and `test_<method>_sqlite` for sqlite). The pair is the canonical reference; do not deviate from the `_sqlite` suffix on the sqlite test name.

**Drift notes.**

- Some storage files include a third coverage test (`test_<method>_in_memory`) where the in-memory variant is the system under test rather than paired parity. Fold these into other `test_<method>` cases if the method is exercised elsewhere.
- `src/settings_tests.rs` uses `DbPool::new(...)` directly rather than the `Storage` API — this is drift; prefer the `Storage`-level API unless you're explicitly testing the lower-level `DbPool` schema (which lives in `src/adapters/driven/storage/db_tests.rs`).
- All `Storage` methods that can fail should also have a paired failure-injection test. See Cross-cutting pattern A — Failure injection.

## Pattern 3 — LLM provider trait

**Purpose.** Test the `LlmProvider` trait implementation for a specific backend (DeepSeek, Mock, Ollama, OpenRouter). Each test asserts the backend-specific behaviour against the trait surface.

**The standard.**

```rust
use crate::adapters::driven::llm::providers::<BackendName>;
use crate::application::ports::llm_provider::LlmProvider;

#[test]
fn test_<backend>_<method>_<expected_outcome>() {
    let backend = <BackendName>::new(<required_config>);
    let result = backend.<method>(<args>);
    assert_eq!(result, <expected>);
}
```

For non-mock backends, the assertion is typically `name()` / `model_name()` returning the expected strings, plus any backend-specific configuration behaviour. For `MockBackend`, the assertion is typically that `complete()` echoes a configured response.

**Exemplar.** `src/adapters/driven/llm/providers/mock_tests.rs` (10 tests, includes `IN_MEM + LLM_MOCK` for multi-call queue behaviour). `src/adapters/driven/llm/providers/ollama_tests.rs` (5 tests, pure trait surface).

**Drift notes.**

- `src/adapters/driven/llm/providers/deepseek_tests.rs` is intentionally a stub — every test asserts `Err("not yet implemented")`. Keep this guardrail intact until the backend is wired for real.
- Other provider test files (`ollama`, `openrouter`, `deepseek`) are minimal (3–5 tests). They cover only `name()` / `model_name()` / `model_name() from config`. Extend on real backend wiring; do NOT pre-emptively add fake tests.
- If you're testing provider behaviour that interacts with `LlmCallRecorder` (sanitization dispatch, forensics, retry), use Pattern 4 — LLM recorder / orchestration — not this pattern.

## Pattern 4 — LLM recorder / orchestration

**Purpose.** Test the orchestration seam around `LlmCallRecorder`: provider gets called, sanitization runs, forensics get persisted, errors propagate correctly. The recorder is the system under test.

**The standard.**

```rust
use std::sync::Arc;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::test_support::make_test_recorder_with_storage;

#[test]
fn complete_<behaviour>_<expected_outcome>() {
    let provider = Arc::new(MockBackend::new());
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(provider, Arc::clone(&storage));

    let result = recorder
        .complete("agent_name", "system_prompt", "user_prompt", None);

    <assert result matches expected outcome>;
    <if expected: also assert forensics state via `storage.list_latest_llm_messages(N)` (`len()` for count, `.pop()` for last saved message)>;
}
```

Use `MockBackend::default()` for the deterministic default response. Use `MockBackend::new().with_fail()` / `.with_empty_response()` / `.with_narrations(...)` only when the test specifically exercises that variant of provider behaviour.

**Exemplar.** `src/application/llm_recorder_tests.rs` (6 tests covering happy path, sanitization strip, error propagation from provider, error propagation from closure save, provider accessor, configurable mock). Includes a `Send + Sync` static assertion on line 17 — keep that pattern in recorder tests; the recorder must be safe to share across tasks.

**Drift notes.**

- `src/application/agents/quantifier/orchestration_tests.rs` uses `make_test_recorder_with_storage` from `src/test_support/llm_recorder_save_seam.rs` — assert via real `Storage::list_latest_llm_messages` for multi-step orchestration assertions.
- Tests of agent-level orchestration (multi-step retry, fallback to empty response) live in `src/application/agents/quantifier/{agent_tests,orchestration_tests}.rs`, not here. Pattern 4 tests the recorder, not the agent.

## Pattern 5 — Application service-layer

**Purpose.** Test an application service method (`execute_action_impl`, `commit_trigger_narration`, query handlers, etc.) end-to-end through the `GameService` seam. The service is the system under test; the LLM is mocked.

**The standard.**

```rust
use std::sync::Arc;
use crate::test_support::{TestAppBuilder, TestDataBuilder, TestStoredTriggerContext};
use crate::test_support::{make_test_recorder};
use crate::adapters::driven::llm::providers::MockBackend;

fn make_test_service(
    narrator_recorder: Arc<LlmCallRecorder>,
    quantifier_provider: Arc<dyn LlmProvider>,
) -> GameService {
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let registry = AgentRegistry::with_agent(Box::new(agent));
    GameService::with_backends(narrator_recorder, registry)
}

#[test]
fn test_<method>_<expected_outcome>() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service))
        .build_service();

    <call app.<method>(<args>) or call impl function directly>;
    <assert final_state via app.load_or_fresh() or service observable>;
}
```

**Important distinction.** This pattern tests **the application service** via direct invocation of its impl functions (e.g., `execute_action_impl(&app, "look".to_string())`). This is the unit-under-test for service-layer tests. The legacy rule "do not call `execute_action_impl` directly — go through `process_action`" applies to **integration** tests in `tests/integration/`, NOT to unit tests of impl functions. If your test is exercising the public `process_action` API path, that's an integration test and lives under `tests/integration/application/`.

**Exemplar.** `src/application/action_pipeline/actions_tests.rs` (7 tests, canonical pattern with `make_test_service` helper, `TestAppBuilder::with_data(...).game_service(...).build_service()` chain).

**Drift notes.**

- `src/application/action_pipeline/pipeline_tests.rs` and `retry_tests.rs` share the same construction chain but are larger (20 and 25 tests respectively) because the corresponding production modules expose many service methods. The chain is identical; only the test count grows.
- `src/application/application_service_tests.rs` covers both the read-only query surface and the Pattern 5 + Pattern 8 (concurrency) `is_generating` invariant checks. See Pattern 8 for the concurrency-specific additions.
- Failure injection for service-layer tests uses `Storage::new_in_memory().with_test_failures()` directly (the test owns the storage before `build_service` constructs the service from it). See Cross-cutting pattern A.

## Pattern 6 — HTTP handler

**Purpose.** Test an axum HTTP handler end-to-end against the `TestAppBuilder`. The handler is the system under test; the response status code or HTML response body is the assertion.

**The standard.**

```rust
use axum::http::StatusCode;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_<handler>_<expected_outcome>() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = <handler_fn>(axum::extract::State(state), <other_args>).await;
    assert_eq!(result.0.status(), StatusCode::OK);
    // OR, for HTML:
    assert!(!result.0.is_empty());
    // OR, for handlers returning Result:
    assert!(matches!(result, Ok(_) | Err(<expected_error>)));
}
```

Tests are `#[tokio::test]` + `async fn test_…`. There are no plain `#[test]` in handler tests.

**Exemplar.** `src/adapters/driving/http/fragments/endpoints_tests.rs` (11 `#[tokio::test]`s, body_median 4 lines per test, all rendering fragments and asserting non-empty HTML). The compact body is the point — happy path is one render + one assertion; sad paths live in the same file alongside.

**Drift notes.**

- HTTP handler tests are exclusively `#[tokio::test]`. Do NOT use plain `#[test]` for handlers — axum's `extract::State` requires an async runtime.
- Some handler tests have `body_median ≤ 5` lines. That's the canonical shape for happy-path handlers (one render + one assertion). If a happy-path test is longer than 10 lines, the test is probably over-specified — assert the status code, not internal state.
- Handler tests using `TestAppBuilder::default_test()` without `.with_data(...)` cover the empty-state rendering path. Tests that require seeded data use `.with_data(TestDataBuilder::default_test().build())`. Both shapes are valid; prefer empty-state for renderer shape tests, seeded for handler-route tests.
- `src/adapters/driving/http/error_tests.rs` is the canonical place to test `ApplicationError::IntoResponse` mappings (6 tests). Do NOT scatter error-mapping assertions across handler test files.

## Pattern 7 — Fragment renderer / template

**Purpose.** Test that an HTML fragment renderer or Askama template renders correctly for an input context. The output HTML is the system under test. **These tests are load-bearing for XSS regression** — never delete an XSS assertion without replacement.

**The standard.**

```rust
#[test]
fn test_<renderer>_<scenario>() {
    let html = <renderer_fn>(<context>).0;
    assert!(!html.is_empty(), "renderer must produce non-empty HTML");
    <assert presence of expected strings>;
    <assert absence of unsafe strings — see Cross-cutting pattern B>;
}

#[test]
fn test_<renderer>_escapes_hostile_input() {
    let ctx = <context with `<script>alert('xss')</script>` in user-controlled fields>;
    let html = <renderer_fn>(ctx).0;
    assert!(!html.contains("<script>alert('xss')</script>"));
    assert!(html.contains("&lt;script&gt;") || !html.contains("<script"));
}
```

**Exemplar.** `src/adapters/driving/http/templates_tests.rs` (36 tests across 4 Askama templates: header, narrative log, action area, visual sidebar). Includes XSS regression tests at lines 25-32 (`test_header_template_escapes_html`) and 69-81 (`test_story_log_template_escapes_html`). Keep both intact.

**Drift notes.**

- **XSS regression checks are load-bearing** — they exist because real XSS bugs have shipped via renderers accepting user input without escape. See legacy `docs/reference/testing.md` "Critical Test Categories" for the full list.
- Render tests should cover both **empty state** and **populated state** for each renderer. Empty-state tests assert non-empty HTML (the chrome around the data). Populated-state tests assert the data appears in expected escaped form.
- Render tests SHOULD NOT duplicate business-logic assertions (e.g., a render test should not assert that an LLM message was correctly persisted). That belongs in a Pattern 4 (LLM recorder) test.

## Pattern 8 — Concurrency invariant

**Purpose.** Test an invariant between two states that must hold even under concurrent access (typically the `is_generating` cached atomic vs. the persisted `GenerationStatus`). The invariant is the system under test; concurrency is how you stress it.

**The standard.**

```rust
use std::sync::{Arc, Barrier};
use tokio::sync::Notify;

#[tokio::test]
async fn test_<invariant>_<expected_outcome>() {
    let app = TestAppBuilder::default_test()
        .with_data(TestDataBuilder::default_test().build())
        .game_service(<mock service>)
        .build_service();

    let barrier = Arc::new(Barrier::new(2));

    let task1 = {
        let barrier = barrier.clone();
        let app = app.clone();
        tokio::spawn(async move {
            barrier.wait();
            app.set_generating(true).await;
            barrier.wait();
            tokio::time::sleep(Duration::from_millis(50)).await;
            app.set_generating(false).await;
        })
    };

    barrier.wait();
    let result = wait_for_condition(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || app.is_generating(),
    ).await;

    task1.await.unwrap();
    <assert invariant holds>;
}

async fn wait_for_condition(
    timeout: Duration,
    interval: Duration,
    predicate: impl Fn() -> bool,
) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if predicate() { return true; }
        tokio::time::sleep(interval).await;
    }
    false
}
```

The `wait_for_condition` helper is **file-local** in `src/application/application_service_tests.rs` (within the `is_generating` invariant section). Do NOT promote it to `src/test_support/` — it's the only site using this pattern, and the helper belongs to the specific invariant test.

**Exemplar.** `src/application/application_service_tests.rs` — the `is_generating` invariant section (3 tests: helper divergence detection, `wait_until_idle` failure mode with `#[should_panic]`, and the load-bearing TOCTOU regression test using `Arc<Barrier>` for entry/exit coordination).

**Drift notes.**

- `src/application/action_pipeline/actions_tests.rs` uses `std::thread::spawn` + sync barriers for its concurrent tests instead of the `tokio::test` + `tokio::spawn` + async polling standard. This is drift — acceptable because the tests pass, but new concurrent tests should use the async form.
- Tests that exercise concurrency invariants should fail fast on `wait_for_condition` timeout — if polling exceeds the expected interval, the invariant is broken; don't `unwrap()` indifferently.

## Pattern 9 — Property-based (proptest!)

**Purpose.** Cover the input-space of a transformation function (`handle_movement`, `apply_npc_events`, `commit_trigger_narration`, etc.) where specific inputs wouldn't cover all relevant cases. The state-consistency invariant is the system under test.

**The standard.**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_<function>_<invariant_name>(
        <inputs as proptest strategies>
    ) {
        let deps = <build deps via TestMap / TestNpc / etc.>;
        let state = <make state via TestGameState or TestMap helpers>;
        let result = <function_under_test>(state, <inputs>, &deps.map, &deps.npcs);

        prop_assert!(result.is_ok());
        crate::domain::engine::state_diagnostics::assert_state_consistency(
            &result.unwrap(), &deps.map, &deps.npcs,
        )?;
    }
}
```

Co-locate `proptest!` blocks with regular `#[test]`s for the same function — proptest covers the input-space general case; specific `#[test]`s cover known edge cases (boundary, off-by-one, ordering). Use both.

**Exemplar.** `src/domain/engine/action_processing_tests.rs:534` (the `proptest!` block covers `handle_movement`, `apply_npc_events`, and `execute_freeaction_impl` against `assert_state_consistency`). `src/domain/model/state/state_tests.rs:148` (the `prop_log_appends_in_order` test covers input-space of log appending).

**Drift notes.**

- Only 2 files currently use `proptest!`. Do NOT pre-emptively add property-based tests to other files; let the input-space grow naturally.
- Property tests should assert **invariants** (state consistency, count preservation, ordering) — not specific output values. If your property test asserts a specific output, it should be a `#[test]` with a fixed input.
- `proptest!` requires the `proptest` crate in `[dev-dependencies]`. The `use proptest::prelude::*;` at the top of the file is required (don't prune it as "unused" — proptest's macros rely on the prelude).

## Cross-cutting patterns

### Cross-cutting A — Failure injection via `TestOverride`

**When.** Testing that the storage layer handles storage failures correctly (insert_message that returns a SQL error, save that fails because the file is unwritable, etc.). Used in 14 files.

**The standard.**

```rust
use crate::adapters::driven::storage::backend::TestOverride;

#[test]
fn test_<method>_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);
    handle.set("<method_name>", TestOverride::internal("<reason>"));
    // OR, for config-style errors:
    handle.set("<method_name>", TestOverride::config("<reason>"));

    <exercise method>;
    let result = <observe result>;
    assert!(result.is_err());
    // AND, if your production code maps the error:
    assert!(matches!(result.unwrap_err(), EngineError::<ExpectedVariant>(_)));
}
```

The two `TestOverride` variants (`internal` and `config`) map to two different `EngineError` arms downstream. Use `internal` for unexpected runtime errors (DB corruption, write failure) and `config` for input-validation failures.

Used in: all `storage/backend/*_tests.rs` files, `application/action_pipeline/{pipeline,retry}_tests.rs`, and `adapters/driving/http/{settings_fragment,prompt_presets_fragment}/handlers_tests.rs`.

### Cross-cutting B — XSS regression checks

**When.** Testing any HTML renderer that interpolates user-controlled data. **Always required for renderer tests.** These tests are load-bearing — never delete one without replacement.

**The standard.**

```rust
#[test]
fn test_<renderer>_escapes_hostile_input() {
    let ctx = <context including a field set to "<script>alert('xss')</script>">;
    let html = <renderer_fn>(ctx).0;
    assert!(
        !html.contains("<script>alert('xss')</script>"),
        "renderer must escape <script> tags in user input"
    );
}
```

Coverage scope: every Askama template in `src/adapters/driving/http/templates.rs`, every `fragment_*_fn` that interpolates form inputs, and every `render_*_fn` that interpolates message text or trigger context. See legacy `docs/reference/testing.md` "Critical Test Categories" for the full list of regression checks that must never be deleted.

### Cross-cutting C — Idempotency tests

**When.** Testing a function that should be safe to call twice with the same input and produce the same observable state (e.g., `seed_game_data`, `world_seeded_default`). Used where the operation touches a side-effecting store.

**The standard.**

```rust
#[test]
fn test_<operation>_idempotent() {
    <set up clean state>;
    <call operation> first;
    let snapshot_1 = <observe observable state>;

    <call operation> second;
    let snapshot_2 = <observe observable state>;

    assert_eq!(snapshot_1, snapshot_2);
}
```

Used in: `src/bootstrap/load_tests.rs` (`seed_game_data` over filesystem worlds), `src/domain/model/state/state_tests.rs` (log append idempotency).

### Cross-cutting D — Sealed-trait polymorphism tests

**When.** Testing that a trait implementation correctly satisfies the trait contract (e.g., `TextChecker`). Used in trait-port tests where the contract is the system under test, not the implementation.

**The standard.**

```rust
#[test]
fn <trait>_dispatches_across_impls() {
    let impls: Vec<Box<dyn <Trait>>> = vec![
        Box::new(<ImplA>::new()),
        Box::new(<ImplB>::new()),
    ];
    for impl_ in impls {
        let result = impl_.<trait_method>(<args>);
        <assert trait contract is met (not impl-specific behaviour)>;
    }
}
```

Critical rule: **scope-guard the test to polymorphism only.** Do NOT duplicate impl-internal tests in the trait-port test file. Implementation-internal tests belong with the impl (e.g., Harper-specific tests live in `harper_text_checker_tests.rs`, not `text_checker_tests.rs`).

Used in: `src/application/ports/text_checker_tests.rs` (2 tests, polymorphism only).

## Document References

- [`./testing.md`](./testing.md) — testing policy (terse, what's required, builder selection).
- Legacy `docs/reference/testing.md` (referenced from this doc; migration target is `docs-diataxis/reference/testing.md` once ticket 15 lands) — critical test categories list, including the full XSS, Continue, scroll, and overflow regression checks that must never be deleted without replacement.
- `src/test_support/mod.rs` — entry point for fixture builders (`TestWorld`, `TestPersona`, `TestNpc`, `TestMap`, `TestGameState`, `TestStoredTriggerContext`).
- `tests/infrastructure/guardrails/structure.rs` (ADR-028) — the `check_test_module_header` guardrail; enforces single-line `//! <summary>` on every `*_tests.rs` file.
