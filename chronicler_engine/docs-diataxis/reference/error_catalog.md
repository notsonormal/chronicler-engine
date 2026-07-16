---
diataxis: reference
title: Error Catalog
---

> **Diátaxis mode:** Reference. This document catalogs every `EngineError` variant — its first-check diagnostic, common causes, and the runtime invariant identifiers that bear on it. The problem it solves for the reader is *look-up*: an error variant appeared in a log or test failure, what does it mean and where does it originate. The `EngineError` definition is the source of truth at `src/error.rs`; invariant guarantee text lives in the invariant contract tests, not in this document. INV-NNN identifiers are stable seams and resolve through `./guardrails.md`.

## Overview

`EngineError` is the engine's single error type; every fallible API returns `Result<T, EngineError>`. Variants are grouped into four families:

- **Subsystem wrappers** (`Llm`, `Narrative`, `Database`) carry an inner failure type that distinguishes transport-vs-parse, prompt-vs-generation, and storage-error kinds.
- **Required-read absences** (`RoomNotFound`, `MessageNotFound`, `GameNotFound`, `PersonaNotFound`, `WorldNotFound`, `WorldHasGames`) come from `Storage::require_*` paths and from application handlers that look up by id.
- **Data and parse failures** (`Io`, `Serde`, `Parse`, `Serialize`, `Navigation`, `Config`, `Template`, `Render`, `DataLoad`, `ContextOverflow`) originate in adapter and bootstrap layers.
- **Invariant violations** (`Internal`) carry the violated invariant identifier string; the violated invariant must have a corresponding test in `tests/infrastructure/invariant_contract.rs`.

Each variant below lists a **First Check** (the highest-signal diagnostic to run first), **Common Causes** (where to look next), and **Related Invariants** (the INV-NNN identifiers that bear on the failure mode). Variant names include their payload fields when the payload carries the contract — the `{ status, body }`, `{ url, detail }`, and similar shapes are what an engineer greps for to find the failing call.

For invariant guarantee text and the contract tests that enforce each INV-NNN, see `./guardrails.md`.

## `EngineError::Llm(LlmFailure)`

The LLM subsystem returns `LlmFailure` from `LlmCallRecorder::complete`; variants distinguish transport-level from parse-level from success-but-empty failures. Each variant maps to a distinct first check.

### `LlmFailure::EmptyResponse`

- **First Check.** Backend logs for `[LLM][req:N] Extracted content via:`. Absence of that line means the model returned an empty `content` / `reasoning` field.
- **Common Causes.** Model returned empty content; prompt longer than context window and truncated; Ollama model unloaded mid-request.
- **Related Invariants.** None — operational/environmental failure.

### `LlmFailure::Http { status, body }`

- **First Check.** The `status` code in the error payload: `401` = API key issue; `429` = rate limited; `5xx` = provider outage.
- **Common Causes.** Invalid API key; rate limiting; model-routing failure; provider maintenance; response body captured in `body` for forensics.
- **Related Invariants.** INV-004 (LLM Calls Are Cancellable) — call cancellation surfaces here when the abort happens mid-stream.

### `LlmFailure::Network { url, detail }`

- **First Check.** Reachability of `url` from the host: `curl -I <url>`.
- **Common Causes.** Ollama not running; network partition; DNS failure; the configured overall request timeout exceeded; truncated gzip stream; server closed connection; TLS handshake failure.
- **Related Invariants.** Ollama must be reachable at the configured `base_url`; INV-004 (LLM Calls Are Cancellable) — the configured timeout figure lives in the LLM transport source and the INV-004 contract test, not in this catalog.

### `LlmFailure::ParseError { raw_response, expected_format }`

- **First Check.** The `raw_response` payload in logs. Is it valid JSON? Does it carry the `expected_format` shape?
- **Common Causes.** Model returned non-JSON prose; response missing `choices[0].message.content`; streaming response when `stream: false` was requested.
- **Related Invariants.** All LLM responses must be valid JSON with a `choices` array carrying a `message` object.

### `LlmFailure::Timeout`

- **First Check.** `RUST_LOG=debug` logs for `[LLM][req:N] Request failed after ...`.
- **Common Causes.** The configured overall timeout exceeded; model too slow for the prompt size; network congestion.
- **Related Invariants.** INV-004 (LLM Calls Are Cancellable) — a non-aborting timeout surfaces here.

## `EngineError::Narrative(NarrativeFailure)`

The narrative pipeline returns `NarrativeFailure` from prompt-build and post-processing paths.

