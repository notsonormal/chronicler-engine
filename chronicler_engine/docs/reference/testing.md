# Specification: Testing Strategy and Architecture

## Objective
Establish a formal policy for ensuring the Chronicler Engine remains heavily tested locally without incurring costs or latency from external LLM APIs.

## Policy Rules
1. **Isolated Unit Tests**: All modules maintain fully isolated unit tests `#[test]` with zero networking overhead. Unit tests live in separate `*_tests.rs` sibling files (not inline `#[cfg(test)]` blocks) to keep source files under the 2,000-line guardrail.
2. **Integration Capabilities**: Cross-module and end-to-end tests live in the top-level `tests/` directory.
3. **LLM Abstraction (The Trait Pattern)**: No component outside of `main.rs` should be hardcoded to contact an external LLM API. Use `MockBackend` and `MockQuantifierBackend` for tests.

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

| Test File | Purpose | Execution Model |
|-----------|---------|-----------------|
| `architecture.rs` | Architecture guardrails (arch-lint, layer enforcement) | In-process |
| `components.rs` | Templates, endpoints, settings, validation | In-process |
| `browser.rs` | UI structure, layouts, interactions | Browser |
| `flow_mock_tests.rs` | Core game loop with mock LLM | Browser + Mock LLM |
| `flow_llm_tests.rs` | LLM narrative smoke tests | Browser + Real LLM |
| `game_service_tests.rs` | Game service logic, DI, retry, snapshots | In-process |
| `guardrails.rs` | Custom convention tests (imports, comments, file length) | In-process |
| `logic_tests.rs` | Movement, room resolution, fuzzy matching | In-process |
| `text_check_tests.rs` | Spell/grammar checking with harper-core | In-process |
| `trigger_tests.rs` | Trigger evaluation and firing | Browser + Mock LLM |

## Backend Selection

### Dependency Injection (Recommended)

Pass mock backends directly to `DefaultGameService`:

```rust
let service = DefaultGameService::with_backends(
    Arc::new(MockBackend),
    Arc::new(MockQuantifierBackend::default()),
);
```

### Config File (`tests/test_config.json`)

For integration tests that need environment-specific overrides.

### Mock Settings File

Integration tests write a temporary `settings.json` with Mock connections and set `CHRONICLER_SETTINGS_PATH`.

## Running Tests

```bash
# Fast suite (default; LLM tests excluded)
cargo test
python build.py

# Include slow LLM tests
cargo test -- --ignored
python build.py --include-llm

# Run only LLM tests
cargo test --test flow_llm_tests -- --ignored
python build.py --llm-only

# Specific suites
cargo test --test components
cargo test --test browser
cargo test --test flow_mock_tests
```

## UI Tests

UI tests use **playwright-rs** for browser automation. Requires Node.js 18+ and Chromium installed via `npx playwright install chromium`.

### Running with Visible Browser

```bash
$env:HEADED = "1"; cargo test --test browser test_page_loads
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
