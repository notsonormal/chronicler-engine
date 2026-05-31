You# Chronicler Engine Knowledge Base

**Generated:** 2026-05-10
**Language:** Rust (Edition 2024)
**Type:** Single crate (binary + library)

## OVERVIEW
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## STRUCTURE
```
chronicler_engine/
├── src/                    # Source code (120+ .rs files)
│   ├── lib.rs             # Library root
│   ├── main.rs            # Binary entry (CLI + server)
│   ├── error.rs           # EngineError enum
│   ├── cli.rs             # Command-line argument parsing
│   ├── settings.rs        # Application settings management
│   ├── bootstrap/         # Startup initialization (load, logging, run, scenario, validate)
│   ├── engine/            # Game logic
│   │   ├── action.rs, action_processing.rs, logic.rs, parser.rs
│   │   ├── trigger_eval.rs, state_diagnostics.rs
│   │   └── game_service/  # Game flow orchestration (actions, context, helpers, retry, service)
│   ├── model/             # Data structures
│   │   ├── agent.rs, character.rs, game.rs, llm_backend.rs, llm_message.rs
│   │   ├── map.rs, message.rs, scenario.rs, settings.rs, state.rs, state_snapshot.rs
│   │   ├── trigger.rs, world.rs
│   ├── narrative/         # LLM integration
│   │   ├── llm_client.rs  # High-level LLM client facade
│   │   ├── agents/        # Agent subsystem (registry, trait_def, quantifier/)
│   │   ├── llm/           # Backend implementations (backend, deepseek, mock, ollama, openrouter)
│   │   ├── prompt/        # Prompt building (budget, builder, context, sanitize, templates, types)
│   │   └── text_check/    # Grammar/spelling checking (check, harper_backend, types)
│   ├── server/            # Axum HTTP/WebSocket
│   │   ├── mod.rs, templates.rs, debug.rs
│   │   ├── fragments/     # HTMX fragment endpoints (actions, endpoints, history, misc, renderers)
│   │   └── settings_fragment/ # Settings UI fragments (fragments, handlers, template)
│   ├── storage/           # Persistence layer (db, models, mappers, snapshot_storage, llm_message_storage)
│   └── test_support/      # Shared test helpers (context, fixtures, in_memory_storage)
├── tests/                 # Integration tests
│   ├── architecture.rs    # arch-lint guardrail tests
│   ├── browser.rs         # Browser automation tests (editing, interaction, structure)
│   ├── components.rs      # In-process server tests (connections, css, debug, fragment, settings, template, text_check, world)
│   ├── diagnostic.rs      # Diagnostic backend tests (backends, scenarios)
│   ├── flow_llm_tests.rs  # End-to-end LLM flow tests
│   ├── flow_mock_tests.rs # End-to-end mock flow tests
│   ├── game_service.rs    # Game flow tests (advanced, basic)
│   ├── guardrails.rs      # Style and structure guardrails
│   ├── logic_tests.rs     # Game logic unit tests
│   ├── test_data.rs       # Test data validation
│   ├── text_check_tests.rs# Text-check integration tests
│   ├── browser/          # Browser integration tests (editing, interaction, structure, trigger)
│   └── test_utils/        # Shared test utilities (browser, server, wait)
├── docs/                  # Extensive documentation (75+ .md files, auto-indexed)
│   ├── architecture/      # System specs (system.md, guardrails.md, invariants.md)
│   ├── system/            # Domain docs (agent_system, character_state, dashboard, dynamic_rooms, game_flow, llm_processing, narration_engine, navigation, prompt_system, startup, text_check, triggers, ui_design)
│   ├── plans/             # Implementation plans (active + archived/)
│   ├── adr/               # Architecture Decision Records (adr-001 through adr-013)
│   ├── diagnostics/       # Error catalog
│   ├── reference/         # Data schemas, API specs, testing strategy, persona/quantifier docs, SillyTavern references
│   ├── reviews/           # Architectural reviews (holistic, defensive, agent-scalability)
│   ├── CHANGELOG.md
│   └── ROADMAP.md
├── data/
│   ├── characters/        # Character configs per world
│   ├── images/            # Character sprites, headshots, room images
│   ├── personas/          # Player persona configs
│   ├── schemas/           # JSON schemas (character, map, settings, world)
│   ├── settings.json      # Default settings
│   └── worlds/            # World and map JSON configs
└── scripts/               # Python helpers
    ├── build.py           # Full validation (fmt + clippy + tests + coverage)
    ├── check_test_structure.py
    ├── coverage_summary.py
    ├── diagnostic_benchmark.py
    ├── extract_images.py
    ├── extract_sillytavern_png.py
    ├── generate_docs_index.py
    ├── install_git_hooks.py
    ├── kimi_hook_wrapper.py
    ├── parse_coverage.py
    ├── refine_character_json.py
    └── validate_data.py
```

## Windows Development Environment

The development environment is Windows, not Linux. The Chronicler Engine is NOT being developed inside a Linux devcontainer. 

## OH MY PI/c

Fork of Pi by @mariozechner

The most capable agent surface that ships. Continuously tuned by real-world use — complete out of the box, open all the way down.

