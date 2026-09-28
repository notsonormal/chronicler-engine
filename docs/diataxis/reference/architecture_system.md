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

`Storage` is a concrete adapter with a `Backend` enum (SQLite / InMemory / Test). Application code may call `Storage` directly when the call is part of the persistence boundary; a `StateRepository` port would be a single-impl trait with no real substitution seam, so the boundary is concrete-by-design. Architectural review owns the discipline for which files sit on the boundary.

## Settings

The settings row is the read source for the whole engine. Every consumer resolves settings from storage at the point of use, so a connection or preset change takes effect on the next request without a restart. Bootstrap seeds the row once from `data/settings.json` when the database has no row yet; after that the file is never consulted. A connection reference that names no stored connection fails the operation that needs it instead of falling back to another backend.

## AppState collaborators

`AppState` is the handler-facing bundle produced by `bootstrap/wiring.rs` (`WiredApp`) and passed to the Axum router. It exposes the named collaborators directly, with no service façade between HTTP handlers and the application layer:

- `pipeline: Arc<ActionPipeline>` — action execution and phase orchestration.
- `message_service: Arc<MessageService>` — snapshot/message persistence and multi-table writes.
- `game_catalogue: GameCatalogue` — game lifecycle (create/switch/delete/list/reset/current id).
- `game_view_query: GameViewQuery` — read-side fragments for the UI and debug state.
- `generation_gate: GenerationGate` — per-game generation-slot gating and reset.
- `world_catalogue: WorldCatalogue` — world CRUD.
- `persona_catalogue: PersonaCatalogue` — persona CRUD.
- `settings_service: SettingsService` — settings persistence.
- `prompt_preset_service: PromptPresetService` — prompt-preset CRUD.
- `text_check_service: Arc<TextCheckService>` — player-command spelling/grammar check.
- `shutdown_token: CancellationToken` — request shutdown signal.

## Deployment Contract

- **HTTP port** — binds a single HTTP port (configurable); serves dashboard + action + polling endpoints. TLS termination and reverse proxying are workspace-operator concerns.
- **SQLite database** — one file per instance (e.g. `chronicler_3000.db`); auto-created on first access.
- **File system** — reads JSON seeds from `data/`; writes runtime data to `saves/`; reads/writes prompt presets to `prompts/`.
- **Outbound LLM calls** — HTTPS to the configured backend (OpenRouter / DeepSeek / Ollama); endpoint URL configurable.
- **In-process text check** — `harper_core` linked into the engine binary.

## Document References

- [`../explanation/architecture.md`](../explanation/architecture.md) — hexagonal architecture and dependency invariant.
- [`./startup.md`](./startup.md) — bootstrap sequence and settings load point.
