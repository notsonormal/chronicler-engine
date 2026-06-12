# Specification: Testing Strategy and Architecture

## Objective

Establish a formal policy for ensuring the Chronicler Engine remains heavily tested locally without incurring costs or latency from external LLM APIs.

## Policy Rules

1. **Isolated Unit Tests**: All modules maintain fully isolated unit tests `#[test]` with zero networking overhead. Unit tests live in separate `*_tests.rs` sibling files (not inline `#[cfg(test)]` blocks) to keep source files under the 2,000-line guardrail.
2. **Integration Capabilities**: Cross-module and end-to-end tests live in the top-level `tests/` directory.
3. **LLM Abstraction (The Trait Pattern)**: No component outside of `main.rs` should be hardcoded to contact an external LLM API. Use `MockBackend` (implements `LlmBackend`) for tests.

## Test File Organization

### Unit Tests (Sibling Files)

Unit tests are co-located with source code in separate `*_tests.rs` files:

```
src/
├── engine/
│   ├── logic.rs
│   └── logic_tests.rs
├── model/
│   ├── state.rs
│   └── state_tests.rs
└── narrative/
    ├── llm_client.rs
    └── llm_client_tests.rs
```

Parent `mod.rs` declares the test module:

```rust
#[cfg(test)]
mod logic_tests;
```

### Integration Tests (`tests/` Directory)

Cross-module and browser-based tests live in the top-level `tests/` directory:

| Test File / Directory | Purpose | Execution Model | Runtime |
|----------------------|---------|-----------------|---------|
| `architecture.rs` | Architecture guardrails (arch-lint, layer enforcement) | In-process | ~2s |
| `components/` | Templates, endpoints, settings, validation, fragments | In-process | ~7s |
| `browser/` | UI structure, layouts, interactions, editing | Browser | ~37s |
| `flow_mock/` | Core game loop, retry, state consistency with mock LLM | In-process + Mock LLM | ~30s |
| `flow_llm_tests.rs` | LLM narrative smoke tests | Browser + Real LLM | ~30–120s |
| `game_service.rs` | Service boundary — constructors, trait delegation | In-process | ~1s |
| `action_pipeline/` | Pipeline behavior — narration, quantifier, trigger, retry, cancellation | In-process | ~0.7s |
| `guardrails/` | Custom convention tests (imports, comments, file length) | In-process | ~2s |
| `llm_message_storage_tests.rs` | SQLite LLM message persistence, auto-pruning | In-process | ~1s |
| `snapshot_storage_tests.rs` | SQLite snapshot persistence, game CRUD | In-process | ~2s |
| `poison_recovery.rs` | Lock poison recovery for `Mutex`/`RwLock` | In-process | ~1s |
| `cli_tests.rs` | CLI argument parsing | In-process | ~1s |
| `invariant_contract_tests.rs` | Runtime invariant regression tests | In-process | ~0.1s |
| `browser/trigger.rs` | Trigger evaluation and firing | Browser + Mock LLM | ~30s |
| `diagnostic/` | Backend diagnostics, scenario validation | In-process | ~2s |

## Backend Selection

### Dependency Injection (Recommended)

Pass mock backends directly to `DefaultGameService` and call the public `execute_action()` method:

```rust
let service = DefaultGameService::with_mock_quantifier(
    Arc::new(MockBackend::new(None)),
    Arc::new(MockBackend::new(None)),
);

let ctx = GameServiceContext::new(/* ... */);
service.execute_action(ctx, "look around".to_string(), "Player".to_string());
```

**Note:** Do not call `execute_action_impl()` directly in tests — use the public `DefaultGameService::execute_action()` wrapper method instead. The `execute_action_impl()` function is an internal implementation detail of the action pipeline module.

### Pipeline-Only Mocking

For tests that only need to verify pipeline behavior (narration → quantification → trigger continuation), implement the `ActionPipelineBackend` trait directly. This avoids constructing `DefaultGameService` and its full backend/registry graph. The narrow mock can then be used with the `execute_action_impl()` function from the action pipeline module:

```rust
use chronicler_engine::application::action_pipeline::{ActionPipelineBackend, execute_action_impl};
use chronicler_engine::narrative::prompt::PromptAssembler;
use chronicler_engine::application::context::GameServiceContext;

struct NarrowMock;

impl ActionPipelineBackend for NarrowMock {
    fn assembler(&self) -> &dyn PromptAssembler {
        // Return a test assembler (e.g., a mock or LayeredPromptAssembler)
        &TEST_ASSEMBLER
    }

    fn complete(&self, _agent: &str, _sys: &str, _usr: &str, _max: Option<u32>) -> Result<LlmCallResult, EngineError> {
        Ok(LlmCallResult { /* ... */ })
    }

    fn run_post_generation_agents(&self, _state: &GameState, _input: &str, _response: &str, _result: &mut QuantifierResult) {
        // Directly set quantifier result without running real agents
    }
}

// Usage in test:
let backend = NarrowMock;
let ctx = GameServiceContext::new(/* ... */);
execute_action_impl(&backend, ctx, "look".to_string(), "Player".to_string());
```