https://github.com/can1357/oh-my-pi
D:\John\Git\oh-my-pi

## YOUR RESPONSIBILITY 

You are responsible for the overall health of the Chronicler Engine. It is more important that the repository is healthy and working (e.g. the build passes) than your specific task succeeded. For example, you should not arbitrarily delete or revert unknown or unexpected files (especially untracked file) simply because they are not working or otherwise in the way of your specific task.

## DOCUMENTATION STRATEGY: SEMANTIC MAPPING
This project follows a **Spec-Driven Implementation** (SDI) strategy.

### The Golden Rule: Spec-First
**NEVER** implement a new technical system or narrative logic without first creating/updating its specification in `docs/`. The code must reflect the spec, not the other way around.

### Core Principles
1. **Naming as Documentation**: Symbols (functions, types, variables) must use verbose, domain-aligned names that map 1-to-1 with concepts in the `docs/`.
2. **Doc Anchors**: Complex logic blocks are marked with `// [DOC: docs/path/to/file.md]`.
3. **Lean Code**: Remove all "What" comments. If the code isn't clear, rename the symbols.
4. **The "Why" Exception**: Comments are reserved ONLY for technical constraints (e.g., `// Workaround for Axum timeout issue`).
5. **Be Consise**: Be extremely concise. Sacrifice grammar for the sake of concision. 

## THE TEST-FIRST PHILOSOPHY
This project relies on a comprehensive suite of integration tests as the ultimate source of truth for behavior.
- **Tests as Documentation**: If you don't understand how a component works, read its tests in `tests/` before reading the source code.
- **Test-Driven Debugging**: Before fixing a bug, find or create a failing test case. If tests pass but the bug exists, the test suite is missing a scenario.
- **No Regression**: Every code change must pass `python build.py` before task/plan completion. *During development*, iterate with the specific tool (e.g. `cargo clippy` for lint fixes, `cargo nextest run <pattern>` for test fixes). Run `build.py` only for final verification.

### TEST FAILURE HANDLING

When tests fail, you MUST:
1. **Show the actual test output** - quote the failure message verbatim
2. **Read the test code** - understand what the test is actually checking before explaining why it failed
3. **Verify your assumptions** - if you claim "this test skips when X is missing", verify X is actually missing and the skip logic exists
4. **Never rationalize failures away** - a test failure is a real signal that requires investigation, not dismissal
5. **Investigate pre-existing test failures and flaky tests** - Even if a test seems unrelated to your changes, check it anyway, as often it is related. And even if it is unrelated, failing tests need to be fixed regardless. 

If you're unsure why a test failed, say so and investigate - don't invent explanations.

You should avoid **analysis paralysis**, that is, spending excessively large amounts of time trying to reason through a problem without ever coming to any conclusion or doing any action. You should read, run, update or write new tests if you are struggling to understand a problem. Or if that doesn't help, check the UI directly via the browser, or to add logging or other diagnostics in the production code.

## PLANNING REQUIREMENTS

When creating or updating a plan for chronicler_engine work (via any planning skill), the plan **must** include these steps explicitly:

1. **Architecture doc update** — Update `docs/architecture/system.md` (and relevant `docs/system/*.md`) **before** writing code. The code must reflect the spec, not the other way around.
2. **Test-first** — Write a failing test or update existing tests **before** implementing the fix/feature. Every task must have a verification step that includes running tests.
3. **Guardrail compliance** — Verify the change won't violate existing guardrails (clippy lints, arch-lint rules, max file size limits). Run `cargo clippy` and `cargo nextest run <relevant_test>` during development, not just at the end.
4. **Build validation** — Final validation with `python build.py` must pass before the task is considered complete.
5. **Plan archive** — Move completed plans to `docs/plans/archived/` and update `CHANGELOG.md`.

**Plan Adherence:** Do not change the plan partway through implementation without explicit user permission. If you encounter a problem not addressed in the current plan, stop and ask before proceeding.

**Why:** Plans that skip these steps result in rework — architecture docs out of sync, missing tests, clippy failures discovered late, and undocumented changes.

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
| Add game feature | `src/engine/` | Action enum, parser, logic, action_processing |
| Game flow / service | `src/engine/game_service/` | actions, context, helpers, retry, service |
| Modify data model | `src/model/` | agent, character, map, scenario, settings, state, trigger, world |
| LLM client / backends | `src/narrative/llm_client.rs`, `src/narrative/llm/` | High-level client + backend impls (deepseek, mock, ollama, openrouter) |
| LLM prompts | `src/narrative/prompt/` | budget, builder, context, sanitize, templates, types |
| Agent system | `src/narrative/agents/` | Registry, trait definitions, quantifier agent |
| Text checking | `src/narrative/text_check/` | Grammar/spelling via harper_backend |
| Trigger system | `src/engine/trigger_eval.rs` | Trigger evaluation, condition checking |
| Web server | `src/server/` | Axum router, WebSocket, HTMX templates |
| HTMX fragments | `src/server/fragments/` | Actions, endpoints, history, renderers |
| Settings UI | `src/server/settings_fragment/` | Settings fragments, handlers, template |
| Bootstrap / startup | `src/bootstrap/` | load, logging, run, scenario, validate |
| CLI args | `src/cli.rs` | Command-line parsing |
| App settings | `src/settings.rs` | Settings management |
| Persistence | `src/storage/` | SQLite db, snapshot storage |
| Shared test helpers | `src/test_support/` | context, fixtures, in_memory_storage |

