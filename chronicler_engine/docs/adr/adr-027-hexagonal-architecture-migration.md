# ADR-027: Hexagonal Architecture Migration

**Date:** 2026-06-30
**Status:** Accepted

## Context

Chronicler Engine was aspirationally hexagonal but only ~60% realized:

- LLM providers had a proper port (`LlmProvider`) with 4 impls (OpenRouter, DeepSeek, Ollama, Mock).
- Storage had no port abstraction — `GameServiceContext` held `Arc<Storage>` (concrete struct with `Backend` enum for SQLite/InMemory/Test).
- `narrative/` bundled 4 unrelated concerns (LLM HTTP, prompt assembly, agents, text_check).
- `text_check/` was homeless in pure-layered taxonomy — input classifier consuming an external NLP library.
- Engine↔application had no port (direct function calls).
- `LlmProvider` trait was half-adapter/half-application: default impls reached into `Storage` and sanitization logic.

Hexagonal formalizes what Chronicler already does: the LLM port + DI constructor (`GameService::with_storage`) are already hexagonal patterns. This ADR formalizes the codebase around them.

## Decision

### Adopt Ports & Adapters (Hexagonal) Architecture

```
src/
  domain/        — pure core (entities + rules), no I/O
  application/   — use cases + port traits (driven-side contracts)
    ports/       — driven-side port traits owned by core
  adapters/
    driving/     — inbound adapters (HTTP, CLI)
    driven/      — outbound adapters (Storage, LLM providers, text check)
  bootstrap/     — composition root (only place that imports both port traits and adapter impls)
```

**Dependency invariant:**
- Core (`domain/`, `application/`) depends on port traits only.
- Adapters implement port traits.
- Only `bootstrap/` imports both port traits and adapter impls.

### Accepted Port Traits

| Port | Rationale | Impl Count |
|------|-----------|------------|
| `LlmProvider` | 4 impls: OpenRouter, DeepSeek, Ollama, Mock. Clear substitution seam. | 4 |
| `LlmMessageRepository` | Consumer (`LlmCallRecorder`) is in core; producer (`Storage`) is driven adapter. Justified by consumer location. | 1 |
| `TextChecker` | Consumer (`TextCheckService`) is in core; producer (`HarperTextChecker`) is driven adapter. Single impl justified by consumer location. | 1 |

### Rejected Port Traits

| Port | Rationale |
|------|-----------|
| `StateRepository` | Single-impl `Storage` struct. Substitution happens via `Backend` enum (SQLite/InMemory/Test), not trait swapping. YAGNI. |
| `DebugPort` | Phantom — single debug consumer (`src/adapters/driving/http/debug.rs`) + single debug surface. The existing debug endpoint reaches into `ApplicationService` directly as an intentional guardrail exemption. |
| `ActionPipelineBackend` | Collapsed into `ActionPipeline` direct fields. God-trait bundled LLM, agents, storage — all now owned by the pipeline directly. |

### Phantom Port Heuristic

**One impl alone does NOT make a port phantom.** A port is phantom (unjustified) when:

- Single impl **AND**
- Consumer is **not** in core **OR** producer is **not** an adapter.

A port is justified (even with one impl) when:
- Single impl **BUT**
- Consumer is in core **AND** producer is a driven adapter.

Without the port, core would import the adapter — violating the dependency invariant.

### Storage Direct Access Exemption

Storage (`Storage` struct with `Backend` enum) is accessed directly by the application layer in 8 grandfathered files, split by marker variant:

**Intentional (6 files)** — form the **application persistence boundary**, marked `// arch-lint: storage-direct — intentional, see ADR-027`:

