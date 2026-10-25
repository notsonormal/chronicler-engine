# Chronicler Engine Knowledge Base

## Overview
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## Structure
<!-- AUTO-STRUCTURE START -->
- **src/**
  - `error.rs` — Error types and result aliases
  - **adapters/**
    - **driven/**
      - `mod.rs` — Driven adapters: outbound external systems (storage, LLM providers, text check)
      - **llm/**
        - `mod.rs` — LLM driven adapters: provider implementations and HTTP transport
        - **providers/**
          - `deepseek.rs` — DeepSeek LLM provider
          - `mock.rs` — Mock LLM provider for testing
          - `mod.rs` — LLM provider implementations
          - `ollama.rs` — Ollama LLM provider
          - `openrouter.rs` — OpenRouter LLM provider
        - **transport/**
          - `mod.rs` — LLM client interface
          - **utils/**
            - `client.rs` — LLM client implementation
            - `mod.rs` — LLM transport implementation helpers.
            - `request.rs` — LLM request building
            - `response.rs` — LLM response parsing
      - **storage/**
        - `changes.rs` — The change counters: one number for each kind of data, raised after the data is committed
        - `characters.rs` — Character storage backend operations
        - `core.rs` — Storage backend and core abstractions
        - `db.rs` — SQLite database connection pool
        - `docks.rs` — The dock record reads: the offered set and the Options failure of one Game
        - `games.rs` — Game storage operations
        - `llm_messages.rs` — LLM message storage
        - `mod.rs` — Storage layer and database access
        - `personas.rs` — Persona storage operations
        - `presets.rs` — Preset storage operations
        - `settings.rs` — Settings storage operations
        - `stories.rs` — The story reads: the end of the story, one Swipe state, and the entries of a window
        - `test_support.rs` — Test infrastructure types for storage failure injection
        - `transaction.rs` — The transaction: one unit of record writes that lands whole
        - `worlds.rs` — World storage backend operations
        - **mappers/**
          - `llm_message.rs` — LLM message mapper
          - `message.rs` — Message mapper
          - `mod.rs` — Row-to-domain object mapping
        - **models/**
          - `character.rs` — Character database model
          - `game.rs` — Game database model
          - `llm_message.rs` — LLM message database model
          - `map.rs` — Database row struct for the `maps` table
          - `message.rs` — Database row struct for the `messages` table
          - `mod.rs` — Database schema entity definitions
          - `persona.rs` — Persona database model
          - `prompt_preset.rs` — Prompt preset model
          - `settings.rs` — Settings database model
          - `swipe.rs` — Database row struct for the `swipes` table
          - `world.rs` — Database row struct for the `worlds` table
        - **utils/**
          - `mod.rs` — Storage-layer plumbing utilities (datetime parsing, schema migrations).
          - `plumbing.rs` — Storage-layer plumbing utilities (datetime parsing, schema migrations).
          - `records.rs` — Shared helper of the story records: the check that a write names a Message of its Game
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
        - `app_state.rs` — Application state and HTTP fragment rendering.
        - `error.rs` — HTTP driving adapter — maps application `ApplicationError` to axum `Response`.
        - `mod.rs` — HTTP server and API endpoints
        - `templates.rs` — Template rendering utilities
        - `view_models.rs` — View models decouple templates from domain types.
        - **action/**
          - `mod.rs` — Action route handlers.
          - **handlers/**
            - `actions.rs` — Action fragment handlers
            - `mod.rs` — Action route handlers.
        - **autoplay/**
          - `mod.rs` — Autoplay route handlers.
          - **handlers/**
            - `autoplay.rs` — Autoplay start handler
            - `mod.rs` — Autoplay route handlers.
        - **bootstrap/**
          - `mod.rs` — HTTP bootstrap — server bring-up
          - `server.rs` — Server implementation
        - **builders/**
          - `forms.rs` — Form field HTML builders.
          - `headers.rs` — Header fragment and Settings role-health cell builders.
          - `mod.rs` — HTTP builders — composition fns that assemble HTML, headers, and routes.
          - `presets.rs` — Prompt-preset card + form HTML builders.
          - `router.rs` — HTTP router composition.
        - **chat_window/**
          - `mod.rs` — Chat window HTTP handlers.
          - **handlers/**
            - `chat_window.rs` — Chat window HTTP request handlers.
            - `mod.rs` — Chat window route handlers.
        - **debug/**
          - `mod.rs` — Debug HTTP handlers.
          - **handlers/**
            - `debug.rs` — Debug utilities and endpoints
            - `mod.rs` — Debug route handlers.
        - **games/**
          - `mod.rs` — Games module.
          - **handlers/**
            - `games.rs` — Games fragment handlers
            - `mod.rs` — Games route handlers.
          - **templates/**
            - `games.rs` — Games templates
            - `mod.rs` — Games askama templates.
        - **history/**
          - `mod.rs` — History route handlers.
          - **handlers/**
            - `history.rs` — History fragment handlers
            - `mod.rs` — History route handlers.
        - **layout/**
          - `mod.rs` — Layout module (route handlers).
          - **handlers/**
            - `endpoints.rs` — Fragment endpoints
            - `mod.rs` — Layout route handlers (partials + status endpoints).
        - **prompt_presets/**
          - `mod.rs` — Prompt presets module.
          - **handlers/**
            - `mod.rs` — Prompt presets route handlers.
            - `prompt_presets.rs` — Prompt preset handlers
          - **templates/**
            - `mod.rs` — Prompt presets askama templates.
            - `prompt_presets.rs` — Prompt preset templates
        - **settings/**
          - `mod.rs` — Settings module.
          - **handlers/**
            - `mod.rs` — Settings route handlers.
            - `settings.rs` — Settings handlers
          - **templates/**
            - `mod.rs` — Settings askama templates.
            - `settings.rs` — Settings templates
        - **utils/**
          - `current_game.rs` — `CurrentGame` — the request's Game, read once per request.
          - `error.rs` — Error rendering helpers for HTTP fragments.
          - `fragment.rs` — Fragment-rendering glue: the try-render wrapper for AppState renderers and the template render helpers.
          - `handler_helpers.rs` — Handler-level utilities: option string and storage-id helpers.
          - `mod.rs` — HTTP utility modules.
          - `response.rs` — HTTP response helpers
          - `template_helpers.rs` — Shared template rendering helpers: the Askama renderer for `<option>`
          - `view_mappers.rs` — Domain → view aggregators used by HTTP handlers. Distinct from `mappers/`, which convert DB rows ↔ domain.
          - `view_models.rs` — View-model helpers shared between template code and tests.
        - **worlds/**
          - `mod.rs` — Worlds module.
          - **handlers/**
            - `mod.rs` — Worlds route handlers.
            - `worlds.rs` — Worlds management handlers
          - **templates/**
            - `mod.rs` — Worlds askama templates.
            - `worlds.rs` — Worlds templates
  - **application/**
    - `connection_test_service.rs` — Connection test service — one short provider call that records nothing.
    - `errors.rs` — ApplicationError — the error envelope of the application services.
    - `llm_recorder.rs` — LLM call orchestrator - owns the LLM message save + postprocessing
    - `persona_catalogue.rs` — Persona catalogue — persona read-side orchestration at the application layer.
    - `prompt_preset_service.rs` — Prompt preset service — prompt preset persistence orchestration at the application layer.
    - `settings_service.rs` — Settings service — settings persistence orchestration at the application layer.
    - `text_check_service.rs` — TextCheckService orchestrator for text checking
    - `world_catalogue.rs` — World catalogue — world CRUD orchestration at the application layer.
    - **agents/**
      - `mod.rs` — Agent hooks: the hook traits and the AgentRunner
      - `runner.rs` — Agent runner: the Agents of each hook, built from their configuration
      - `trait_def.rs` — Agent hook traits: one trait per hook, each giving its own kind of result
      - **options/**
        - `agent.rs` — Options agent implementation.
        - `mod.rs` — Options agent system — generates the Options to pick from.
        - `prompt.rs` — Options prompt construction
        - `types.rs` — Options agent type definitions
        - **utils/**
          - `mod.rs` — Options agent shared helpers
          - `orchestration.rs` — Options orchestration — LLM call + result processing + entry point.
          - `parser.rs` — Options output parsing — accepts both seeded prompt shapes.
      - **quantifier/**
        - `agent.rs` — Quantifier agent implementation.
        - `mod.rs` — Quantifier agent system
        - `prompt.rs` — Quantifier prompt construction
        - `types.rs` — Quantifier type definitions
        - **utils/**
          - `mod.rs` — Quantifier utility modules.
          - `orchestration.rs` — Quantifier orchestration — LLM call + result processing + entry point.
          - `parser.rs` — Quantifier response parsing helpers.
    - **autoplay/**
      - `budget.rs` — The Budget of an Autoplay run: the number of cycles it plays
      - `driver.rs` — The Autoplay loop: one blocking task that runs the Jobs of one run, one after the other.
      - `mod.rs` — Autoplay: the loop of ordinary Jobs that plays the Persona while it holds the Game.
      - `pause.rs` — The test pause point of the Autoplay loop: a hold that stops the loop before an accept
    - **debug/**
      - `dto.rs` — DebugStateView — debug-state DTO for the HTTP `/debug/state` endpoint.
      - `mod.rs` — Debug DTOs for the HTTP `/debug/state` endpoint.
    - **games/**
      - `active.rs` — ActiveGame — the active-Game pointer of the server process.
      - `catalogue.rs` — GameCatalogue — game-lifecycle storage orchestration.
      - `mod.rs` — Game lifecycle orchestration and read-side queries.
      - `view_query.rs` — GameViewQuery — read-side queries that don't mutate game state.
    - **generation/**
      - `gate.rs` — GameWriter and JobWriter — the writer rule and the guard: the one registry of the busy state, and the transactions a Job writes through.
      - `job_tasks.rs` — The watch on the tasks a Job spawns, so a tier-1 test waits for a Job to end
      - `mod.rs` — The writer rule and the guard: the one registry of the busy state: one holder per busy Game.
    - **narration/**
      - `mod.rs` — Narration: the generation a Job's narration call makes
      - `narrator.rs` — The narrator: the assembly and the model call a Job's narration needs
    - **pipeline/**
      - `job.rs` — The Job: the plan of one request, its generations, and the steps of a generation
      - `mod.rs` — The Job of one request.
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
    - `init_game.rs` — Game state initialization and game resolution
    - `load.rs` — Game data seeding and initialization routines
    - `logging.rs` — Logging setup and configuration
    - `run.rs` — Main entry point and runtime execution
    - `wiring.rs` — Composition root for application orchestrators
  - **domain/**
    - **model/**
      - `action.rs` — Action enum and semantic command types (slash-command steering entry)
      - `agent.rs` — Agent definitions and behavior types
      - `change.rs` — The change kinds the server counts, and one snapshot of their numbers
      - `character.rs` — Character sheet data and Character card definitions
      - `dock.rs` — The dock record: the offered set and the one Options failure of a Game
      - `game.rs` — Game state and Game management
      - `generation.rs` — The order of one generation, and the plan rules that turn one request into a plan
      - `llm_backend.rs` — LLM backend provider types
      - `llm_message.rs` — LLM message DTO: the stored record of one model call
      - `map.rs` — Map and Room data structures
      - `message.rs` — Message types and conversation history (Message, Swipe)
      - `mod.rs` — Core data models and domain types
      - `prompt_preset.rs` — Prompt preset configurations
      - `quantifier.rs` — The Quantifier result and the Character transitions the Game rules derive from it
      - `scenario.rs` — Scenario definitions and world data
      - `settings.rs` — Settings and configuration types
      - `story_window.rs` — The story window: the newest entries of the story, with the room each one shows, and the end-of-story facts that go with them
      - `tail.rs` — The end of the story, and the rules derived from it
      - `template.rs` — Template placeholder substitution for author-controlled text fields.
      - `trigger.rs` — Trigger conditions and event types
      - `world.rs` — World model definitions
      - **state/**
        - `event_record.rs` — The Trigger reference of an event record, and the event record of a fired Trigger
        - `generation_status.rs` — Generation status enums: the step and the status are independent axes; the Job in `application/pipeline/` drives them.
        - `message_types.rs` — Message type and entry definitions
        - `mod.rs` — Game state representations (submodule declarations)
        - `movement.rs` — Player movement state
        - `scene_state.rs` — The Characters present in one Swipe state
        - `swipe_state.rs` — The game state of one Swipe: movement, the Characters present, encounters and the event record
      - **utils/**
        - `game_name.rs` — Game name generation with date-based disambiguation.
        - `game_rules.rs` — The game rules: movement, the Characters present and the Triggers, as a pure function of the Swipe state
        - `mod.rs` — Domain model utility modules.
        - `name_uniqueness.rs` — Trim-and-compare name uniqueness shared by the settings/preset services and preset storage.
        - `opening.rs` — The opening Narration of a new Game and the parts only it has.
        - `scenario_defaults.rs` — Serde default-fn-pointers for StartingScenario fields.
        - `settings_defaults.rs` — Serde default-fn-pointers for `AppSettings` fields. Cannot become methods — `#[serde(default = "...")]` requires a fn path.
        - `world_defaults.rs` — Serde default-fn-pointers for WorldManifest fields.
        - `xml.rs` — XML string formatting utilities.
  - **test_support/**
    - `chat_completions_mock.rs` — A one-request chat-completions endpoint on a free local port, for provider tests.
    - `context.rs` — Builds `WiredApp` instances for integration tests.
    - `env_guard.rs` — `ApiKeyEnvGuard` — serializes tests that mutate API-key environment variables.
    - `fixtures.rs` — Test fixtures shared between unit and integration tests.
    - `http.rs` — HTTP response helpers for unit tests that call handlers directly.
    - `quantifier.rs` — Quantifier test utilities
    - `scripted_model.rs` — `ScriptedModel`: the strict, holdable LLM port fake for tier-1 tests.
    - `test_app_builder.rs` — Test application builder for HTTP and integration tests.
    - `test_data_builder.rs` — Test data bundle builder for integration tests.
  - **utils/**
    - `cli.rs` — Command-line interface definitions
- `build.py` — Full build, validate, and test for Chronicler Engine.
- **scripts/**
  - `build_slot.py` — Machine-wide build slot: one cargo compile or test step at a time across all checkouts.
  - `check_python_docstrings.py` — Scan Python files in scripts/ and scripts/issue_tracker/ for missing module docstrings.
  - `check_test_structure.py` — Enforce unit-test structure rules: no inline test modules, every *_tests.rs registered.
  - `coverage_summary.py` — Print a coverage summary (overall + low-coverage files) from cargo-llvm-cov JSON.
  - `extract_http_routes.py` — Generate `docs/diataxis/reference/frontend/http_routes.md` from `router.rs`.
  - `extract_images.py` — Extract and process images from SillyTavern character cards (original + cropped versions).
  - `extract_sillytavern_png.py` — Extract embedded PNG images from SillyTavern character cards.
  - `generate_docs_index.py` — Generate an auto-updating index for docs/AGENTS.md.
  - `generate_guardrails_doc.py` — Generate `docs/diataxis/reference/coding_standards/guardrails.md` from coding-standards sources.
  - `generate_structure_index.py` — Generate AGENTS.md structure index from module summaries.
  - `generate_tests_structure_index.py` — Generate tests/AGENTS.md structure index from module summaries.
  - `healthcheck.py` — Chronicler Engine healthcheck dispatcher.
  - `install_git_hooks.py` — Install git hooks from scripts/git-hooks/ to .git/hooks/.
  - `parse_coverage.py` — Parse coverage report from cargo-llvm-cov JSON output.
  - `precommit_regenerate.py` — Regenerate the four generated index files for the git pre-commit hook.
  - `prepare_review_bundle.py` — Assemble a review bundle (stat, per-area patches, untracked/deleted lists) under tmp/.
  - `refine_character_json.py` — Split character card descriptions into structured personality/scenario/description fields.
  - `remove_worktrees.py` — Remove linked git worktrees, but only ones that are safe to lose, and prune stale target dirs.
  - `target_seed.py` — Seed a cold cargo target dir with dependency artifacts from a warm sibling checkout.
  - `vale_lint.py` — Vale prose linter wrapper for Chronicler Engine docs.
  - `validate_data.py` — Validate JSON data files against schemas and check cross-file references.
  - `validate_docs.py` — Validate markdown docs + DOC anchors under docs/.
  - `validate_feature_spec.py` — Validate that every scenario in a feature spec has a covering integration test and every annotated test references a declared scenario.
  - `validate_seam_cases.py` — Validate the seam contracts: every test in a declaring file carries one `CASE` tag.
<!-- AUTO-STRUCTURE END -->

## Your Responsibility

You are responsible for the overall health of the Chronicler Engine. It is more important that the repository is healthy and working than your specific task succeeded. For example, you should not arbitrarily delete or revert unknown or unexpected files (especially untracked files) simply because they are not working or otherwise in the way of your specific task, as this could interfere with other work going on in the repository at the same time. 

However, during code reviews you should not make code changes to fix problems.

## Communication

Label epistemic status when it matters: known, inferred, or guessed.

If you don't know something, say "I don't know" instead of inventing an answer.

When responding to user feedback or an analysis, explicitly say whether you agree or disagree before saying what you changed.

### Decision Making

Before implementing (e.g. during planning):

- **State your assumptions explicitly.** If uncertain or something is unclear, stop, name what's confusing, and ask.
- **If an instruction contradicts what you see, say so before acting.** Do not silently work around the mismatch or proceed as if the instruction were accurate.
- **If multiple interpretations exist, present them** — Don't pick silently unless obvious.
- **Hold a reasoned position.** Push back when a simpler approach exists, and if the user pushes back while your reasoning still holds, say why.
- **Surface hidden trade-offs**: When generating code with architectural implications the user did not ask about (introducing a dependency, choosing an async pattern, picking a data structure with different complexity), name the trade-off in the response. Do not bury it.

These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## The Test-First Philosophy

A comprehensive suite of unit and integration tests is the ultimate source of truth for behavior. Read a component's tests in `tests/` before its source; before fixing a bug, find or create a failing test case — and never rationalize a failure away. Integration tests go in `tests/`; the failure-handling protocol lives in `tests/AGENTS.md`.

Avoid **analysis paralysis**: when reasoning stops converging, act instead — read, run, or write a test, check the UI directly in the browser, or add logging and diagnostics to the production code.

## Documentation Index

`docs/AGENTS.md` is an autogenerated catalogue of `docs/diataxis/`; `tests/AGENTS.md` does the same for integration test files. Read them when searching docs or tests. The pre-commit hook regenerates and stages all generated indexes (these two, the Structure section above, and the guardrails doc) on every commit — never hand-edit the generated blocks. Without the hook, regenerate via the `scripts/generate_*_index.py` scripts.

`CODING_STANDARDS.md` holds the implementation, code-comment, code-review and testing rules. Read it before writing or reviewing code.

## Development Loop

`tmp/` is the git-ignored scratch folder.

Every build action goes through `build.py`. Each run writes `logs/build_*.log` and appends a row to `logs/build_history.txt`. Run `build.py` in the foreground, with the `timeout` parameter of the bash tool set to 1200. A piped `tail` of stdout loses the result when the run times out, so read the result from the newest log file:

```bash
tail -n 10 "$(ls -t logs/build_*.log | head -1)"
```

| Command | Covers | Warm cost |
|---|---|---|
| `python build.py clippy` | Formatting check (`cargo fmt --check`), then compile and lint of all targets; warnings are errors | ~12 s |
| `python build.py test-pattern "<pattern>"` | Tests whose name contains the pattern (a substring, not a regex), in every test binary | ~3 s, depends on the pattern |
| `python build.py test-pattern "<pattern>" --test-target <name>` | Tests whose name matches the pattern, in one test binary (`--test <name>`; `--test-target lib` for the lib unit tests). Only that binary is rebuilt | ~6 s after a one-line edit |
| `python build.py integration` | Every test binary except browser, architecture and guardrails; lib unit tests included | ~21 s |
| `python build.py guardrails` | The architecture and guardrails test binaries | ~10 s |
| `python build.py browser` | The browser (Playwright) binary only | ~92 s |
| `python build.py --no-browser` | The full gate without the browser tier | ~50 s |
| `python build.py` | The full gate: installs git hooks, removes safe-to-lose linked worktrees, fmt (rewrites files), data, doc and structure checks, Python tests, clippy, guardrails, integration, browser, packaging | ~145 s |
| `python build.py validate-docs` | Markdown docs and DOC anchors under `docs/`, and the source paths that `AGENTS.md`, `tests/AGENTS.md` and the other root docs name | <1 s |
| `python build.py run` | Builds and runs the dev server | see `ENVIRONMENT.md` |

Warm cost is the median of recent runs in `logs/build_history.txt`, including runs after a Rust edit. A run with no Rust change is faster (`ENVIRONMENT.md`, *Typical gate cost*). A narrow loop with `--test-target` does not compile the other test binaries, so a change to a shared API shows in `clippy` or in the full gate instead.

`python build.py --help` lists every subcommand and flag. `ENVIRONMENT.md`: read it when a build or test run is slow, waits for the build slot, or runs out of memory. It also says where the dev server keeps its DB and reads its data.

## Agent Skills

### Issue tracker

Local markdown under `.scratch/` (per `docs/agents/issue-tracker.md`). No `gh` CLI dependency.

### Triage labels

Five canonical role strings, used as `Status:` lines in local-markdown files (per `docs/agents/triage-labels.md`).

### Domain docs

Single-context: one `CONTEXT.md` glossary at the repo root (per `docs/agents/domain.md`).

## Permissions System

Read `.pi/extensions/pi-permission-system/config.json` to see allowed permissions. Do not circumvent them. You may *recommend* permission changes at the end of a task, but you may not *apply* them without explicit user approval. These restrictions exist to prevent the agent from touching git without supervision.

Don't commit without explicit approval, even if committing is allowed in the permissions config. Normal git commands are fine but destructive git commands should be avoided in general. 

## Subagents and delegation extra rules

Delegate a task only when it is substantial, and do a short task yourself: each new subagent pays again to load the context. A skill that runs several subagents, such as code review or parallel wayfinder tickets, keeps its own split.

`implementer` is the default for implementation work whose brief names the design, the files and the checks. Choose `strong-implementer` when a wrong judgement is expensive:

- the agent must decide questions that the design leaves open;
- the change must keep many parts consistent across layers;
- an `implementer` run on the same work came back wrong on points its brief had already decided.

Give a substantial investigation to a `scout`: work whose result is a finding, such as how a flow works, every place a change must touch, or whether a claim holds, and whose raw reads you will not need again. Name that finding in the brief.
