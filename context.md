# Code Context

## Project Overview

**mrn-general** — Multi-service AI workspace on branch `hexagon-phase2` (commit `e0cd301`).

Contains:
- **chronicler_engine/** — Rust 2024 Edition interactive fiction engine with LLM-driven narrative
- **docker/** — Docker Compose, Caddy proxy, socket proxy
- **scripts/ObsidianVaultExport/** — Python vault export tooling
- **docs/** — Workspace documentation

---

## Files Retrieved

### Root Level
1. `AGENTS.md` (full) — Project-wide agent guidelines, anti-patterns, conventions
2. `CONTEXT-MAP.md` — Maps `chronicler_engine/CONTEXT.md`, `docker/CONTEXT.md`, `scripts/CONTEXT.md`
3. `opencode.json` — Agent config with kimi-for-coding model, MCP servers (browsermcp, openrouter-image)

### Chronicler Engine
4. `chronicler_engine/AGENTS.md` (full) — Detailed engine knowledge:
   - Spec-Driven Implementation (SDI) strategy
   - Test-first philosophy with test mirror convention
   - Full module tree under `src/`
   - Concurrent build safety with `--target-dir` and `--no-fmt` flags

### Docker
5. `docker/docker-compose.yml` — Main compose file
6. `docker/docker-proxy.yml` — Socket proxy config
7. `docker/Caddyfile` — Reverse proxy config
8. `docs/docker.md` — Docker & AI stack architecture docs

### .agents Configuration
9. `.agents/rules/` — chronicler_engine.md, DEBUGGING.md, rust-skills.md, environment.md, rules.md, rust_conventions.md
10. `.agents/skills/` — 60+ skill directories including chronicler-dev-workflow, test-police, rust-*, antipattern-checker

---

## Key Code

### Build Validation Entry Point
```bash
cd chronicler_engine && python build.py   # Full gate: fmt + clippy + tests + coverage
```
Flags: `--release`, `--target-dir`, `--no-fmt`, `--coverage`, `--llm-only`, `--cleanup`

### Docker Commands
```bash
docker compose -f docker/docker-compose.yml up -d
docker compose -f docker/docker-compose.yml down
```

### Rust Source Structure (`chronicler_engine/src/`)
```
src/
├── error.rs                  # Error types
├── settings.rs               # App settings
├── adapters/
│   ├── driven/
│   │   ├── llm/              # LLM provider impls
│   │   ├── storage/          # SQLite, snapshots
│   │   └── text_check/       # HarperTextChecker
│   └── driving/
│       └── http/             # Axum server, handlers, router
├── application/
│   ├── action_pipeline/      # Action processing phases
│   ├── agents/               # Agent registry, quantifier agent
│   ├── narrative_prompt/      # Prompt assembly, budgeting
│   ├── game_service.rs
│   └── text_check_service.rs
├── bootstrap/
│   ├── init_game.rs
│   ├── llm_factory.rs
│   ├── run.rs
│   └── state.rs
└── domain/
    ├── engine/               # Action, parser, logic, triggers
    └── model/                # Game, message, world, scenario, etc.
```

### Test Structure
Integration tests mirror `src/` paths under `tests/integration/` per test binary (integration/http/browser/llm/infrastructure).

---

## Architecture

```
mrn-general/
├── chronicler_engine/     Rust game engine (Edition 2024, Rust 1.85+)
│   ├── src/               Hexagonal architecture: adapters, application, domain, bootstrap
│   ├── tests/             Integration tests mirror src/ paths
│   ├── scripts/           build.py, healthcheck.py, validate_data.py, etc.
│   └── docs/              Architecture docs, ADRs
├── docker/                Docker Compose + Caddy + Socket Proxy
│   ├── docker-compose.yml # Ollama, OpenNotebook, SurrealDB services
│   ├── docker-proxy.yml   # tecnativa/docker-socket-proxy (restricted)
│   └── Caddyfile          # Reverse proxy config
├── scripts/               Python automation
│   └── ObsidianVaultExport/
├── docs/                  Workspace-level docs (env, hardware, models, docker)
├── .agents/
│   ├── rules/             Engine conventions, debugging playbook
│   └── skills/            60+ agent skills (chronicler-dev-workflow, test-police, etc.)
└── opencode.json          Agent config (MCP servers, kimi-for-coding model)
```

### Key Patterns
- **Spec-Driven Implementation**: Architecture docs in `docs/` before code
- **Test Mirror Convention**: Tests live at mirrored paths under `tests/integration/`
- **LLM Backend**: Trait-based (`LlmBackend`), mock via `MockBackend` in tests
- **Docker Proxy**: `host.docker.internal:2375` with CONTAINERS/IMAGES/NETWORKS only

---

## Start Here

**For Rust engine work:** Open `chronicler_engine/AGENTS.md` first — contains full module tree, build commands, development loop guidelines.

**For Docker/AI stack work:** Open `docs/docker.md` — contains architecture diagram and service descriptions.

**For agent skill work:** Open `.agents/rules/chronicler_engine.md` and check relevant skill in `.agents/skills/`.

---

## Supervisor coordination

No coordination needed. Standard scouting complete.

---

## Acceptance Report

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Scouting completed: project structure, key files, build commands, architecture patterns documented in context.md"
    }
  ],
  "changedFiles": [],
  "testsAddedOrUpdated": [],
  "commandsRun": [],
  "validationOutput": [],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "No code changes. Scouting output written to /home/moridin84/projects/mrn-general/context.md",
  "reviewFindings": [
    "no blockers - standard project exploration"
  ],
  "manualNotes": "Project is a multi-service AI workspace with: (1) chronicler_engine - Rust 2024 interactive fiction engine, (2) docker/ - AI stack (Ollama, OpenNotebook, SurrealDB) with socket proxy, (3) scripts/ - Python automation, (4) .agents/ - 60+ agent skills. Branch: hexagon-phase2, commit: e0cd301. Key files: AGENTS.md, chronicler_engine/AGENTS.md, docs/docker.md, .agents/rules/*, opencode.json."
}
```