1. `src/application/context.rs`
2. `src/application/application_service.rs`
3. `src/application/game_service.rs`
4. `src/application/persistence_gate/gate.rs` — owns the persistence boundary; Storage import is the seam, not a leak. (T2 ticket 02.)
5. `src/application/game_catalogue/gate.rs` — game-lifecycle orchestration; reaches Storage via `PersistenceGate::storage()` accessor (no direct import).
6. `src/application/world_catalogue/gate.rs` — worlds/presets persistence; deliberate asymmetry vs `GameCatalogue` (raw `Arc<Storage>` vs `Arc<PersistenceGate>`) to keep game/world seams independent. (T2 ticket 04.)

**Deferred to G1-B (2 files)** — agent constructors that still take `Option<Arc<Storage>>` directly; T2 carve-out has landed (see `persistence_gate`), but full caller-site migration is tracked by ticket G1-B. Marked `// arch-lint: storage-direct — deferred to G1-B, see ADR-027`:

7. `src/application/agents/registry.rs`
8. `src/application/agents/quantifier/agent.rs`

The exemption is intentional, not a leak:

- `Storage` is a concrete adapter with no port trait.
- Substitution happens via the `Backend` enum (SQLite/InMemory/Test), not trait swapping.
- Wrapping `Storage`'s ~40 methods in a `StateRepository` trait would be YAGNI (one impl, no real substitution seam).
- The 6 intentional grandfathered files form the application persistence boundary; the 2 deferred agent files still import `Storage` directly pending G1-B. Any other `application/` file importing `Storage` directly is blocked by the `check_application_storage_direct` arch-lint guardrail (arch-lint 0.4.x has no per-file allowlists, so all eight sites must carry a marker comment). Test files (`*_tests.rs`) are excluded from the guardrail because arch-lint cannot distinguish test fakes from production leaks.

### `domain/engine/` Subfolder Kept

`src/domain/engine/` (7 pure-rule files) stays as-is. It calls `domain/model/` only, no I/O, no port needed at the `engine` ↔ `application` boundary (application calls engine functions directly). Flattening into `domain/` root was rejected as churn for no architectural gain — the subfolder communicates "types (`model/`) vs rules (`engine`)" at zero cost.

### `LlmProviderConfig` Stays in Domain

`LlmProviderConfig` (formerly `Connection`) is embedded as `Vec<LlmProviderConfig>` in `AppSettings` in `domain/model/settings.rs`. It stays in domain rather than moving to `application/ports/llm_provider.rs`: moving would force `AppSettings` to import `application::ports::` — violating the `model → application` arch-lint rule. The `api_key`, `base_url`, `provider`, and `max_context_tokens` fields ride along with the persisted `AppSettings` JSON contract. Same precedent as `LlmBackendType` staying in domain.

### `Swipe::snapshot_id` Stays in Domain

`Swipe::snapshot_id: Option<u64>` (DB-assigned FK into the `snapshots` table) stays on the domain entity in `src/domain/model/message.rs`. The original hexagonal review flagged it as a persistence concern leaked into domain and proposed a `Message` (domain) vs `MessageRow` (adapter DTO) split. Rejected as YAGNI: `Message::id`, `Message::sender`, and `MessageType` are equally persistence-assigned, so the split would force DTO duplication across the entire message aggregate — more complexity than it solves. The hexagonal principle is about dependency direction (domain must not depend on adapter types), and `Option<u64>` is a primitive — no direction violation. Application code legitimately reads `snapshot_id` at 6 sites (`context.rs`, `message_editing.rs`, `application_service.rs`, `action_pipeline/retry.rs`); moving it to the mapper only would force N+1 `storage.fetch_snapshot_id_for_swipe()` queries for zero architectural benefit. Same precedent as Deviations 1 (`LlmBackendType`) and 2 (`LlmProviderConfig`).

## Consequences

### Positive

- ✅ Architecture visible at file-tree level. `ls src/` shows hexagonal structure immediately.
- ✅ Dependency direction enforced. Core depends on ports; adapters implement ports; `bootstrap/` wires both.
- ✅ LLM, TextChecker, Storage-direct-access exemptions documented and marked in code.
- ✅ "Phantom port" heuristic is explicit. Future port decisions have clear criteria.

