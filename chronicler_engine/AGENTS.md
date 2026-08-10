# Chronicler Engine Knowledge Base

## OVERVIEW
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## STRUCTURE
<!-- AUTO-STRUCTURE START -->
- **src/**
  - `error.rs` — Error types and result aliases
  - **adapters/**
    - **driven/**
      - `mod.rs` — Driven adapters: outbound external systems (storage, LLM providers, text check)
      - **llm/**
        - `mod.rs` — LLM driven adapters: provider implementations and HTTP transport
      - **storage/**
        - `db.rs` — SQLite database connection pool
        - `mod.rs` — Storage layer and database access
        - `preset_store.rs` — PresetStore newtype — distinguishes preset storage from game storage
      - **text_check/**
        - `harper_text_checker.rs` — Harper text check adapter implementing TextChecker port
        - `mod.rs` — Text checking and validation
      - **utils/**
        - `harper.rs` — Harper text-check adapter boundary helpers.
        - `mock.rs` — Mock LLM provider utilities.
        - `mod.rs` — Driven-adapter shared utilities.
    - **driving/**
      - `mod.rs` — Driving adapters: HTTP and CLI interfaces
      - **http/**
        - `app_state.rs` — Application state management
        - `error.rs` — HTTP driving adapter — maps application `ApplicationError` to axum `Response`.
        - `mod.rs` — HTTP server and API endpoints
        - `templates.rs` — Template rendering utilities
        - `view_models.rs` — View models decouple templates from domain types.
  - **application/**
    - `arrival_service.rs` — Arrival narration use case — generates the opening scene when a player enters a room
    - `errors.rs` — ApplicationError + ProcessActionResult — error envelope and action-result tri-state.
    - `llm_message.rs` — LLM message DTO + recorder save seam
    - `llm_recorder.rs` — LLM call orchestrator - owns forensics save + postprocessing
    - `message_service.rs` — Game-state lifecycle seam for message history and snapshots.
    - `persona_catalogue.rs` — Persona catalogue — persona read-side orchestration at the application layer.
    - `prompt_preset_service.rs` — Prompt preset service — prompt preset persistence orchestration at the application layer.
    - `settings_service.rs` — Settings service — settings persistence orchestration at the application layer.
    - `text_check_service.rs` — TextCheckService orchestrator for text checking
    - `world_catalogue.rs` — World catalogue — world CRUD orchestration at the application layer.
    - **agents/**
      - `mod.rs` — Agent registry and trait definitions
      - `registry.rs` — Runtime agent lookup and lifecycle
      - `trait_def.rs` — Agent trait definitions
      - **quantifier/**
        - `agent.rs` — Quantifier agent implementation.
        - `mod.rs` — Quantifier agent system
        - `prompt.rs` — Quantifier prompt construction
        - `types.rs` — Quantifier type definitions
    - **debug/**
      - `dto.rs` — DebugStateView — debug-state DTO for the HTTP `/debug/state` endpoint.
      - `mod.rs` — Debug DTOs for the HTTP `/debug/state` endpoint.
    - **games/**
      - `catalogue.rs` — GameCatalogue — game-lifecycle storage orchestration.
      - `mod.rs` — Game lifecycle orchestration and read-side queries.
      - `view_query.rs` — GameViewQuery — read-side queries that don't mutate game state.
    - **generation/**
      - `gate.rs` — GenerationGate — per-game slot orchestration. Generation truth is persisted `GenerationStatus` only.
      - `guard.rs` — Generation guard logic
      - `mod.rs` — Generation gating and per-game slot orchestration.
      - `slot.rs` — GenerationSlot — per-game registry slot enum (distinct from domain `GenerationStatus`, which is the pipeline phase).
    - **pipeline/**
      - `mod.rs` — Action pipeline for processing game actions
      - `phase_error.rs` — Canonical phase-level error type for the action pipeline.
      - `phases.rs` — Phase implementations for the action pipeline
      - `pipeline.rs` — Action pipeline orchestration and execution
      - `spawn.rs` — Shared spawn helper for pipeline tasks
    - **ports/**
      - `llm_provider.rs` — LLM provider port (transport-only)
      - `mod.rs` — Application ports: outbound interfaces (driven port traits)
      - `text_checker.rs` — TextChecker port trait and CheckResult DTO
    - **prompting/**
      - `assembler.rs` — Multi-stage prompt assembler: orchestrates layer rendering + context fitting.
      - `mod.rs` — Prompt construction orchestration
      - `prompt_merge.rs` — Prompt merge helper for models that ignore the system role.
      - `sanitize.rs` — LLM input/output sanitization
      - `token_budget.rs` — Token budget management
      - `types.rs` — Prompt type definitions
      - **builders/**
        - `mod.rs` — Narrative prompt builder modules.
        - `sections.rs` — Multi-stage prompt builder.
      - **utils/**
        - `context.rs` — Prompt context fitting — message budget enforcement.
        - `mod.rs` — Narrative prompt utility modules.
  - **bootstrap/**
    - `init_game.rs` — Game state initialization and arrival narration spawning
    - `load.rs` — Game data seeding and initialization routines
    - `logging.rs` — Logging setup and configuration
    - `run.rs` — Main entry point and runtime execution
    - `validate.rs` — Data validation utilities
    - `wiring.rs` — Composition root for application orchestrators
  - **domain/**
    - **model/**
      - `action.rs` — Action enum and semantic command types
      - `agent.rs` — Agent definitions and behavior types
      - `character.rs` — Character sheet data and NPC card definitions
      - `game.rs` — Game state and session management
      - `llm_backend.rs` — LLM backend provider types
      - `map.rs` — Map and location data structures
      - `message.rs` — Message types and conversation history
      - `message_history.rs` — Message history tracking
      - `mod.rs` — Core data models and domain types
      - `prompt_preset.rs` — Prompt preset configurations
      - `quantifier.rs` — Quantifier types for narrative evaluation
      - `scenario.rs` — Scenario definitions and world data
      - `settings.rs` — Settings and configuration types
      - `template.rs` — Template placeholder substitution for author-controlled text fields.
      - `trigger.rs` — Trigger conditions and event types
      - `world.rs` — World model definitions
      - **state/**
        - `game_state.rs` — Main game state and builder
        - `game_state_snapshot.rs` — State snapshot value types (persistable representations of game state).
        - `generation_status.rs` — Generation status enums and input buffer — phase/status are independent axes; live state machine lives in `application/pipeline/pipeline.rs`.
        - `message_types.rs` — Message type and entry definitions
        - `mod.rs` — Game state representations (submodule declarations)
        - `movement.rs` — Player movement state
        - `narrative_state.rs` — Narrative state with history and input buffer
        - `scene_state.rs` — Current scene NPCs and quantifier confidence
        - `trigger_context.rs` — Stored trigger snapshot context
      - **utils/**
        - `game_name.rs` — Game name generation with date-based disambiguation.
        - `mod.rs` — Domain model utility modules.
        - `scenario_defaults.rs` — Serde default-fn-pointers for StartingScenario fields.
        - `settings_defaults.rs` — Serde default-fn-pointers for `AppSettings` fields. Cannot become methods — `#[serde(default = "...")]` requires a fn path.
        - `template.rs` — Template placeholder substitution for author-controlled text fields.
        - `world_defaults.rs` — Serde default-fn-pointers for WorldManifest fields.
  - **test_support/**
    - `context.rs` — Builds `WiredApp` instances for integration tests.
    - `fixtures.rs` — Test fixtures shared between unit and integration tests.
    - `quantifier.rs` — Quantifier test utilities
    - `test_app_builder.rs` — Test application builder for HTTP and integration tests.
    - `test_data_builder.rs` — Test data bundle builder for integration tests.
  - **utils/**
    - `cli.rs` — Command-line interface definitions
    - `settings.rs` — Application settings and configuration
- **scripts/**
  - `build.py` — Full build, validate, and test for Chronicler Engine.
  - `check_python_docstrings.py` — Scan Python files in scripts/ and scripts/issue_tracker/ for missing module docstrings.
  - `check_test_structure.py` — Enforce unit-test structure rules: no inline test modules, every *_tests.rs registered.
  - `coverage_summary.py` — Print a coverage summary (overall + low-coverage files) from cargo-llvm-cov JSON.
  - `diagnostic_benchmark.py` — Run the diagnostic benchmark suite and produce an aggregated markdown/JSON report.
  - `extract_http_routes.py` — Generate `docs/diataxis/reference/frontend/http_routes.md` from `router.rs`.
  - `extract_images.py` — Extract and process images from SillyTavern character cards (original + cropped versions).
  - `extract_sillytavern_png.py` — Extract embedded PNG images from SillyTavern character cards.
  - `generate_docs_index.py` — Generate an auto-updating index for chronicler_engine/docs/AGENTS.md.
  - `generate_structure_index.py` — Generate AGENTS.md structure index from module summaries.
  - `generate_tests_structure_index.py` — Generate tests/AGENTS.md structure index from module summaries.
  - `healthcheck.py` — Chronicler Engine healthcheck dispatcher.
  - `install_git_hooks.py` — Install git hooks from chronicler_engine/scripts/git-hooks/ to .git/hooks/.
  - `parse_coverage.py` — Parse coverage report from cargo-llvm-cov JSON output.
  - `refine_character_json.py` — Split character card descriptions into structured personality/scenario/description fields.
  - `vale_lint.py` — Vale prose linter wrapper for Chronicler Engine docs.
    - `validate_data.py` — Validate JSON data files against schemas and check cross-file references.
  - `validate_docs.py` — Validate markdown docs + DOC anchors under chronicler_engine/.
  - `validate_feature_spec.py` — Validate that every scenario in a feature spec has a covering integration test and every annotated test references a declared scenario.
<!-- AUTO-STRUCTURE END -->

## YOUR RESPONSIBILITY 

You are responsible for the overall health of the Chronicler Engine. It is more important that the repository is healthy and working (e.g. the build passes) than your specific task succeeded. For example, you should not arbitrarily delete or revert unknown or unexpected files (especially untracked file) simply because they are not working or otherwise in the way of your specific task.

**Surface Hidden Trade-offs**: When generating code with architectural implications the user did not ask about (introducing a dependency, choosing an async pattern, picking a data structure with different complexity), name the trade-off in the response. Do not bury it.

**Honest Status Reporting**: When asked "is X done?", answer based on what is verified, not what was attempted. "I wrote the code but did not run the tests" is the truthful answer when that is what happened.

## DOCUMENTATION STRATEGY: SEMANTIC MAPPING

### Core Principles
1. **Naming as Documentation**: Symbols (functions, types, variables) must use verbose, domain-aligned names that map 1-to-1 with concepts in the `docs/`.
2. **Module-Level Two-Line Headers**: Every file in `src/` has:
   - Line 1: `//! [DOC: chronicler_engine/docs/diataxis/reference/<area>/<name>.md]` (links to reference documentation; `reference/` only — no `explanation/`, `how-to/`, or `tutorials/` targets)
   - Line 2: `//! Human-readable summary` (used for auto-generating STRUCTURE section)
   Function-level anchors removed.
3. **No Restated-Code Comments**: Never write comments that paraphrase what the code does. If the code isn't clear, rename the symbols rather than comment. Comments explain the WHY only when non-obvious: a hidden constraint, behavior that would surprise a reader.
4. **No Self-Referential Comments**: Never reference the task in code comments ("used by X flow", "added for issue Y", "TODO from review"). Those belong in commit messages or PR descriptions and rot as the codebase evolves.

## THE TEST-FIRST PHILOSOPHY
This project relies on a comprehensive suite of integration tests as the ultimate source of truth for behavior.
- **Tests as Documentation**: If you don't understand how a component works, read its tests in `tests/` before reading the source code.
- **Test-Driven Debugging**: Before fixing a bug, find or create a failing test case. If tests pass but the bug exists, the test suite is missing a scenario.

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

## LLM TEST POLICY
- `python build.py` runs the fast suite only. LLM tests are `#[ignore]'`d by default.
- When modifying ANY file in `src/application/narrative_prompt/` or `src/adapters/driven/llm/`, or changing LLM prompt/parsing behavior, you MUST also run `python build.py --llm-only` to verify real LLM integration.

## ANTI-PATTERNS
- **Never** continue previous reasoning after user says stop, wait, nevermind, or asks a direct question. Halt immediately and answer directly.
- **Never** defend existing architecture as a reason to keep complicated code. You might need to take a stepback and consider architecture or code holistically.

## DOCUMENTATION INDEX
The file `chronicler_engine/docs/AGENTS.md` holds an autogenerated catalogue of all docs in the `chronicler_engine/docs` folder (excluding docs like `CHANGELOG.md`). This is similar the file list in the `STRUCTURE` section above. Read this when you want to search for or through docs.

The file `chronicler_engine/tests/AGENTS.md` holds an autogenerated catalogue of integration test files, similar to the `STUCTURE` section above. 

To regenerate the index after adding, removing, or renaming docs:
```bash
python chronicler_engine/scripts/generate_docs_index.py
```

This script is automatically called by the pre-commit hook.

To install the git pre-commit hook (regenerates index before every commit):
```bash
python scripts/install_git_hooks.py
```

## DEVELOPMENT LOOP

Temporary files should be written into tmp folders e.g. `tmp` or `chronicler_engine/tmp`.

`build.py` writes logs to both standard output and to the `chronicler_engine/logs` folder. The standard build should take about 1 minute normally. If it times out or fails, check the build logs for failures.

### COMMANDS

#### Iteration (use these while fixing)
```bash
cargo fmt                                       # Check formatting
cargo clippy --all-targets -- -D warnings       # ~10s — fix warnings here
cargo test --lib                                # Run the unit tests
cargo nextest run --test architecture           # Run the architecture tests
cargo nextest run --test guardrails             # Run the guardrails tests
cargo nextest run <test_name>                   # Run one test or pattern
cargo nextest run --tests                       # Run integration test suite (~1–2 min)
cargo run -- --world redmist_estate --port 3000 # Run the server
```

#### Final Validation (run once before considering done)
```bash
python build.py                                 # Full gate: fmt + clippy + guardrails + tests (~1 min)
```

A majority of the time taken by `build.py` is the integration tests. Running the full integration tests just before running the `build.py` is inefficient. Either run targeted `cargo nextest` or skip them and run `build.py` straight away. 

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

- Do not preserve backward compatibility unless the user asks for it.
- Read files in full before wide-ranging changes, before editing files you have not fully inspected, and when asked to investigate or audit. Do not rely on search snippets for broad changes.
- When the user asks a question, answer it first before making edits or running implementation commands.
- When responding to user feedback or an analysis, explicitly say whether you agree or disagree before saying what you changed.

## DOING CODE REVIEWS

Do not run try to compile, build or test the code when doing code reviews. Unless the review explictly calls for it (e.g. the `test-police` review). 

During code reviews, avoid vague architectural critiques and focus on actionable changes.