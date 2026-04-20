# Chronicler Engine Knowledge Base

**Generated:** 2026-04-20
**Language:** Rust (Edition 2024)
**Type:** Single crate (binary + library)

## OVERVIEW
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## STRUCTURE
```
chronicler_engine/
├── src/                    # Source code (27 .rs files)
│   ├── lib.rs             # Library root (7 modules)
│   ├── main.rs            # Binary entry (CLI + server)
│   ├── error.rs           # EngineError enum
│   ├── engine/            # Game logic (action, logic, parser, trigger_eval)
│   ├── model/             # Data structures (world, map, character, state, scenario, trigger)
│   ├── narrative/         # LLM integration (llm, prompt, openrouter_client, continuation, quantifier)
│   ├── server/            # Axum HTTP/WebSocket (mod, templates, template_builders, fragments)
│   └── ui/                # Dashboard components (mod, dashboard)
├── tests/                 # Integration tests (7 files)
│   ├── component_tests.rs    # In-process unit tests
│   ├── e2e_tests.rs          # Browser/Playwright tests
│   ├── flow_mock_tests.rs    # Mock LLM tests
│   ├── flow_llm_tests.rs     # Real LLM tests (requires OPENROUTER_API_KEY)
│   ├── trigger_tests.rs      # Trigger system integration tests
│   ├── test_utils.rs         # Shared test helpers (TestServer, port allocation)
│   └── test_data.rs          # Test fixtures
├── docs/                  # Extensive documentation (34+ .md files)
│   ├── architecture/      # System specs (system.md)
│   ├── system/            # Domain docs (dashboard, navigation, narration, llm, etc.)
│   ├── plans/            # Implementation plans (active + archived/)
│   ├── adr/              # Architecture Decision Records
│   └── reference/        # Data schemas, API specs, testing strategy
├── data/
│   ├── worlds/           # Game data (JSON configs per world)
│   │   ├── redmist_estate/  # Default world
│   │   └── test/             # Test world
│   └── images/           # Character sprites and assets
└── scripts/              # Python helpers
    ├── refine_character_json.py
    ├── extract_sillytavern_png.py
    ├── extract_images.py
    └── coverage_summary.py
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Add game feature | `src/engine/` | Action enum, parser, logic |
| Modify data model | `src/model/` | World, map, character, state, scenario, trigger |
| LLM changes | `src/narrative/` | llm.rs (trait), prompt.rs (templates), openrouter_client.rs, continuation.rs |
| Trigger system | `src/engine/trigger_eval.rs` | Trigger evaluation, condition checking |
| Web server | `src/server/` | Axum router, WebSocket, HTMX templates |
| Dashboard UI | `src/ui/dashboard.rs` | HTMX components |
| Run tests | `tests/` | trigger_tests (Playwright), flow_mock_tests, flow_llm_tests |
| Add world | `data/worlds/<name>/` | world.json, map.json, player.json, characters/ |
| Write docs | `docs/` | Follow existing structure |

## CONVENTIONS (THIS PROJECT)
- **Result over panic**: Use `EngineError` enum, propagate with `?`
- **Import order**: std → external crates → local modules
- **Tests**: Inline `#[cfg(test)]` in source + `tests/` directory
- **Async**: Use `#[tokio::test]` for integration tests
- **LLM backend**: Trait-based (`LlmBackend`), mock via `LLM_BACKEND=mock` env var
- **Validation**: Run `cargo fmt`, `cargo clippy`, `cargo test` before commit

## TEST PORTS
| Test Suite | Port | Notes |
|------------|------|-------|
| Most test suites | Dynamic (3010-3050) | Allocated per-test from `tests/test_config.json` |
| flow_llm_tests | Real LLM | Requires `OPENROUTER_API_KEY` env var |

**Note:** Test ports are dynamically allocated to avoid port conflicts when tests run in parallel. The `TestServer` utility in `test_utils.rs` allocates ports from the range 3010-3050 via `get_available_port()`. Tests do NOT hardcode port numbers — they call `get_config_port("tests/test_config.json")` at runtime.

## TEST CONFIGURATION
| File | Purpose |
|------|---------|
| `tests/test_utils.rs` | `TestServer`, `get_available_port()`, smart waiting helpers |
| `tests/test_config.json` | Port range (3010-3050) and per-test backend selection |
| `tests/test_data.rs` | Test fixtures (NpcCard, Room, GameState builders) |
| `tests/trigger_tests.rs` | Integration tests for the trigger system (Playwright + mock LLM) |

All integration tests use:
- `TestServer::with_config()` or `TestServer::from_config()` to start the server
- `get_config_port()` to get a dynamically allocated port
- `LLM_BACKEND=mock` for fast test execution (no real LLM API calls)

## ANTI-PATTERNS (THIS PROJECT)
- **Never** use `.unwrap()` or `.expect()` in production code
- **Never** commit without running `cargo clippy`
- **Never** skip architecture update (see `docs/architecture/system.md`)

## UNIQUE STYLES
- Rust 2024 edition (requires Rust 1.85+)
- Extensive SDD docs hierarchy (adr/, plans/, system/, reference/)
- Smart waiting in tests (poll-based, not sleep)
- World loading from external JSON (not hardcoded)
- HTMX for UI (no client-side JS framework)

## COMMANDS
```bash
python build.py             # Full build + test (recommended)
cargo build                 # Release build
cargo test                  # All tests
cargo test --test flow_mock_tests  # Fast mock tests only
cargo run -- --world redmist_estate --port 3000
cargo clippy -- -D warnings # Strict linting
cargo fmt                   # Format code
```

## NOTES
- LLM requires `OPENROUTER_API_KEY` env var or .env file
- Default world: `redmist_estate` (use `test` for testing)
- Game state is Single Source of Truth in `src/model/state.rs`
- Use `python build.py` for complete validation (runs fmt, clippy, tests, coverage)
- You should aggressively stop/kill the running application if it is stopping you from building/rerunning the application