# ADR-009: Agent Trait and Registry Architecture

**Date:** 2026-05-10

---

## Context

The quantifier was originally a hardcoded pipeline step in `DefaultGameService`. It ran between narration generation and `execute_freeaction_impl`, with direct function calls and no abstraction. Adding any new post-processing step (e.g., a continuity checker or prose guardian) would require rewriting the orchestrator.

Reviews identified that the pipeline shape was deeply coupled: while backend traits (`LlmBackend`, `QuantifierBackendTrait`) allowed swapping implementations, the *orchestration* was fixed in code.

---

## Decision

**Introduce a phase-based `Agent` trait and `AgentRegistry` for extensible pipeline orchestration.**

### Agent Trait

```rust
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn phase(&self) -> ExecutionPhase;
    fn backend_selector(&self) -> BackendSelector;
    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError>;
}
```

### Execution Phases

| Phase | When | Current Agents |
|-------|------|----------------|
| `PreGeneration` | Before main LLM call | `NarratorAgent` (stub, reserved) |
| `PostGeneration` | After main LLM response | `QuantifierAgent` |

### Agent Result Types

```rust
pub enum AgentResult {
    PromptDirective { /* injection into next phase */ },
    StatePatch(StatePatch),
    NoOp,
}

pub enum StatePatch {
    Scene { npc_ids: Vec<String>, movement_destination: Option<String>, confidence: Confidence },
}
```

### Registry Construction

`AgentRegistry` loads agents from `AppSettings.agents` config at startup. Each agent is constructed with its resolved backend (`Arc<dyn ...>`). `BackendSelector` (`UseMain` | `UseNamed(String)`) determines which connection profile the agent uses.

### Quantifier Migration

The existing quantifier code moved from `src/narrative/quantifier/` to `src/narrative/agents/quantifier/` and implements `Agent`. `DefaultGameService` no longer owns a `QuantifierBackendTrait` directly; it owns an `AgentRegistry` and iterates post-generation agents.

### Bridge Pattern

`action_processing.rs` still takes `QuantifierResult` (not `AgentResult`) in Phase 2. `game_service.rs` translates `StatePatch::Scene` back into `QuantifierResult` as a temporary bridge. Phase 3 will replace this with generic dispatch.

---

## Consequences

### Positive
- **Extensibility**: New agents added by implementing `Agent` + config entry — no orchestrator changes
- **Phase isolation**: Pre-generation and post-generation concerns are cleanly separated
- **Backend per agent**: Each agent can use a different model/provider via `BackendSelector`
- **Testability**: Mock agents can be injected via `AgentRegistry` constructor

### Negative
- **Indirection cost**: `dyn Agent` dispatch adds one vtable call per agent
- **Config complexity**: `settings.json` now includes an `agents` array
- **Bridge maintenance**: `StatePatch` → `QuantifierResult` translation is temporary technical debt

### Trade-offs
- Chose trait objects over generics to avoid infecting the entire call stack with type parameters
- Chose two-phase (pre/post) over Marinara's three-phase (pre/parallel/post) for simplicity
- Chose per-agent backends over single shared backend for flexibility

---

## Related ADRs

- [ADR-006: Quantifier-Driven Game Systems](./adr-006-quantifier-systems.md) — Quantifier predates the Agent abstraction
- [ADR-008: SQLite Snapshot Persistence](./adr-008-sqlite-snapshot-persistence.md) — Snapshots enable safe agent retry

---

## History

- **2026-05-10**: Phase 2 implementation — `Agent` trait, `AgentRegistry`, `QuantifierAgent` migration