### `NarrativeFailure::PromptBuild { stage, reason }` — test-fixture only

- **First Check.** Search the codebase for `NarrativeFailure::PromptBuild` constructions.
- **Common Causes.** Prompt exceeds `max_context_tokens`; history too long; token-budget miscalculation.
- **Related Invariants.** None. In production, context overflow raises `EngineError::ContextOverflow` directly; this variant is constructed in test fixtures only.

### `NarrativeFailure::Generation { stage, reason }`

- **First Check.** Backend-specific logs. The mock backend uses `stage: "mock"` for narration and `stage: "mock_trigger"` for trigger continuation.
- **Common Causes.** LLM call failed after prompt built successfully; backend misconfiguration (e.g. DeepSeek not implemented).
- **Related Invariants.** See `./guardrails.md` §"Runtime Invariants" for the INV-NNN identifiers that bear on pipeline ordering.

## `EngineError::Internal(InternalError)`

A logic invariant was violated. These should never appear in normal operation.

- **First Check.** The `invariant` field — the violating invariant identifier.
- **Common Causes.** State corruption: message-history ordering, room-map consistency, NPC-set consistency, or log ordering violated. The invariant strings are matched by `tests/infrastructure/invariant_contract.rs`. Recovery: reload state from the snapshot; the heal path runs on the next action.
- **Related Invariants.** INV-NNN identifiers that may surface as `Internal` resolve through the contract tests; see `./guardrails.md` §"Runtime Invariants" for the identifier table.

## `EngineError::Database(rusqlite::Error)`

A SQLite call returned an error; the `rusqlite::Error` source carries the kind (`QueryReturnedNoRows`, `SqliteFailure`, `InvalidQuery`, etc.). Raised by storage adapters under `src/adapters/driven/storage/backend/`.

- **First Check.** The wrapped `rusqlite::Error` in logs.
- **Common Causes.** Missing or locked SQLite file; malformed query; FK violation; integer overflow in a column.
- **Related Invariants.** None — storage failures propagate raw from rusqlite.

## `EngineError::Io(String)`

A filesystem or network I/O error outside SQLite and serde paths.

- **First Check.** The path or context string in the error message; permissions; disk space.
- **Common Causes.** Missing world directory; read-only data directory; failed to create the HTTP client (rare; usually wrapped in `LlmFailure::Network`).
- **Related Invariants.** The data directory must exist and be readable.

## `EngineError::Serde(serde_json::Error)`

JSON (de)serialization rejected a payload; the `serde_json::Error` source carries the line/column.

- **First Check.** The file path in the surrounding `DataLoad` or `Parse` context.
- **Common Causes.** Schema mismatch in a seed file; manual JSON editing introduced syntax errors.
- **Related Invariants.** None — schema mismatches are caught by `python build.py --validate-data` against the canonical schemas in `data/schemas/`.

## `EngineError::Parse(String)`

A hand-rolled parser rejected input; the message names the failing fragment.

- **First Check.** The input string that failed parsing.
- **Common Causes.** Malformed command in scenario / trigger / template parsing; invalid settings TOML or JSON.
- **Related Invariants.** None.

## `EngineError::Serialize(String)`

A serialization attempt on an incomplete or inconsistent value failed. The `Serde` variant covers serde-driven (de)serialization failures.

- **First Check.** The object being serialized.
- **Common Causes.** Circular references in domain state; non-serializable types inside a snapshot.
- **Related Invariants.** INV-002 (State Mutation Order) — incomplete state typically signals a missing mutation step.

## `EngineError::Navigation(String)`

A player movement failed; the message is the room identifier or context.

- **First Check.** `state.movement.current_room_id` and the available exits in that room.
- **Common Causes.** Walking in a direction with no exit; room id typo; stale dynamic-room reference.
- **Related Invariants.** None.

## `EngineError::Config(String)`

Settings or backend configuration is missing or invalid.

- **First Check.** `src/domain/model/settings.rs` and the backend that raised the error.
- **Common Causes.** Missing settings file; backend not implemented (e.g. DeepSeek); lock poisoned in server state.
- **Related Invariants.** None.

## `EngineError::Template(String)`

Askama template substitution failed.

- **First Check.** Template file syntax; variable names in the template context.
- **Common Causes.** Template variable renamed in Rust but not in HTML; HTML syntax error in a `.html` template.
- **Related Invariants.** None.

## `EngineError::Render(String)`

A render stage (HTMX/Tera) failed downstream of template substitution.

