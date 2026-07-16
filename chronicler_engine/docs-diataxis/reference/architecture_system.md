---
diataxis: reference
title: Architecture System
---

> **Diátaxis mode:** Reference. This document describes the architecture system as it is: the eight-tier tier map, the port inventory, the storage-direct-access contract, the settings flow shape, and the default settings. The problem it solves for the reader is *look-up* — which tier a module lives in, which port trait defines which boundary, where settings are loaded from. Module paths and port trait names are stable file-tree identifiers. Hexagonal architecture and dependency invariant live in `../explanation/architecture.md` §5; the bootstrap sequence lives in `./startup.md`.

## Overview

The source tree has eight tiers with one-direction dependency rules enforced by `arch-lint.toml`. Three port traits (`LlmProvider`, `LlmMessageRepository`, `TextChecker`) define the application-to-driven-adapter boundary under `crate::application::ports/`. A small set of `arch-lint: storage-direct` markers permit direct `Storage` access at the persistence boundary — either as an intentional persistence seam or as a deferred exemption. Settings are loaded once at startup in `bootstrap/run.rs::load_settings` and propagated as `Arc<RwLock<AppSettings>>` held on `AppState.settings`; construction-chain recipients take a reference to settings at wiring time and no business-logic layer reloads settings from disk after bootstrap. This document is the canonical home for the tier map and the settings-loading shape.

## Tier Map

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
- `crate::settings` — Settings data model; DB-backed, loaded once at startup.
- `crate::test_support` — Shared fixtures, `TestDataBuilder`, and the `TestAppBuilder` API.

## Port Inventory

Three port traits live under `crate::application::ports/`: `LlmProvider`, `LlmMessageRepository`, and `TextChecker`. Trait paths under `application/ports/`.

## Storage Direct Access

A small set of `src/application/` files import `Storage` directly under `// arch-lint: storage-direct` markers, indicating either an intentional persistence boundary or a deferred exemption tracked in the T2 reliability plan. The current marker file list is enumerated by `git grep "arch-lint: storage-direct" src/application/`; the list is not duplicated in this document (it drifts).

## Settings Flow

Settings are loaded **once** at startup. The loaded `AppSettings` is shared via `Arc<RwLock<AppSettings>>` and held on `AppState.settings`. Construction-chain recipients (the game service, the agent registry, and the quantifier agent) take a reference to settings at wiring time.

No business-logic layer reloads settings from disk after bootstrap. Connection changes require a server restart to take effect; only `max_context_tokens` is read dynamically per call (it is a per-call budget, not a connection-rewiring decision). The detailed bootstrap sequence is in `./startup.md`.

## Default Settings

`AppSettings`'s `Default` implementation in `src/domain/model/settings.rs` is the runtime source of truth. The JSON seed `data/settings.json` is not consulted at startup.

| Provider | Default `max_context_tokens` |
|----------|------------------------------|
| `LlmBackendType::Ollama` | 8192 |
| `LlmBackendType::OpenRouter` / `DeepSeek` | 32768 |
| `LlmBackendType::Mock` | 4096 |

Default connections (3): `openrouter-gpt-4o-mini`, `openrouter-euryale`, `ollama-gemma-4-26B`. Default `narration_connection_id` and `quantifier_connection_id`: `"openrouter-gpt-4o-mini"`.

## Document References

- [`../explanation/architecture.md`](../explanation/architecture.md) — hexagonal architecture (§5), dependency invariant (§5), port ownership and storage direct-access exemption.
- [`./startup.md`](./startup.md) — bootstrap sequence and the point at which settings are loaded.
- [ADR-010: Concurrency and Generation Gate Model](../../docs/adr/adr-010-concurrency-generation-gate.md) — settings ownership pattern (`Arc<RwLock<AppSettings>>`) and the `AtomicBool` generation gate.
- [ADR-027: Hexagonal Architecture Migration](../../docs/adr/adr-027-hexagonal-architecture-migration.md) — port ownership and the storage direct-access exemption.
