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

| Test File / Directory | Purpose | Execution Model |
|----------------------|---------|-----------------|
| `architecture.rs` | Architecture guardrails (arch-lint, layer enforcement) | In-process |
| `components/` | Templates, endpoints, settings, validation, fragments | In-process |
| `browser/` | UI structure, layouts, interactions, editing | Browser |
| `flow_mock/` | Core game loop, retry, state consistency with mock LLM | In-process + Mock LLM |
| `flow_llm_tests.rs` | LLM narrative smoke tests | Browser + Real LLM |
| `game_service/` | Game service logic, DI, retry, snapshots | In-process |
| `guardrails/` | Custom convention tests (imports, comments, file length) | In-process |
| `logic_tests.rs` | Movement, room resolution, fuzzy matching | In-process |
| `llm_message_storage_tests.rs` | SQLite LLM message persistence, auto-pruning | In-process |
| `snapshot_storage_tests.rs` | SQLite snapshot persistence, checkpoints | In-process |
| `state_snapshot_tests.rs` | Snapshot serialization/deserialization | In-process |
| `text_check_tests.rs` | Spell/grammar checking with harper-core | In-process |
| `trigger_tests.rs` | Trigger evaluation and firing | Browser + Mock LLM |
| `diagnostic/` | Backend diagnostics, scenario validation | In-process |

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
cargo nextest run
python build.py

# Include slow LLM tests
cargo nextest run --run-ignored only
python build.py --include-llm

# Run only LLM tests
cargo nextest run --test flow_llm_tests --run-ignored only
python build.py --llm-only

# Specific suites
cargo nextest run --test components
cargo nextest run --test browser
cargo nextest run --test flow_mock
cargo nextest run --test game_service
```

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