### Config File (`tests/test_config.json`)

For integration tests that need environment-specific overrides.

### Mock Settings File

Integration tests write a temporary `settings.json` with Mock connections and set `CHRONICLER_SETTINGS_PATH`.

## Running Tests

### Full Suite

```bash
# Default: fast suite (~70 sec, LLM tests excluded)
cargo nextest run
python build.py

# Include slow LLM tests
cargo nextest run --run-ignored only
python build.py --include-llm

# Run only LLM tests
cargo nextest run --test flow_llm_tests --run-ignored only
python build.py --llm-only
```

### Fast Iteration

Run individual test binaries directly (bypasses fmt, clippy, guardrails, and full suite):

| Command | Duration | Use When |
|---------|----------|----------|
| `cargo nextest run --test action_pipeline` | ~0.7s | Pipeline changes |
| `cargo nextest run --test components` | ~7s | Template/endpoint changes |
| `cargo nextest run --test game_service` | ~1s | Service boundary changes |
| `cargo nextest run --test guardrails` | ~2s | Guardrail changes |
| `cargo nextest run --test invariant_contract_tests` | ~0.1s | Invariant changes |
| `cargo nextest run --test browser` | ~37s | UI changes |
| `cargo nextest run --test flow_mock` | ~30s | Flow logic changes |

## UI Tests

UI tests use **playwright-rs** for browser automation. Requires Node.js 18+ and Chromium installed via `npx playwright install chromium`.

### Running with Visible Browser

```bash
$env:HEADED = "1"; cargo nextest run --test browser test_page_loads
```

### Diagnostics on Failure

- Screenshots: `chronicler_engine/tmp/screenshots/`
- DOM dumps: `chronicler_engine/tmp/test_diagnostics/`

## Code Coverage

Coverage is verified using `cargo-llvm-cov`:

```bash
cargo +stable install cargo-llvm-cov --locked
cargo llvm-cov test --json --output-path coverage.json
```

See `docs/adr/` for detailed rationale behind testing patterns.

## Test Fixtures

The `test_support::fixtures` module provides reusable test data builders. Prefer these over inline struct construction:

| Fixture | Methods | Use For |
|---------|---------|---------|
| `TestWorld` | `minimal()`, `with_rule(rule)` | `WorldCard` instances |
| `TestPlayer` | `standard()`, `named(name)` | `PlayerCard` instances |
| `TestNpc` | `named(id, name)`, `with_times_met_trigger(...)`, `with_room_scoped_trigger(...)` | `NpcCard` instances |
| `TestMap` | `room(id)`, `room_named(id, name)`, `single_room(id)`, `two_rooms(a, b)` | `Room` and `MapDef` instances |
| `TestGameState` | `in_room(id)`, `with_npc(...)`, `with_npcs(...)` | `GameState` instances |
| `TestStoredTriggerContext` | `standard()`, `for_npc(...)`, `named(...)`, `with_max_tokens(...)` | `StoredTriggerContext` instances |
| `TestPromptPreset` | `system(id, name)`, `system_default(id, name)` | `PromptPreset` instances |
| `TestWorldManifest` | `minimal()` | `WorldManifest` instances |
| `TestCharacterSheet` | `hero()` | `CharacterSheet` instances |

### Example

```rust
use crate::test_support::{
    TestCharacterSheet, TestMap, TestNpc, TestPlayer,
    TestPromptPreset, TestStoredTriggerContext, TestWorldManifest,
};

let preset = TestPromptPreset::system("my_preset", "My Preset");
let trigger = TestStoredTriggerContext::standard();
let manifest = TestWorldManifest::minimal();
```

## What We Keep

Critical tests that must not be removed:

| Test | Why |
|------|-----|
| `test_list_games_fragment_escapes_html` | XSS security — template escaping |
| `test_action_handler_empty_command` | Empty input triggers continuation (SillyTavern "Continue" behavior) |
| `test_story_log_scrollable` | Functional — can't scroll history |
| `test_no_horizontal_overflow` | Regression — breaks page layout |

## Smart Waiting Patterns

Tests use polling, not fixed sleep:

```rust
// BAD: Fixed delay
sleep(Duration::from_millis(15000)).await;

// GOOD: Wait for condition
wait_for_llm_idle(port, Duration::from_secs(30)).await;
wait_for_status_ready(&page).await;
```

## Test Config

Dynamic port allocation avoids conflicts:

```json
// tests/test_config.json
{
  "port_range": {"min": 3010, "max": 3050},
  "default_backend": "mock"
}
```
