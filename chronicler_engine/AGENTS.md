# Chronicler Engine Knowledge Base

**Generated:** 2026-04-13
**Language:** Rust (Edition 2024)
**Type:** Single crate (binary + library)

## OVERVIEW
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## STRUCTURE
```
chronicler_engine/
├── src/                    # Source code (19 .rs files)
│   ├── lib.rs             # Library root (6 modules)
│   ├── main.rs            # Binary entry (CLI + server)
│   ├── error.rs           # EngineError enum
│   ├── engine/            # Game logic (action, logic, parser)
│   ├── model/             # Data structures (world, map, character, state)
│   ├── narrative/         # LLM integration
│   ├── server/            # Axum HTTP/WebSocket
│   └── ui/                # Dashboard components
├── tests/                 # Integration tests (8 files)
├── docs/                  # Extensive (22 .md files)
│   ├── system/            # Design docs
│   ├── architecture/      # System specs
│   ├── plans/             # Implementation plans
│   └── reference/         # API/data schemas
├── data/worlds/           # Game data (JSON configs)
└── scripts/               # Python helpers
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Add game feature | `src/engine/` | Action enum, parser, logic |
| Modify data model | `src/model/` | World, map, character, state |
| LLM changes | `src/narrative/llm.rs` | LlmBackend trait |
| Web server | `src/server/` | Axum router, WebSocket |
| Dashboard UI | `src/ui/dashboard.rs` | HTMX components |
| Run tests | `tests/` | flow_mock_tests, flow_llm_tests |
| Add world | `data/worlds/<name>/` | world.json, map.json, player.json |
| Write docs | `docs/` | Follow existing structure |

## CONVENTIONS (THIS PROJECT)
- **Result over panic**: Use `EngineError` enum, propagate with `?`
- **Import order**: std → external crates → local modules
- **Tests**: Inline `#[cfg(test)]` in source + `tests/` directory
- **Async**: Use `#[tokio::test]` for integration tests
- **LLM backend**: Trait-based (`LlmBackend`), mock via env var `LLM_BACKEND=mock`
- **Validation**: Run `cargo fmt`, `cargo clippy`, `cargo test` before commit

## ANTI-PATTERNS (THIS PROJECT)
- **Never** use `.unwrap()` or `.expect()` in production code (found in `src/ui/dashboard.rs:32,184`, `src/server/mod.rs:142`)
- **Never** commit without running `cargo clippy`
- **Never** skip architecture update (see `docs/architecture/system.md`)

## UNIQUE STYLES
- Rust 2024 edition (requires Rust 1.85+)
- Extensive SDD docs hierarchy (adr/, plans/, system/, reference/)
- Smart waiting in tests (poll-based, not sleep)
- World loading from external JSON (not hardcoded)

## COMMANDS
```bash
cargo build                    # Release build
cargo test                     # All tests
cargo test --test flow_mock_tests  # Fast mock tests only
cargo run -- --world redmist_estate --port 3000
cargo clippy -- -D warnings    # Strict linting
```

## NOTES
- LLM requires `OPENROUTER_API_KEY` env var or .env file
- Default world: `redmist_estate`
- WebSocket at `/ws` for real-time updates
- Game state is Single Source of Truth in `src/model/state.rs`