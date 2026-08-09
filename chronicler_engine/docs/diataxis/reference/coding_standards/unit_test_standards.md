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

If the function is reused across tests, lift the input construction into a `fn make_<thing>() -> <Type>` at the bottom of the file (promote to `src/test_support/` only when used by ≥3 test files).

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

## Pattern 4 — LLM recorder / orchestration

**Purpose.** Test the orchestration seam around `LlmCallRecorder`: provider gets called, sanitization runs, forensics get persisted, errors propagate correctly. The recorder is the system under test.

**The standard.**

```rust
use std::sync::Arc;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::test_support::make_test_recorder;

#[test]
fn complete_<behaviour>_<expected_outcome>() {
    let provider = Arc::new(MockBackend::new());
    let recorder = make_test_recorder(provider);

    let result = recorder
        .complete("agent_name", "system_prompt", "user_prompt", None);

    <assert result matches expected outcome>;
    <if expected: construct recorder with storage and assert forensics state via `storage.list_latest_llm_messages(N)` (`len()` for count, `.pop()` for last saved message)>;
}
```

Use `MockBackend::default()` for the deterministic default response. Use `MockBackend::new().with_fail()` / `.with_empty_response()` / `.with_narrations(...)` only when the test specifically exercises that variant of provider behaviour.

## Pattern 5 — Application service-layer

**Purpose.** Test an application service method (`execute_action`, `commit_trigger_narration`, query handlers, etc.) end-to-end through the pipeline-override seam. The service is the system under test; the LLM is mocked.

**The standard.**

```rust
use std::sync::Arc;
use crate::test_support::{TestAppBuilder, TestDataBuilder, TestStoredTriggerContext};
use crate::test_support::{make_test_pipeline_with_mock_quantifier, make_test_recorder};
use crate::adapters::driven::llm::providers::MockBackend;

#[test]
fn test_<method>_<expected_outcome>() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(pipeline)
        .build_service();

    <call app.<method>(<args>)>;
    <assert final_state via app.persistence_gate.load_or_fresh() or service observable>;
}
```

The override supplies recorder / assembler / agent registry; `build_app_graph_for_tests` rebinds the pipeline's persistence and settings to the app graph, so the pipeline and the seeded storage always agree.

The service method under test is invoked directly on the built `app` (e.g., `app.process_action(input)` or `app.execute_action(input)`). To exercise the public `process_action` API path end-to-end against real SQLite, use Pattern 1 (`SqliteTestAppBuilder`) and place the test under `tests/http/`.

## Pattern 6 — HTTP handler

**Purpose.** Test an axum HTTP handler end-to-end against the `TestAppBuilder`. The handler is the system under test; the response status code or HTML response body is the assertion.

**The standard.**

```rust
use axum::http::StatusCode;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_<handler>_<expected_outcome>() {
    let state = TestAppBuilder::default_test().build_service();
    let result = <handler_fn>(axum::extract::State(state), <other_args>).await;
    assert_eq!(result.0.status(), StatusCode::OK);
    // OR, for HTML:
    assert!(!result.0.is_empty());
    // OR, for handlers returning Result:
    assert!(matches!(result, Ok(_) | Err(<expected_error>)));
}
```

Tests are `#[tokio::test]` + `async fn test_…`.

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

## Pattern 8 — Property-based (proptest!)

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
        let state = result.unwrap();
        state.assert_state_consistency(&deps.map, &deps.npcs).ok();
    }
}
```

Co-locate `proptest!` blocks with regular `#[test]`s for the same function — proptest covers the input-space general case; specific `#[test]`s cover known edge cases (boundary, off-by-one, ordering). Use both.

## Cross-cutting patterns

### Cross-cutting A — Failure injection via `TestOverride`

**Purpose.** Test that the storage layer handles storage failures correctly (insert_message that returns a SQL error, save that fails because the file is unwritable, etc.). Used in 14 files.

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

Applied at storage, action-pipeline, and HTTP-fragment tiers (`storage/backend/*_tests.rs`, `application/action_pipeline/{pipeline,retry}_tests.rs`, `adapters/driving/http/{settings,prompt_presets}/handlers/*_tests.rs`).

### Cross-cutting B — XSS regression checks

**Purpose.** Test any HTML renderer that interpolates user-controlled data. **Always required for renderer tests.** These tests are load-bearing — never delete one without replacement.

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

Coverage scope: every renderer that interpolates user-controlled data — Askama templates, `fragment_*` handlers, and `render_*` functions touching message text or trigger context.

### Cross-cutting C — Idempotency tests

**Purpose.** Test a function that should be safe to call twice with the same input and produce the same observable state (e.g., `seed_game_data`, `world_seeded_default`). Used where the operation touches a side-effecting store.

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

### Cross-cutting D — Sealed-trait polymorphism tests

**Purpose.** Test that a trait implementation correctly satisfies the trait contract (e.g., `TextChecker`). Used in trait-port tests where the contract is the system under test, not the implementation.

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

Critical rule: **scope-guard the test to polymorphism only** — implementation-internal tests belong with the impl (e.g., Harper-specific tests live in `harper_text_checker_tests.rs`, not `text_checker_tests.rs`).

## Document References

- [`./testing.md`](./testing.md) — testing policy (terse, what's required, builder selection).
- `src/test_support/mod.rs` — entry point for fixture builders.
- `tests/infrastructure/guardrails/structure.rs` (ADR-028) — the `check_test_module_header` guardrail; enforces single-line `//! <summary>` on every `*_tests.rs` file.
