---
diataxis: reference
title: Architecture System
---

## Overview

The engine is organised as a hexagon.

## Layer Structure

- `crate::domain::model` — Pure data structures AND pure rule methods.
- `crate::application` — Orchestration.
- `crate::adapters::driven` — Outbound adapters (storage, LLM providers, text check).
- `crate::adapters::driving` — Inbound adapters (HTTP, CLI).
- `crate::bootstrap` — Composition root.
- `crate::test_support` — Shared fixtures.

## Dependency Invariant

- `domain/` + `application/` depend on port traits only.
- Adapters implement port traits.
- Only `bootstrap/` imports both port traits and adapter impls.
- Driven-side port traits are owned by `application/ports/`.

Enforced by `[[deny-scope-dep]]` in `arch-lint.toml`.

## Storage Direct Access

`arch-lint: storage-direct` markers in `src/application/` permit direct `Storage` access at the persistence boundary. The marker list is canonical at the source; see `## Document References` for the ADR that codifies the intentional/deferred split.

## Settings

Loaded once at startup in `src/utils/settings.rs::load_settings` and shared via `Arc<RwLock<AppSettings>>` held on `AppState.settings`. Construction-chain recipients take a reference at wiring time; no business-logic layer reloads from disk. `max_context_tokens` is read dynamically per call (per-call budget).

## AppState collaborators

`AppState` is the handler-facing bundle produced by `bootstrap/wiring.rs` (`WiredApp`) and passed to the Axum router. It exposes the named collaborators directly, with no service façade between HTTP handlers and the application layer:

- `pipeline: Arc<ActionPipeline>` — action execution and phase orchestration.
- `world_persona: WorldPersonaCatalogue` — world and persona CRUD.
- `game_catalogue: GameCatalogue` — game lifecycle (create/switch/delete/list/reset/current id).
- `game_view_query: GameViewQuery` — read-side fragments for the UI and debug state.
- `generation_gate: GenerationGate` — per-game generation-slot gating and reset.
- `persistence_gate: Arc<PersistenceGate>` — snapshot/message persistence and multi-table writes.
- `text_check_service: Arc<TextCheckService>` — player-command spelling/grammar check.
- `settings: Arc<RwLock<AppSettings>>` — runtime settings.
- `shutdown_token: CancellationToken` — request shutdown signal.
- `storage` / `preset_storage` — `Arc<Storage>` for the game and preset databases.

## Deployment Contract

- **HTTP port** — binds a single HTTP port (configurable); serves dashboard + action + polling endpoints. TLS termination and reverse proxying are workspace-operator concerns.
- **SQLite database** — one file per instance (e.g. `chronicler_3000.db`); auto-created on first access.
- **File system** — reads JSON seeds from `data/`; writes runtime data to `saves/`; reads/writes prompt presets to `prompts/`.
- **Outbound LLM calls** — HTTPS to the configured backend (OpenRouter / DeepSeek / Ollama); endpoint URL configurable.
- **In-process text check** — `harper_core` linked into the engine binary.

## Document References

- [`../explanation/architecture.md`](../explanation/architecture.md) — hexagonal architecture and dependency invariant.
- [`./startup.md`](./startup.md) — bootstrap sequence and settings load point.
- [ADR-010: Concurrency and Generation Gate Model](../../docs/adr/adr-010-concurrency-generation-gate.md) — settings ownership pattern and `AtomicBool` generation gate.
- [ADR-027: Hexagonal Architecture Migration](../../docs/adr/adr-027-hexagonal-architecture-migration.md) — port ownership and the storage direct-access exemption.