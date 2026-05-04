# Chronicler Engine Knowledge Base

**Generated:** 2026-04-21
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
├── docs/                  # Extensive documentation (34+ .md files)
│   ├── architecture/      # System specs (system.md)
│   ├── system/            # Domain docs (dashboard, navigation, narration, llm, triggers, etc.)
│   ├── plans/            # Implementation plans (active + archived/)
│   ├── adr/              # Architecture Decision Records
│   └── reference/        # Data schemas, API specs, testing strategy
├── data/
│   ├── worlds/           # Game data (JSON configs per world)
│   └── images/           # Character sprites and assets
└── scripts/              # Python helpers
```

## DOCUMENTATION STRATEGY: SEMANTIC MAPPING
This project follows a **Spec-Driven Implementation** (SDI) strategy.

### The Golden Rule: Spec-First
**NEVER** implement a new technical system or narrative logic without first creating/updating its specification in `docs/`. The code must reflect the spec, not the other way around.

### Core Principles
1. **Naming as Documentation**: Symbols (functions, types, variables) must use verbose, domain-aligned names that map 1-to-1 with concepts in the `docs/`.
2. **Doc Anchors**: Complex logic blocks are marked with `// [DOC: docs/path/to/file.md]`.
3. **Lean Code**: Remove all "What" comments. If the code isn't clear, rename the symbols.
4. **The "Why" Exception**: Comments are reserved ONLY for technical constraints (e.g., `// Workaround for Axum timeout issue`).

### THE TEST-FIRST PHILOSOPHY
This project relies on a comprehensive suite of integration tests as the ultimate source of truth for behavior.
- **Tests as Documentation**: If you don't understand how a component works, read its tests in `tests/` before reading the source code.
- **Test-Driven Debugging**: Before fixing a bug, find or create a failing test case. If tests pass but the bug exists, the test suite is missing a scenario.
- **No Regression**: Every change must be verified by running `python build.py` (fast path: debug build + tests).

### Example: Semantic vs. Traditional
**❌ BAD (Traditional)**
```rust
// Loop through NPCs and check if they are in the room
for npc in all_npcs {
    if npc.room_id == current_room {
        // ...
    }
}
```

**✅ GOOD (Semantic Mapping)**
```rust
// [DOC: docs/system/navigation.md]
let residents = find_npcs_in_current_location(all_npcs, current_room);
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Add game feature | `src/engine/` | Action enum, parser, logic |
| Modify data model | `src/model/` | World, map, character, state, scenario, trigger |
| LLM changes | `src/narrative/` | llm.rs (trait), prompt.rs (templates) |
| Trigger system | `src/engine/trigger_eval.rs` | Trigger evaluation, condition checking |
| Web server | `src/server/` | Axum router, WebSocket, HTMX templates |
| Dashboard UI | `src/ui/dashboard.rs` | HTMX components |

## CONVENTIONS
- **Result over panic**: Use `EngineError` enum, propagate with `?`
- **Doc Anchors**: Always link complex blocks to `docs/` via `// [DOC: docs/path/to/file.md]`
- **LLM backend**: Trait-based (`LlmBackend`), mock via `LLM_BACKEND=mock` env var
- **Validation**: Run `python build.py` before commit (fmt + clippy + tests + guardrails)

## LLM TEST POLICY
- `python build.py` runs the fast suite only. LLM tests are `#[ignore]`d by default.
- When modifying ANY file in `src/narrative/` or changing LLM prompt/parsing behavior,
  you MUST also run `python build.py --llm-only` to verify real LLM integration.

## ANTI-PATTERNS
- **Never** use redundant "What" comments (e.g., `// Add to log`).
- **Never** skip architecture/spec update before implementing engine changes.
- **Never** use `.unwrap()` or `.expect()` in production code.

## GUARDRAILS (PROGRAMMATIC ENFORCEMENT)

Conventions above are **not advisory** — they are enforced automatically.
If AI-generated code violates these rules, the build will fail.

### Layer 1: Clippy (Compile-Time)
`src/lib.rs` declares `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::dbg_macro, clippy::todo, clippy::unimplemented, clippy::print_stdout, clippy::print_stderr)]`.
Test code is exempt via `#![cfg_attr(test, allow(...))]`.
Binary code (`main.rs`) is exempt for CLI bootstrap only.

### Layer 2: arch-lint (Test-Time)
`tests/architecture.rs` runs `arch_lint::check!()` against `arch-lint.toml`.

**Configured rules:**
- `no-unwrap-expect` (AL001) — forbids `.unwrap()` / `.expect()` in production code
- `no-sync-io` (AL002) — forbids blocking I/O in async contexts
- `no-error-swallowing` (AL003) — forbids silently swallowed errors
- `no-silent-result-drop` (AL013) — forbids discarding `Result` without handling
- `require-thiserror` (AL005) — requires `thiserror::Error` derive on error types
- Layer enforcement via `[[deny-scope-dep]]` — `model/` must not import `server/`, `narrative/`, or `engine/`

**Suppressing a violation:**
```rust
// For infallible operations only (e.g., hardcoded regex, static HTTP response)
#[allow(clippy::expect_used)]
#[arch_lint::allow(no_unwrap_expect, reason = "Hardcoded pattern, validated at compile time")]
```

### Adding New Rules
To encode new review feedback as a permanent guardrail:
1. **Clippy-level** (mechanical): Add the lint to `#![deny(...)]` in `src/lib.rs`
2. **Architecture-level** (structural): Add a declarative rule to `arch-lint.toml` (scopes, dependency bans, crate preferences)
3. **Custom rule** (advanced): Write a Rust rule using `arch_lint_core::Rule` and register it in `tests/architecture.rs`

## COMMANDS
```bash
python build.py             # Fast validation (fmt + clippy + guardrails + debug build + tests)
python build.py --release   # Release build + tests + package
cargo test                  # All tests
cargo run -- --world redmist_estate --port 3000
```

## Repository Map

A full codemap is available at `src/codemap.md`.

Before working on any task, read `src/codemap.md` to understand:
- Engine architecture and entry points
- Module responsibilities and design patterns
- Data flow between engine components