- **First Check.** The render-stage message; the wrapped template output if exposed.
- **Common Causes.** Fragment render failure (e.g. story-log HTML render failure as in `src/adapters/driving/http/fragments/misc/swipe.rs`); context-merge failure.
- **Related Invariants.** None.

## `EngineError::RoomNotFound(String)`

A room lookup by identifier missed.

- **First Check.** `state.movement.current_room_id`; if it starts with `dynamic_`, the quantifier returned an unrecognized destination.
- **Common Causes.** `room_id` mismatch between map and trigger; quantifier movement detection returned an unknown room.
- **Related Invariants.** None — runtime-only.

## `EngineError::MessageNotFound(u64)`

A message lookup by id missed (deleted, gated, or never persisted).

- **First Check.** The `message_id` in the error payload; verify it exists in `messages` for the current game.
- **Common Causes.** Stale swipe reference; race with delete; gated access.
- **Related Invariants.** None.

## `EngineError::GameNotFound(u64)`

A game id does not correspond to a live game session in storage.

- **First Check.** The `game_id` in the error payload; verify it exists in the `games` table.
- **Common Causes.** Stale active-game cookie; game row purged between sessions.
- **Related Invariants.** None.

## `EngineError::PersonaNotFound(String)`

A persona lookup by name missed.

- **First Check.** The persona key in the error payload; verify it exists in the `personas` table.
- **Common Causes.** Persona seed file removed; persona key renamed without updating game binding.
- **Related Invariants.** None.

## `EngineError::WorldNotFound(String)`

A world lookup by identifier missed.

- **First Check.** The world key in the error payload; verify it exists in the `worlds` table.
- **Common Causes.** World seed file removed; world key renamed without updating game references.
- **Related Invariants.** None.

## `EngineError::WorldHasGames { game_count }`

A world cannot be deleted because games still reference it; `game_count` is the blocker count.

- **First Check.** The `game_count` in the error payload; list games whose `world_key` matches the offending world.
- **Common Causes.** Games created against the world after the delete attempt began; admin trying to delete a world with active sessions.
- **Related Invariants.** None — referential-integrity guard enforced at the storage layer (`src/adapters/driven/storage/backend/worlds.rs`).

## `EngineError::DataLoad { path, source }`

A data file loaded from `path` failed to parse or validate; `source` is the wrapped `EngineError`.

- **First Check.** The `path` field; verify the file exists and is valid JSON.
- **Common Causes.** File missing; JSON syntax error; schema mismatch against the corresponding `data/schemas/*.schema.json`.
- **Related Invariants.** None — caught early by `python build.py --validate-data`.

## `EngineError::ContextOverflow { requested, max }`

The prompt budget was exceeded: `requested` tokens pushed past `max` for the active connection.

- **First Check.** The token-budget calculation documented in `./quantifier_prompt.md`.
- **Common Causes.** History too long; system prompt too large; combined context exceeds `max_context_tokens`.
- **Related Invariants.** The prompt builder must never exceed the configured context window.

## Document References

- [`./guardrails.md`](./guardrails.md) — INV-001..INV-007 identifier table and the runtime-invariants section this catalog cites. Invariant guarantee text and contract tests live alongside `tests/infrastructure/invariant_contract.rs`, not in this catalog.
- [`./storage.md`](./storage.md) — the `get_*` / `require_*` read contract that produces `GameNotFound`, `MessageNotFound`, `WorldNotFound`, `PersonaNotFound`, and `RoomNotFound`; the `WorldHasGames` blocker rule.
- [`./data_schemas.md`](./data_schemas.md) — the JSON schema files (`data/schemas/*.schema.json`) that the schema-mismatch branches in `Serde` and `DataLoad` are checked against.
- [`src/error.rs`](../../src/error.rs) — the `EngineError` enum and its inner types (`LlmFailure`, `NarrativeFailure`, `InternalError`) as the source of truth for variant definitions and payload shapes.
- [ADR-012: LLM Message Logging](../../docs/adr/adr-012-llm-message-logging.md) — historical record for the `LlmMessageRepository` forensics layer that surrounds `LlmFailure` variants.
- [ADR-014: Action Pipeline Architecture](../../docs/adr/adr-014-action-pipeline.md) — phase-based pipeline rationale that bears on `NarrativeFailure::Generation` ordering.
- [ADR-032: PhaseError](../../docs/adr/adr-032-phaseerror.md) — historical record for the `PhaseError` type that lives in the action pipeline and intersects `NarrativeFailure` at the pipeline boundary.