## CONVENTIONS
- **Doc Anchors**: Always link complex blocks to `docs/` via `// [DOC: docs/path/to/file.md]`
- **LLM backend**: Trait-based (`LlmBackend`), mock via `MockBackend` in tests
- **Validation**: Run `python build.py` before commit (fmt + clippy + tests + guardrails)

## LLM TEST POLICY
- `python build.py` runs the fast suite only. LLM tests are `#[ignore]`d by default.
- When modifying ANY file in `src/narrative/` or changing LLM prompt/parsing behavior,
  you MUST also run `python build.py --llm-only` to verify real LLM integration.

## ANTI-PATTERNS
- **Never** skip architecture/spec update before implementing engine changes.
- **Never** continue previous reasoning after user says stop, wait, nevermind, or asks a direct question. Halt immediately and answer directly.
- **Never** defend existing architecture as a reason to keep complicated code. If a simpler approach exists, propose it.

## GUARDRAILS (PROGRAMMATIC ENFORCEMENT)

Conventions above are **not advisory** — they are enforced automatically.
If AI-generated code violates these rules, the build will fail.

### Layer 1: Clippy (Compile-Time)
`src/lib.rs` declares `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::dbg_macro, clippy::todo, clippy::unimplemented, clippy::print_stdout, clippy::print_stderr)]`.
Test code is exempt via `#![cfg_attr(test, allow(...))]`.
Binary code (`main.rs`) is exempt for CLI bootstrap only.

### Layer 2: arch-lint (Test-Time)
`tests/architecture.rs` runs `arch_lint::check!()` against `arch-lint.toml`.


## DOCUMENTATION INDEX
`docs/README.md` is **auto-generated**. Do not edit the file list inside the `<!-- AUTO-INDEX -->` block manually.

To regenerate the index after adding, removing, or renaming docs:
```bash
python scripts/generate_docs_index.py
```

To install the git pre-commit hook (regenerates index before every commit):
```bash
python scripts/install_git_hooks.py
```

### Kimi Code CLI Hook (Optional)
Add to `~/.kimi/config.toml` to refresh the index at session start:
```toml
[[hooks]]
event = "SessionStart"
command = "python /absolute/path/to/chronicler_engine/scripts/kimi_hook_wrapper.py"
timeout = 10
```

## COMMANDS

### Iteration (use these while fixing)
```bash
cargo fmt                                       # Check formatting
cargo clippy --all-targets -- -D warnings       # ~10s — fix warnings here
cargo nextest run <test_name>                          # Run one test or pattern
cargo nextest run --tests                              # Run integration test suite (~2–3 min)
cargo run -- --world redmist_estate --port 3000 # Run the server
```

### Final Validation (run once before considering done)
```bash
python build.py             # Full gate: fmt + clippy + guardrails + tests
python build.py --release   # Release build + package
```

## DEVELOPMENT LOOP

When fixing a known failure (e.g. clippy warning, single test):
1. Run only the failing tool until green.
2. Then run `python build.py` once to confirm nothing else broke.

❌ Inefficient: `build.py` → fix one line → `build.py` → fix one line → `build.py`  
✅ Efficient: `cargo clippy` → fix all warnings → `python build.py` (once)

For UI bugs or single test failures, use `cargo nextest run <pattern>` or `cargo check` repeatedly. Run `python build.py` only for final validation.

## CONCURRENT BUILDS
Multiple KimiCode agents building simultaneously can conflict because:
- `cargo fmt` rewrites source files in-place
- `target/` is shared, causing cargo lock contention

Use the concurrent-safe flags for secondary agents:
```bash
# Primary agent — normal build
python build.py

# Secondary agent — isolated target, skip fmt
python build.py --target-dir target/agent2 --no-fmt

# Secondary agent — coverage review (used by /test-police skill)
python build.py --coverage --target-dir target/test_police --no-fmt
```

`build.py` checks if the target directory is locked by another cargo process and prints a warning if so.

To clean up lingering processes and build artifacts:
```bash
python build.py --cleanup
python build.py --cleanup --target-dir target/test_police
```

Tests are already concurrency-safe: they allocate ports dynamically from the range 3010-3050 using file-based locking (`get_available_port` in `tests/test_utils.rs`).

## CODE QUALITY

- Keep answers short and concise
- Do not preserve backward compatibility unless the user asks for it.
- Read files in full before wide-ranging changes, before editing files you have not fully inspected, and when asked to investigate or audit. Do not rely on search snippets for broad changes.
- Technical prose only, be direct
- When the user asks a question, answer it first before making edits or running implementation commands.
- When responding to user feedback or an analysis, explicitly say whether you agree or disagree before saying what you changed.
- For UI changes, verify in the browser with a screenshot before claiming completion.