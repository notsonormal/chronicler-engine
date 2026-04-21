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
- **Validation**: Run `python build.py` before commit (fmt + clippy + tests)

## ANTI-PATTERNS
- **Never** use redundant "What" comments (e.g., `// Add to log`).
- **Never** skip architecture/spec update before implementing engine changes.
- **Never** use `.unwrap()` or `.expect()` in production code.

## COMMANDS
```bash
python build.py             # Full build + test (recommended)
cargo build                 # Release build
cargo test                  # All tests
cargo run -- --world redmist_estate --port 3000
```

## Repository Map

A full codemap is available at `src/codemap.md`.

Before working on any task, read `src/codemap.md` to understand:
- Engine architecture and entry points
- Module responsibilities and design patterns
- Data flow between engine components