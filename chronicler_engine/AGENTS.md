# Chronicler Engine Knowledge Base

**Generated:** 2026-04-18
**Language:** Rust (Edition 2024)
**Type:** Single crate (binary + library)

## OVERVIEW
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## STRUCTURE
```
chronicler_engine/
├── src/                    # Source code (23 .rs files)
│   ├── lib.rs             # Library root (6 modules)
│   ├── main.rs            # Binary entry (CLI + server)
│   ├── error.rs           # EngineError enum
│   ├── engine/            # Game logic (action, logic, parser)
│   ├── model/             # Data structures (world, map, character, state, scenario)
│   ├── narrative/         # LLM integration (llm, prompt, openrouter_client)
│   ├── server/            # Axum HTTP/WebSocket (mod, templates, template_builders, fragments)
│   └── ui/                # Dashboard components (mod, dashboard)
├── tests/                 # Integration tests (6 files)
│   ├── component_tests.rs    # In-process unit tests
│   ├── e2e_tests.rs          # Browser/Playwright tests
│   ├── flow_mock_tests.rs    # Mock LLM tests (port 3006)
│   ├── flow_llm_tests.rs     # Real LLM tests (port 3007, requires OPENROUTER_API_KEY)
│   ├── test_utils.rs         # Shared test helpers
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
| Modify data model | `src/model/` | World, map, character, state, scenario |
| LLM changes | `src/narrative/` | llm.rs (trait), prompt.rs (templates), openrouter_client.rs |
| Web server | `src/server/` | Axum router, WebSocket, HTMX templates |
| Dashboard UI | `src/ui/dashboard.rs` | HTMX components |
| Run tests | `tests/` | flow_mock_tests (fast), flow_llm_tests (requires API key) |
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
| flow_mock_tests | 3006 | Fast - uses mock LLM |
| flow_llm_tests | 3007 | Requires OPENROUTER_API_KEY |
| behavior_tests | 3003 | - |
| layout_tests | 3002 | - |
| spec_tests | 3001 | - |

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
cargo build                # Release build
cargo test                 # All tests
cargo test --test flow_mock_tests  # Fast mock tests only
cargo run -- --world redmist_estate --port 3000
cargo clippy -- -D warnings    # Strict linting
cargo fmt                  # Format code
```

## NOTES
- LLM requires `OPENROUTER_API_KEY` env var or .env file
- Default world: `redmist_estate`
- WebSocket at `/ws` for real-time updates
- Game state is Single Source of Truth in `src/model/state.rs`
- Use `python build.py` for complete validation (runs fmt, clippy, tests, coverage)