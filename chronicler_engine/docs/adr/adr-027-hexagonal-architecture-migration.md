# ADR-027: Hexagonal Architecture Migration

**Date:** 2026-06-30
**Status:** Accepted
**Drivers:** Formalize Ports & Adapters architecture; document rejected/accepted port decisions; establish "phantom port" heuristic

## Context

Prior to this decision, Chronicler Engine was **aspirationally hexagonal but only ~60% realized**:

- LLM providers already had a proper port (`LlmBackend` trait) with 4 impls (OpenRouter, DeepSeek, Ollama, Mock).
- Storage had **no port abstraction** — `GameServiceContext` held `Arc<Storage>` (concrete struct with `Backend` enum for SQLite/InMemory/Test).
- `narrative/` bundled 4 unrelated concerns (LLM HTTP, prompt assembly, agents, text_check).
- `text_check/` was homeless in pure-layered taxonomy — it's an input classifier consuming an external NLP library.
- Engine↔application had no port (direct function calls).
- `LlmBackend` trait was **half-adapter/half-application**: default impls (`save_message`, `wrap_and_save`, `postprocess_response_text`) reached into `Storage` and sanitization logic.

Pure-layered was rejected because the LLM port *already exists*; standardizing on pure-layered would require either dropping the trait (regression) or pretending it isn't a port (mixed architecture — what we want to avoid).

Hexagonal is the natural fit. Chronicler's existing LLM port + DI constructor (`DefaultGameService::with_backends`) are already hexagonal patterns. This ADR formalizes the rest of the codebase around them.

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
- Core (`domain/`, `application/`) depends on port traits only
- Adapters implement port traits
- Only `bootstrap/` imports both port traits and adapter impls

### Accepted Port Traits

| Port | Rationale | Impl Count |
|------|-----------|------------|
| `LlmProvider` | 4 impls: OpenRouter, DeepSeek, Ollama, Mock. Clear substitution seam. | 4 |
| `LlmMessageRepository` | Consumer (`LlmCallRecorder`) is in core; producer (`Storage`) is driven adapter. Port justified by **consumer location**, not impl count. See "Phantom Port Heuristic" below. | 1 |
| `TextChecker` | Consumer (`TextCheckService`) is in core; producer (`HarperTextChecker`) is driven adapter. Single impl justified by consumer location. | 1 |

### Rejected Port Traits

| Port | Rationale |
|------|-----------|
| `StateRepository` | Single-impl (`Storage` struct). Substitution happens via `Backend` enum, not trait swapping. YAGNI. |
| `DebugPort` | Phantom — single debug consumer + single debug surface. |
| `ActionPipelineBackend` | Collapsed into `ActionPipeline` direct fields (Phase 2.4). God-trait bundled LLM, agents, storage — all now owned by the pipeline directly. |

### Phantom Port Heuristic

**One impl alone does NOT make a port phantom.**

A port is **phantom** (unjustified) when:
- Single impl **AND**
- Consumer is **not** in core **OR** producer is **not** an adapter

A port is **justified** (even with single impl) when:
- Single impl **BUT**
- Consumer is in core **AND** producer is a driven adapter

Example: `LlmMessageRepository` has one impl (`Storage`), but the consumer (`LlmCallRecorder`) is in `application/` and the producer (`Storage`) is in `adapters/driven/`. Without the port, core would import the adapter — violating the dependency invariant.

### Storage Direct Access Exemption

Storage (`Storage` struct with `Backend` enum) is accessed directly by the application layer in exactly 3 files:

1. `src/application/context.rs`
2. `src/application/application_service.rs`
3. `src/application/game_service.rs`

These files are marked with `// arch-lint: storage-direct — intentional, see ADR-027` comments. This exemption is **intentional**, not a leak:

- `Storage` is a concrete adapter with no port trait
- Substitution happens via the `Backend` enum (SQLite/InMemory/Test), not trait swapping
- Wrapping `Storage`'s ~40 methods in a `StateRepository` trait would be YAGNI (one impl, no real substitution seam)
- The 3 exempted files form the **application persistence boundary** — no other `application/` file may import `Storage` directly

### Deferred arch-lint Rules

`arch-lint.toml` deny rules for `application → adapters/driven` are **deferred** (arch-lint 0.4.3 lacks TOML-level scoped file exemptions). The 3 exempted files are documented via code comments instead.

Once arch-lint supports scoped file exemptions, add the rule:

```toml
[[rules]]
name = "application → adapters/driven"
deny-scope-dep = ["application", "adapters/driven"]
exempt-files = [
  "src/application/context.rs",
  "src/application/application_service.rs",
  "src/application/game_service.rs",
]
rationale = "Storage direct access — see ADR-027"
```

Deferred rules tracked in `docs/plans/hexagonal-deferred-arch-lint-rules.md`.

## Alternatives Considered

### Alternative A: Pure-Layered Architecture

Adopt pure-layered (Domain → Application → Infrastructure) instead of hexagonal.

