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

The record writes of a Game's story exist only on `Tx`, and only `GameWriter` opens a transaction of them (see *Who Owns What*).

## Settings

The settings row is the read source for the whole engine. Every consumer resolves settings from storage at the point of use, so a connection or preset change takes effect on the next request without a restart. Bootstrap seeds the row once from `data/settings.json` when the database has no row yet; after that the file is never consulted. A connection reference that names no stored connection fails the operation that needs it instead of falling back to another backend.

## AppState collaborators

`AppState` is the handler-facing bundle that `bootstrap/wiring.rs` (`WiredApp`) builds and the Axum router receives. A handler calls the application layer directly, through the named collaborators of the bundle. The table in "Who Owns What" shows which part owns and writes each record.

## Who Owns What

Each part owns one thing and writes only what its row names.

| Part | Owns | May write |
|---|---|---|
| `GameWriter` (`generation/gate.rs`) | The busy-state registry (one holder for each busy Game), the guard, and the transactions of record writes | Every record of a Game's story and the config columns of the Game record, through a transaction: accept, stop, edit, delete, swipe switch, config save, create, reset, delete of a Game |
| `JobWriter` (`generation/gate.rs`) | One Job's right to write | For its own Job only, while the job entry of its Game's holder holds its id: the Input save, the text save, the state update, the failure line, the Options save and the step name |
| `Job` (`pipeline/job.rs`) | The plan of one request, its generations and their steps | Nothing directly. It writes through its `JobWriter` and passes ids between steps. |
| `Narrator` (`narration/narrator.rs`) | The assembly and the model call of one Narration | Nothing. It returns text. |
| `AgentRunner` and the Agents | The hooks and the value each hook returns | Nothing. An Agent returns values. |
| `AutoplayRun` (`autoplay/driver.rs`) | One Autoplay run: its budget, its cycles and its cycle rule | Its holder and its progress only, through `GameWriter`. Its Jobs write through their own `JobWriter`. |
| `GameCatalogue` | The lifecycle of Games, and `ActiveGame` | `ActiveGame`, and the display name of the Game record. The config columns, create, reset and delete go through `GameWriter`. |
| `GameViewQuery` | The read-side views and the status rule | Nothing |
| `PromptPresetService` | The preset a Game uses for the Narrator and for each hook, and the rules of a preset edit and a preset delete | Preset rows, through `Storage` |
| `Storage` | The records on disk, the backend mutex, and the read handles `stories()` and `docks()` | Its own tables. It writes a Game's story records only inside a `Tx`. |
| `LlmCallRecorder` | The LLM message of each LLM call | `llm_messages` rows, through `SaveLlmMessageFn` |
| HTTP handlers | Request parsing and response rendering | Nothing. They call `Job::start`, `AutoplayRun::start`, `GameWriter`, the catalogues and `PromptPresetService`. |

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
