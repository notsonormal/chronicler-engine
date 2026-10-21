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

`Storage` is a concrete adapter with a `Backend` enum (SQLite), wrapped by `BackendKind` (Direct / Test). Every game-scoped method takes the Game as its first argument. The active-Game pointer is not storage state. Application code may call `Storage` directly when the call is part of the persistence boundary. Architectural review owns the discipline for which files sit on the boundary.

The record writes of a Game's story exist only on `Tx`, and only `GameWriter` opens a unit of them (see *Who Owns What*).

## Settings

The settings row is the read source for the whole engine. Every consumer resolves settings from storage at the point of use, so a connection or preset change takes effect on the next request without a restart. Bootstrap seeds the row once from `data/settings.json` when the database has no row yet; after that the file is never consulted. A connection reference that names no stored connection fails the operation that needs it instead of falling back to another backend.

## AppState collaborators

`AppState` is the handler-facing bundle produced by `bootstrap/wiring.rs` (`WiredApp`) and passed to the Axum router. It exposes the named collaborators directly, with no service façade between HTTP handlers and the application layer:

- `game_writer: GameWriter` — the writer rule: every write to a Game's records, and the registry of job entries.
- `job_deps: JobDeps` — what `Job::start` needs: the narrator, the Agent runner, storage and the watch on Job tasks.
- `game_catalogue: GameCatalogue` — game lifecycle (create/switch/delete/list/reset/active id). It is the only reader and the only writer of `ActiveGame`.
- `game_view_query: GameViewQuery` — read-side fragments for the UI and debug state, and the status rule.
- `world_catalogue: WorldCatalogue` — world CRUD.
- `persona_catalogue: PersonaCatalogue` — persona CRUD.
- `settings_service: SettingsService` — settings persistence.
- `prompt_preset_service: PromptPresetService` — prompt-preset CRUD, the preset of a Game for the Narrator and for each hook, and the rules of a preset edit and a preset delete.
- `connection_test_service: ConnectionTestService` — one short provider call that records nothing.
- `text_check_service: Arc<TextCheckService>` — player-command spelling/grammar check.

## Who Owns What

Each part owns one thing and writes only what its row names.

| Part | Owns | May write |
|---|---|---|
| `GameWriter` (`generation/gate.rs`) | The registry of job entries, the guard, and the units of record writes | Every record of a Game's story, through a save unit: accept, stop, edit, delete, swipe switch, create, reset, delete of a Game |
| `JobWriter` (`generation/gate.rs`) | One Job's right to write | For its own Job only, while the job entry of its Game holds its id: the Input save, the text save, the state update, the failure line, the Options save and the step name |
| `Job` (`pipeline/job.rs`) | The plan of one request, its generations and their steps | Nothing directly. It writes through its `JobWriter` and passes ids between steps. |
| `Narrator` (`narration/narrator.rs`) | The assembly and the model call of one Narration | Nothing. It returns text. |
| `AgentRunner` and the Agents | The hooks and the value each hook returns | Nothing. An Agent returns values. |
| `GameCatalogue` | The lifecycle of Games, and `ActiveGame` | `ActiveGame`, and the config columns of the Game record. Create, reset and delete go through `GameWriter`. |
| `GameViewQuery` | The read-side views and the status rule | Nothing |
| `PromptPresetService` | The preset a Game uses for the Narrator and for each hook, and the rules of a preset edit and a preset delete | Preset rows, through `Storage` |
| `Storage` | The records on disk, the backend mutex, and the read handles `stories()` and `docks()` | Its own tables. It writes a Game's story records only inside a `Tx`. |
| `LlmCallRecorder` | The forensics record of each LLM call | `llm_messages` rows, through `SaveLlmMessageFn` |
| HTTP handlers | Request parsing and response rendering | Nothing. They call `Job::start`, `GameWriter`, the catalogues and `PromptPresetService`. |

A running Job reads the story and the Game, and the job id check guards each of its writes. Between steps, a Job passes ids and loads each record when its step starts.

## Module Shape

An inherent `impl` of a type lives in the file that defines the type, or in a folder named after the type. For example, `impl Storage` may sit in any file under `storage/`. One file may define more than one type, and then it holds the inherent impl of each. `gate.rs` holds `GameWriter`, `JobWriter` and the types that their signatures need. `Tx` and each read handle (`StoryReads`, `DockReads`) hold their inherent impl in their own file.

`gate.rs` has no child modules, so Rust privacy keeps the registry lock inside it.

## Deployment Contract

- **HTTP port** — binds a single HTTP port (configurable); serves dashboard + action + polling endpoints. TLS termination and reverse proxying are workspace-operator concerns.
- **SQLite database** — one file per instance (e.g. `chronicler_3000.db`); auto-created on first access.
- **File system** — reads JSON seeds from `data/`; writes the SQLite database file. Prompt presets and settings are database rows.
- **Outbound LLM calls** — HTTPS to the configured backend (OpenRouter / DeepSeek / Ollama); endpoint URL configurable.
- **In-process text check** — `harper_core` linked into the engine binary.

## Document References

- [`../explanation/architecture.md`](../explanation/architecture.md) — hexagonal architecture and dependency invariant.
- [`./game_flow.md`](./game_flow.md) — the Job and the writer rule.
- [`./startup.md`](./startup.md) — bootstrap sequence and settings load point.
