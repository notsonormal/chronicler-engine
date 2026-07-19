---
diataxis: reference
title: Architecture System
---

> **Diátaxis mode:** Reference. This document describes the architecture system as it is: the eight-tier tier map, the dependency invariant, the port inventory with impl counts, the storage-direct-access contract, the settings flow shape, and the deployment contract. The problem it solves for the reader is *look-up* — which tier a module lives in, which port trait defines which boundary, where settings are loaded from, what the engine binds at runtime. Module paths and port trait names are stable file-tree identifiers.

## Overview

The source tree has eight tiers with one-direction dependency rules enforced by [`arch-lint.toml`](../../arch-lint.toml). Three port traits (`LlmProvider`, `LlmMessageRepository`, `TextChecker`) define the application-to-driven-adapter boundary under [`src/application/ports/`](../../src/application/ports/). A small set of `arch-lint: storage-direct` markers permit direct `Storage` access at the persistence boundary — either as an intentional persistence seam or as a deferred exemption. Settings are loaded once at startup in `bootstrap/run.rs::load_settings` and propagated as `Arc<RwLock<AppSettings>>` held on `AppState.settings`; construction-chain recipients take a reference to settings at wiring time and no business-logic layer reloads settings from disk after bootstrap. At runtime the engine binds a single HTTP port, writes to one SQLite file, reads seeds from `data/`, writes runtime data to `saves/`, reads and writes prompt presets to `prompts/`, and calls out to a configured LLM backend over HTTPS.

## Layer Structure

Eight tiers organize the codebase:

- `crate::domain::model` — Pure data structures; single source of truth for game state.
- `crate::domain::engine` — Pure simulation logic.
- `crate::application` — Orchestration layer; owns port traits under `application/ports/`. Sub-modules from the T2 land package:
  - `crate::application::persistence_gate` — application persistence boundary; owns `Arc<Storage>` + `Arc<PresetStore>`.
  - `crate::application::generation_gate` — per-game registry (write-side truth, `Arc<RwLock<HashMap<u64, GenerationSlot>>>`) + atomic projection (`Arc<AtomicBool>`, read-only cache of "any slot Generating"); `is_shutting_down` lives on `DefaultApplicationService` reading `AppState.shutdown_token`.
  - `crate::application::game_catalogue` — game-lifecycle orchestration; borrows `Arc<PersistenceGate>`.
  - `crate::application::world_catalogue` — worlds/presets persistence; takes raw `Arc<Storage>` (deliberate asymmetry vs GameCatalogue — independent seams).
- `crate::adapters::driven` — Outbound adapters (storage, LLM providers, text check).
  - `crate::adapters::driven::storage::preset_store` — adapter around `Storage` for prompt preset CRUD (storage-backed).
- `crate::adapters::driving` — Inbound adapters (HTTP server, CLI).
- `crate::bootstrap` — Composition root.
- `crate::settings` — Settings data model (`src/settings.rs`); DB-backed, loaded once at startup.
- `crate::test_support` — Shared fixtures, `TestDataBuilder`, and the `TestAppBuilder` API.

## Dependency Invariant

The load-bearing rule for the eight tiers:

- `domain/` + `application/` depend on port traits only.
- Adapters implement port traits.
- Only `bootstrap/` imports both port traits and adapter impls.
- Driven-side port traits are owned by `application/ports/`.

Enforced by the `[[deny-scope-dep]]` rules in [`arch-lint.toml`](../../arch-lint.toml).

## Port Inventory

Three port traits live under [`src/application/ports/`](../../src/application/ports/): `LlmProvider`, `LlmMessageRepository`, `TextChecker`.

| Port | Prod impls | Test impls |
|---|---|---|
| `LlmProvider` | 4 (`openrouter`, `deepseek`, `ollama`, `mock`) | 0 |
| `LlmMessageRepository` | 1 (`storage/backend/llm_messages.rs`) | 3 (`RecordingForensics`, `NoopForensics`, `SpyForensics`) |
| `TextChecker` | 1 (`harper_text_checker.rs`) | 4 (`StubCheckerA`, `StubCheckerB`, `StubCheckerNone` in `ports/text_checker_tests.rs`; `StubChecker` in `text_check_service_tests.rs`) |

Impl counts may shift as ports or test fakes are added; regenerate via `git grep -c '^impl <Port> for' src/`.

## Storage Direct Access

The list of `arch-lint: storage-direct` markers is canonical at the source: `git grep "arch-lint: storage-direct" src/application/`. Current markers are either intentional persistence boundaries or deferred exemptions.

## Settings Flow

Settings are loaded **once** at startup. The loaded `AppSettings` is shared via `Arc<RwLock<AppSettings>>` and held on `AppState.settings`. Construction-chain recipients (the game service, the agent registry, and the quantifier agent) take a reference to settings at wiring time.

No business-logic layer reloads settings from disk after bootstrap. Connection changes require a server restart to take effect; only `max_context_tokens` is read dynamically per call (it is a per-call budget, not a connection-rewiring decision). The detailed bootstrap sequence is in `./startup.md`.

## Deployment Contract

The engine's deployment contract ends at its process boundary — what it binds, reads, and calls out to. Workspace-level orchestration (Caddy, Docker, network topology) is out of scope for this doc.

- **HTTP port** — binds a single HTTP port (configurable); serves dashboard + action + polling endpoints; TLS termination and reverse proxying are workspace-operator concerns.
- **SQLite database** — one file per instance (e.g. `chronicler_3000.db`); auto-created on first access.
- **File system** — reads JSON seeds from `data/`; writes runtime data to `saves/`; reads and writes prompt presets to `prompts/`.
- **Outbound LLM calls** — HTTPS to the configured backend (OpenRouter / DeepSeek / Ollama); endpoint URL configurable.
- **In-process text check** — `harper_core` linked into engine binary; text checking is a function call.

## Default Settings

AppSettings defaults are authored in `src/domain/model/settings.rs::Default`; settings.json is **not** consulted at startup.

## Document References

- [`../explanation/architecture.md`](../explanation/architecture.md) — hexagonal architecture ([§Layer Structure](../explanation/architecture.md#layer-structure)), dependency invariant (same section), port ownership and storage direct-access exemption.
- [`./startup.md`](./startup.md) — bootstrap sequence and the point at which settings are loaded.
- [ADR-010: Concurrency and Generation Gate Model](../../docs/adr/adr-010-concurrency-generation-gate.md) — settings ownership pattern (`Arc<RwLock<AppSettings>>`) and the `AtomicBool` generation gate.
- [ADR-027: Hexagonal Architecture Migration](../../docs/adr/adr-027-hexagonal-architecture-migration.md) — port ownership and the storage direct-access exemption.
