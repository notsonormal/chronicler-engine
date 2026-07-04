# ADR-009: Agent Trait and Registry Architecture

**Date:** 2026-05-10
**Status:** Accepted

---

## Context

The quantifier was originally a hardcoded pipeline step in the game service. It ran between narration generation and action execution, with direct function calls and no abstraction. Adding any new post-processing step (e.g., a continuity checker or prose guardian) would require rewriting the orchestrator.

Reviews identified that the pipeline shape was deeply coupled: while the backend trait allowed swapping implementations, the *orchestration* was fixed in code.

---

## Decision

**Introduce a phase-based `Agent` trait and `AgentRegistry` for extensible pipeline orchestration.**

### Agent Trait

```rust
pub trait Agent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn phase(&self) -> ExecutionPhase;
    fn backend_selector(&self) -> BackendSelector;
    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError>;
}
```

### Execution Phases

| Phase | When | Current Agents |
|-------|------|----------------|
| `PreGeneration` | Before main LLM call | Reserved for future agents |
| `PostGeneration` | After main LLM response | Scene analysis (quantifier) |

### Agent Result Types

```rust
pub enum AgentResult {
    PromptDirective(String),
    StatePatch(StatePatch),
    NoOp,
}

pub enum StatePatch {
    Scene { npc_ids: Vec<String>, movement_destination: Option<String>, confidence: Confidence },
}
```

### Registry Construction

`AgentRegistry` loads agents from `AppSettings.agents` config at startup. Each agent is constructed with its resolved backend (`Arc<dyn ...>`). `BackendSelector` (`UseMain` | `UseNamed(String)`) determines which connection profile the agent uses.

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
- **Bridge maintenance**: Agent result translation back into legacy quantifier types is temporary technical debt

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
- **2026-05-17**: `game_service` extracted from `engine/` to `application/game_service/`