**Rejected because:** the LLM port (`LlmBackend`/`LlmProvider`) already exists. Pure-layered would require either:
- Dropping the trait entirely (regression — lose DI, test isolation)
- Pretending it isn't a port (mixed architecture — undermines architectural clarity)

Hexagonal formalizes what Chronicler already does.

### Alternative B: Wrap Storage in `StateRepository` Trait

Create a `StateRepository` trait wrapping all ~40 `Storage` methods.

**Rejected because:**
- Single impl — trait adds ceremony without benefit
- Substitution already works via `Backend` enum (SQLite/InMemory/Test)
- Per-aggregate module split (`backend/characters.rs`, `backend/games.rs`, etc.) provides interface segregation at the module level
- YAGNI — no anticipated second impl

### Alternative C: Reject ALL Single-Impl Ports

Adopt a strict heuristic: "one impl = phantom port, reject all."

**Rejected because:** this would reject `LlmMessageRepository` and `TextChecker`, both of which are justified by **consumer location**. The consumer (`LlmCallRecorder`, `TextCheckService`) is in the core; the producer is an adapter. Without the port, core would import the adapter — violating hexagonal dependency direction.

The "one impl" heuristic is necessary but not sufficient — **location of consumer matters**.

## Consequences

### Positive

- ✅ **Architecture visible at file-tree level.** `ls src/` shows hexagonal structure immediately.
- ✅ **Dependency direction enforced.** Core depends on ports; adapters implement ports; `bootstrap/` wires both.
- ✅ **LLM, TextChecker ports enforced via dependency direction.** Adapters depend on port traits, application orchestration depends on port traits.
- ✅ **Storage exemption is intentional, documented.** 3 files only, marked with comments, forward-referenced to this ADR.
- ✅ **"Phantom port" heuristic is explicit.** Future port decisions have clear criteria.

### Negative

- ⚠️ **Storage direct access is a documented exception.** Not a pure hexagonal implementation. Mitigated by:
  - Exactly 3 files — no creep
  - Comments mark the exemption explicitly
  - ADR documents the tradeoff
- ⚠️ **arch-lint rules are deferred.** 1–4 pre-existing layer leaks (see `hexagonal-deferred-arch-lint-rules.md`) prevent immediate enforcement. Mitigated by:
  - Comments in exempted files
  - This ADR as the authority
  - Phase 2 closes leaks; enforcement follows

### Neutral

- **Folder renaming.** Phase 1 moved files to match hexagonal layout (`model/` → `domain/model/`, `server/` → `adapters/driving/http/`, `storage/` → `adapters/driven/storage/`, `narrative/` split).
- **Port trait locations.** `LlmProvider`, `LlmMessageRepository`, `TextChecker` all in `src/application/ports/`.

## Architecture Impact

### Modified Modules

| Module | Change |
|--------|--------|
| `src/application/ports/` | New folder for driven-side port traits: `llm_provider.rs`, `llm_message_repository.rs`, `text_checker.rs` |
| `src/adapters/driven/` | New parent folder: `llm/`, `storage/`, `text_check/` |
| `src/adapters/driving/` | New parent folder: `http/`, `cli.rs` |
| `src/domain/` | New parent folder: `model/`, `engine/` |
| `src/bootstrap/` | Composition root: `llm_factory.rs`, `text_check_factory.rs` |
| `src/application/context.rs` | Marked with `// arch-lint: storage-direct` comment |
| `src/application/application_service.rs` | Marked with `// arch-lint: storage-direct` comment |
| `src/application/game_service.rs` | Marked with `// arch-lint: storage-direct` comment |
| `docs/architecture/system.md` | Updated with hexagonal section + port inventory |
| `docs/plans/hexagonal-deferred-arch-lint-rules.md` | New file tracking deferred rules |

### Verification

Implementation complete in working tree; verification plan:

- **Build**: `cargo build` clean; `cargo clippy` clean
- **Tests**: `python build.py` passes (baseline: 1190 passed + 2 skipped)
- **arch-lint**: `application → adapters/driven` rule deferred (documented in this ADR + deferred-rules doc)
- **Comment markers**: `grep -rn "arch-lint: storage-direct" src/application/` → exactly 3 matches (context.rs, application_service.rs, game_service.rs)

## Related

- Plan: `docs/plans/hexagonal-reorganization-plan.md` (Phase 1 + Phase 2 + Phase 3.4)
- Deferred rules: `docs/plans/hexagonal-deferred-arch-lint-rules.md`
- ADR-012: LLM Call Logging and Forensics (motivation for `LlmMessageRepository`)
- ADR-018: Application Service Layer (application orchestration layer)
- ADR-020: Unified Storage Struct (`Storage` concrete adapter)
- ADR-022: PromptAssembler Trait Decoupling (port trait precedent)
- ADR-026: Relocate Persona Binding from World to Game (preceding hexagon-phase2 work)
