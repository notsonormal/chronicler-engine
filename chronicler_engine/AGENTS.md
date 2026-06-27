# Chronicler Engine Knowledge Base

**Generated:** 2026-05-10
**Language:** Rust (Edition 2024)
**Type:** Single crate (binary + library)

## OVERVIEW
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## STRUCTURE
<!-- AUTO-STRUCTURE START -->
- **src/**
  - `cli.rs` — Command-line interface definitions
  - `error.rs` — Error types and result aliases
  - `settings.rs` — Application settings and configuration
  - **application/**
    - `application_service.rs` — Main application service coordinating game operations
    - `context.rs` — Application context and state management
    - `game_lifecycle.rs` — Game lifecycle management and state transitions
    - `game_service.rs` — Game service handling gameplay operations
    - `message_editing.rs` — Message editing and modification utilities
    - `query_handlers.rs` — Read-only data access for game state and debug views
    - **action_pipeline/**
      - `actions.rs` — Action enum and action processing types
      - `mod.rs` — Action pipeline for processing game actions
      - `phases.rs` — Phase implementations for the action pipeline
      - `pipeline.rs` — Action pipeline orchestration and execution
      - `retry.rs` — Retry logic for action pipeline operations
  - **bootstrap/**
    - `init_game.rs` — Game state initialization and arrival narration spawning
    - `load.rs` — Game data seeding and initialization routines
    - `logging.rs` — Logging setup and configuration
    - `run.rs` — Main entry point and runtime execution
    - `scenario.rs` — Scenario injection and initialization
    - `state.rs` — Bootstrap game state from saved snapshots
    - `validate.rs` — Data validation utilities
  - **engine/**
    - `action.rs` — Action enum and semantic command types
    - `action_processing.rs` — Action execution pipeline and validation
    - `logic.rs` — Game logic and rule evaluation
    - `parser.rs` — Parser for game data formats
    - `state_diagnostics.rs` — State diagnostics and debugging utilities
    - `trigger_eval.rs` — Trigger evaluation and condition checking
  - **model/**
    - `agent.rs` — Agent definitions and behavior types
    - `character.rs` — Character sheet data and NPC card definitions
    - `game.rs` — Game state and session management
    - `llm_backend.rs` — LLM backend provider types
    - `llm_message.rs` — LLM message formatting and structure
    - `map.rs` — Map and location data structures
    - `message.rs` — Message types and conversation history
    - `message_history.rs` — Message history tracking
    - `prompt_preset.rs` — Prompt preset configurations
    - `quantifier.rs` — Quantifier types for narrative evaluation
    - `scenario.rs` — Scenario definitions and world data
    - `settings.rs` — Settings and configuration types
    - `state_snapshot.rs` — State snapshot serialization
    - `template.rs` — Template placeholder substitution for author-controlled text fields.
    - `trigger.rs` — Trigger conditions and event types
    - `world.rs` — World model definitions
    - **state/**
      - `game_state.rs` — Main game state and builder
      - `generation_status.rs` — Generation status enums and input buffer
      - `message_types.rs` — Message type and entry definitions
      - `mod.rs` — Game state representations (re-exports from submodules)
      - `movement.rs` — Player movement state
      - `narrative_state.rs` — Narrative state with history and input buffer
      - `scene_state.rs` — Current scene NPCs and quantifier confidence
      - `trigger_context.rs` — Stored trigger snapshot context
  - **narrative/**
    - **agents/**
      - `mod.rs` — Agent registry and trait definitions
      - `registry.rs` — Runtime agent lookup and lifecycle
      - `trait_def.rs` — Agent trait definitions
      - **quantifier/**
        - `agent.rs` — Quantifier agent implementation
        - `mod.rs` — Quantifier agent system
        - `orchestration.rs` — Quantifier orchestration
        - `parser.rs` — Quantifier output parsing
        - `prompt.rs` — Quantifier prompt construction
        - `test_support.rs` — Quantifier test utilities
        - `types.rs` — Quantifier type definitions
    - **llm/**
      - `backend.rs` — LLM backend abstraction
      - `deepseek.rs` — DeepSeek LLM provider
      - `mock.rs` — Mock LLM provider for testing
      - `mod.rs` — LLM provider implementations
      - `ollama.rs` — Ollama LLM provider
      - `openrouter.rs` — OpenRouter LLM provider
      - `sanitize.rs` — LLM input/output sanitization
    - **llm_client/**
      - `client.rs` — LLM client implementation
      - `mod.rs` — LLM client interface
      - `request.rs` — LLM request building
      - `response.rs` — LLM response parsing
    - **prompt/**
      - `assembler.rs` — Multi-stage prompt builder
      - `budget.rs` — Token budget management
      - `context.rs` — Prompt context building
      - `mod.rs` — Prompt construction orchestration
      - `types.rs` — Prompt type definitions
    - **text_check/**
      - `check.rs` — Text check execution
      - `harper_backend.rs` — Harper text check backend
      - `mod.rs` — Text checking and validation
      - `types.rs` — Text check type definitions
  - **server/**
    - `app_state.rs` — Application state management
    - `debug.rs` — Debug utilities and endpoints
    - `handlers.rs` — Core HTTP request routing and handling
    - `port_utils.rs` — Port management utilities
    - `router.rs` — Router configuration
    - `server_impl.rs` — Server implementation
    - `templates.rs` — Template rendering utilities
    - `view_models.rs` — View models decouple templates from domain types.
    - **fragments/**
      - `actions.rs` — Action fragment handlers
      - `endpoints.rs` — Fragment endpoints
      - `generation_guard.rs` — Generation guard logic
      - `history.rs` — History fragment handlers
      - `mod.rs` — UI fragment modules
      - **misc/**
        - `game_control.rs` — Game control fragment handlers
        - `mod.rs` — Miscellaneous fragment utilities (re-exports from submodules)
        - `swipe.rs` — Swipe and retry fragment handlers
        - `text_check.rs` — Text check fragment handler
      - **renderers/**
        - `fragment_renderers.rs` — Fragment renderers
        - `mod.rs` — Fragment renderers (re-exports from submodules)
        - `response.rs` — HTTP response helpers
    - **games_fragment/**
      - `handlers.rs` — Games fragment handlers
      - `mod.rs` — Games fragment module
      - `template.rs` — Games templates
    - **prompt_presets_fragment/**
      - `fragments.rs` — Prompt preset fragments
      - `handlers.rs` — Prompt preset handlers
      - `mod.rs` — Prompt presets fragment module
      - `template.rs` — Prompt preset templates
    - **settings_fragment/**
      - `fragments.rs` — Settings fragments
      - `handlers.rs` — Settings handlers
      - `mod.rs` — Settings fragment module
      - `template.rs` — Settings templates
    - **worlds_fragment/**
      - `fragments.rs` — Worlds fragment renderers
      - `handlers.rs` — Worlds management handlers
      - `mod.rs` — Worlds management fragment module
      - `template.rs` — Worlds templates
  - **storage/**
    - `db.rs` — SQLite database connection pool and migrations
    - **backend/**
      - `characters.rs` — Character storage backend operations
      - `core.rs` — Storage backend trait and core abstractions
      - `games.rs` — Game storage operations
      - `helpers.rs` — Shared helper functions for storage backend operations
      - `llm_messages.rs` — LLM message storage
      - `messages.rs` — Message storage operations
      - `mod.rs` — Storage backend modules
      - `personas.rs` — Persona storage operations
      - `presets.rs` — Preset storage operations
      - `settings.rs` — Settings storage operations
      - `snapshots.rs` — Snapshot storage operations
      - `swipes.rs` — Swipe data storage
      - `worlds.rs` — World storage backend operations
    - **mappers/**
      - `llm_message.rs` — LLM message mapper
      - `message.rs` — Message mapper
      - `mod.rs` — Row-to-domain object mapping
      - `state_snapshot.rs` — State snapshot mapper
    - **models/**
      - `character.rs` — Character database model
      - `game.rs` — Game database model
      - `game_state_snapshot.rs` — Game state snapshot model
      - `llm_message.rs` — LLM message database model
      - `message.rs` — Message database model
      - `mod.rs` — Database schema entity definitions
      - `persona.rs` — Persona database model
      - `prompt_preset.rs` — Prompt preset model
      - `settings.rs` — Settings database model
      - `world.rs` — Database row structs for world and map tables
- **scripts/**
  - `build.py` — Full build, validate, and test for Chronicler Engine.
  - `check_python_docstrings.py` — Summary
  - `check_test_structure.py` — No summary
  - `coverage_summary.py` — No summary
  - `diagnostic_benchmark.py` — No summary
  - `extract_images.py` — Extract and process images from SillyTavern character cards (original + cropped versions).
  - `extract_sillytavern_png.py` — Extract embedded PNG images from SillyTavern character cards.
  - `generate_docs_index.py` — Generate an auto-updating index for chronicler_engine/docs/README.md.
  - `generate_structure_index.py` — Generate AGENTS.md structure index from module summaries.
  - `healthcheck.py` — Chronicler Engine healthcheck dispatcher.
  - `install_git_hooks.py` — No summary
  - `parse_coverage.py` — Parse coverage report from cargo-llvm-cov JSON output.
  - `refine_character_json.py` — No summary
  - `validate_data.py` — No summary
<!-- AUTO-STRUCTURE END -->

## Windows Development Environment

The development environment is Windows, not Linux. The Chronicler Engine is NOT being developed inside a Linux devcontainer. 

## YOUR RESPONSIBILITY 

You are responsible for the overall health of the Chronicler Engine. It is more important that the repository is healthy and working (e.g. the build passes) than your specific task succeeded. For example, you should not arbitrarily delete or revert unknown or unexpected files (especially untracked file) simply because they are not working or otherwise in the way of your specific task.

## DOCUMENTATION STRATEGY: SEMANTIC MAPPING
This project follows a **Spec-Driven Implementation** (SDI) strategy.

### The Golden Rule: Spec-First
**NEVER** implement a new technical system or narrative logic without first creating/updating its specification in `docs/`. The code must reflect the spec, not the other way around.

### Core Principles
1. **Naming as Documentation**: Symbols (functions, types, variables) must use verbose, domain-aligned names that map 1-to-1 with concepts in the `docs/`.
2. **Module-Level Two-Line Headers**: Every file in `src/` has:
   - Line 1: `//! [DOC: docs/path/to/domain-doc.md]` (links to domain documentation)
   - Line 2: `//! Human-readable summary` (used for auto-generating STRUCTURE section)
   Function-level anchors removed.
3. **Lean Code**: Remove all "What" comments. If the code isn't clear, rename the symbols.
4. **The "Why" Exception**: Comments are reserved ONLY for technical constraints (e.g., `// Workaround for Axum timeout issue`).
5. **Be Consise**: Be extremely concise. Sacrifice grammar for the sake of concision. 

## THE TEST-FIRST PHILOSOPHY
This project relies on a comprehensive suite of integration tests as the ultimate source of truth for behavior.
- **Tests as Documentation**: If you don't understand how a component works, read its tests in `tests/` before reading the source code.
- **Test-Driven Debugging**: Before fixing a bug, find or create a failing test case. If tests pass but the bug exists, the test suite is missing a scenario.
- **No Regression**: Every code change must pass `python build.py` before task/plan completion. *During development*, iterate with the specific tool (e.g. `cargo clippy` for lint fixes, `cargo nextest run <pattern>` for test fixes). Run `build.py` only for final verification.

Unit tests go in the `src/` folder beside the class they are testing (e.g. `production_class.rs` -> `production_class_test.rs`).

Integration tests go in the `test/` folder.

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

## CONVENTIONS
- **Module-Level DOC Anchors**: Every `src/` file has `//! [DOC: ...]` on line 1 pointing to domain-specific docs. Remove function-level `/// [DOC:` and `// [DOC:` comments.
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
Multiple agents building simultaneously can conflict because:
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

## DOING CODE REVIEWS

Do not run try to compile, build or test the code when doing code reviews. Unless the review explictly calls for it (e.g. the `test-police` review). 