---
diataxis: reference
title: Architecture System
---

## Overview

Eight tiers in `src/`, dependency rules enforced by `arch-lint.toml`. Three port traits under `src/application/ports/`. Single-process deployment against one SQLite file.

## Layer Structure

- `crate::domain::model` — Pure data structures; game state.
- `crate::domain::engine` — Pure simulation logic.
- `crate::application` — Orchestration; owns port traits under `application/ports/`.
  - `application::persistence_gate` — Application persistence boundary; owns `Arc<Storage>` + `Arc<PresetStore>`.
  - `application::generation_gate` — Per-game registry (`Arc<RwLock<HashMap<u64, GenerationSlot>>>`) + atomic projection (`Arc<AtomicBool>`).
  - `application::game_catalogue` — Game-lifecycle orchestration; borrows `Arc<PersistenceGate>`.
  - `application::world_catalogue` — Worlds/presets persistence; takes raw `Arc<Storage>`.
- `crate::adapters::driven` — Outbound adapters (storage, LLM providers, text check).
  - `adapters::driven::storage::preset_store` — Adapter around `Storage` for prompt preset CRUD.
- `crate::adapters::driving` — Inbound adapters (HTTP, CLI).
- `crate::bootstrap` — Composition root.
- `crate::settings` — Settings data model (`src/settings.rs`); DB-backed.
- `crate::test_support` — Shared fixtures; `TestDataBuilder`, `TestAppBuilder`.

## Dependency Invariant

- `domain/` + `application/` depend on port traits only.
- Adapters implement port traits.
- Only `bootstrap/` imports both port traits and adapter impls.
- Driven-side port traits are owned by `application/ports/`.

Enforced by `[[deny-scope-dep]]` in `arch-lint.toml`.

## Port Inventory

Three port traits under `src/application/ports/`: `LlmProvider`, `LlmMessageRepository`, `TextChecker`.

| Port | Prod impls | Test impls |
|---|---|---|
| `LlmProvider` | 4 (`openrouter`, `deepseek`, `ollama`, `mock`) | 0 |
| `LlmMessageRepository` | 1 (`storage/backend/llm_messages.rs`) | 3 (`RecordingForensics`, `NoopForensics`, `SpyForensics`) |
| `TextChecker` | 1 (`harper_text_checker.rs`) | 4 |

Impl counts may shift; regenerate via `git grep -c '^impl <Port> for' src/`.

## Storage Direct Access

`arch-lint: storage-direct` markers in `src/application/` permit direct `Storage` access at the persistence boundary. The marker list is canonical at the source; see `## Document References` for the ADR that codifies the intentional/deferred split.

## Settings

Loaded once at startup in `bootstrap/run.rs::load_settings`. Shared via `Arc<RwLock<AppSettings>>` held on `AppState.settings`. Construction-chain recipients take a reference at wiring time; no business-logic layer reloads from disk. `max_context_tokens` is read dynamically per call (per-call budget). Defaults are authored in `src/domain/model/settings.rs::Default`; `settings.json` is not consulted at startup.

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