### Negative

- ⚠️ Storage direct access is a documented exception, not a pure hexagonal implementation. Mitigated: exactly 6 intentional files (plus 2 deferred to G1-B), marked with comments, ADR documents the tradeoff.
- ⚠️ `LlmProviderConfig` infra fields (`api_key`, `base_url`, `provider`) remain in domain — rides along with persisted `AppSettings` JSON contract.
- ⚠️ `Swipe::snapshot_id` DB FK remains on domain entity — YAGNI; full DTO split would force duplication across message aggregate.

### Trade-offs

- Chose hexagonal over pure-layered to keep the existing LLM port rather than drop it or pretend it isn't a port (mixed architecture).
- Chose concrete `Storage` + `Backend` enum over `StateRepository` trait — substitution seam already exists without trait ceremony.
- Chose consumer-location heuristic over strict "one-impl = phantom" to keep `LlmMessageRepository` and `TextChecker` (justified by consumer location).
- Chose `LlmProviderConfig` rename-in-place over full split — naming win achieved without schema migration cost.

## Related ADRs

- ADR-012 — LLM Call Logging and Forensics (motivation for `LlmMessageRepository`)
- ADR-020 — Unified Storage Struct (`Storage` concrete adapter)
- ADR-022 — PromptAssembler Trait Decoupling (port trait precedent)
- ADR-026 — Relocate Persona Binding from World to Game (preceding hex-phase work)

## History

- **2026-06-30**: Initial decision.
- **2026-07-04**: Phase B (arch-lint scope split) landed — `model → application` and ports deny rules enforced in `arch-lint.toml`. Phase C (composition root cleanup) landed — `Bootstrap::wiring` is now the only module importing both port traits and adapter impls for LLM/agent construction. `Connection` renamed to `LlmProviderConfig` (Phase E.1). `Swipe::snapshot_id` kept on domain entity (Phase E.2 dropped as YAGNI — Deviation 3). Phase F.1 landed — `ArrivalTaskContext` extracted from `bootstrap/init_game.rs` to `application/arrival_service.rs`; `inject_scenario_logs` moved from `bootstrap/scenario.rs` to `application/scenario.rs`. Storage exemption reduced from 5 files to 3: `QuantifierAgent::from_config_with_storage` and `AgentRegistry::from_configs_with_storage` no longer take `Option<Arc<Storage>>` — recorder injected directly.
- **2026-07-06**: Corrected — exemption is 5 files, not 3. The 2 "deferred to T2" sites (`src/application/agents/registry.rs`, `src/application/agents/quantifier/agent.rs`) still import `Storage` directly (T2 not yet landed). Current grandfathered list: `src/application/context.rs`, `src/application/game_service.rs`, `src/application/application_service.rs` (intentional) + `src/application/agents/registry.rs`, `src/application/agents/quantifier/agent.rs` (deferred to T2). Storage-direct access in any other `application/` file is now blocked by `check_application_storage_direct` guardrail in `tests/infrastructure/guardrails/layers.rs`, since arch-lint 0.4.x lacks per-file allowlists. Test files (`*_tests.rs`) are excluded because arch-lint cannot distinguish test fakes from production leaks.
- **2026-07-09**: T2 land package landed. The 3 intentional-grandfathered carve-outs (`persistence_gate`, `game_catalogue`, `world_catalogue`) joined the boundary. `generation_gate` does not touch `Storage` (cancel token + atomic slot only — not on the list). Grandfathered list grew from 5 to 8; "Deferred to T2" relabeled "Deferred to G1-B" — T2 done, but the 2 agent constructors still take `Option<Arc<Storage>>` and will be migrated by ticket G1-B (separate effort). Marker comments in the 2 deferred files (`agents/registry.rs`, `agents/quantifier/agent.rs`) updated to point at the now-landed `persistence_gate` carve-out.
