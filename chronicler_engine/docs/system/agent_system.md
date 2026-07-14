# Agent System

## Overview

The Chronicler Engine supports an extensible agent architecture where specialized agents can inject behavior into the narrative pipeline at specific execution phases. An **agent** is any type implementing the `Agent` trait. Agents are loaded from `AppSettings` at startup and registered in the `AgentRegistry`.

**Agents:**

- `QuantifierAgent` — Post-generation scene analysis (NPC presence, movement)

---

## Agent Trait

```rust
pub trait Agent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn phase(&self) -> ExecutionPhase;
    fn backend_selector(&self) -> BackendSelector;
    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult>;
}
```

| Method | Purpose |
|--------|---------|
| `name()` | Human-readable identifier for logging |
| `phase()` | When the agent runs (`PreGeneration` or `PostGeneration`) |
| `backend_selector()` | Which LLM backend the agent uses. Reserved; not consulted by the registry. |
| `execute()` | Run the agent; returns `AgentResult` |

---

## Execution Phases

```rust
pub enum ExecutionPhase {
    PreGeneration,   // Before main LLM call. Reserved; no agent dispatched here today.
    PostGeneration,  // After main LLM call. The `QuantifierAgent` runs here.
}
```

**Pipeline flow** (simplified):

1. Load state from snapshot
2. Generate main narration via LLM
3. Run **PostGeneration** agents (`QuantifierAgent` analyzes narration)
4. Apply agent results → `execute_freeaction_impl` → save snapshot

The `PreGeneration` variant exists in the enum for forward compatibility but no dispatcher reads it; the pipeline iterates only `PostGeneration` agents.

---

## Agent Results

```rust
pub enum AgentResult {
    PromptDirective(String),  // Inject text into prompt. Not constructed by any registered agent.
    StatePatch(StatePatch),   // Mutate game state
    NoOp,                     // No action
}
```

### StatePatch

```rust
pub struct StatePatch {
    pub npc_ids: Vec<String>,
    pub movement_destination: Option<String>,
    pub confidence: Confidence,  // High | Medium | Low
}
```

The `QuantifierAgent` returns `StatePatch` with the NPCs it detected in the narration and any movement destination. `GameService` translates this patch back into a `QuantifierResult` for the action pipeline.

### Quantifier Forensics Gap

The narration LLM calls are captured by the recorder (post-call sanitize + forensic save). The quantifier's separate LLM call is **not** captured — it bypasses the recorder and goes directly through the provider. Forensic data for quantifier calls is therefore missing from the forensics log.

Tests can wrap the recorder around the quantifier. The production path has no equivalent.

**Known limitation:** the bypass is current behaviour.

---

## Agent Registry

`AgentRegistry` is constructed at startup from `AppSettings.agents`:

```rust
let registry = AgentRegistry::from_configs(&settings.agents)?;
```

If no agent config exists, defaults are injected for backward compatibility:

- `quantifier` agent enabled, `PostGeneration`, `UseNamed("quantifier")` backend

### Config Format (settings.json)

Each agent receives a recorder bound at wiring time (see Backend Selection below); the settings format is the `AppSettings.agents` array. The `agent_type` discriminator selects the agent implementation; `enabled` controls registration.

---

## Backend Selection

```rust
pub enum BackendSelector {
    UseMain,           // Use the main narration backend
    UseNamed(String),  // Use a named connection from settings
}
```

**QuantifierAgent**: backend is bound at wiring time. `AgentRegistry::from_configs_with_storage` receives a pre-built `quantifier_recorder` (constructed from `settings.quantifier_connection_id` in `bootstrap::wiring`); the agent's `backend_selector()` is not consulted. `UseMain` is reserved (no fallback to the narration backend is currently applied).

---

## Per-Agent Backends

Each agent can use a different LLM connection:

- Main narrator → `narration_connection_id`
- Quantifier → `quantifier_connection_id` (or a custom connection via `UseNamed`)

This enables cost optimization (cheap model for quantifier, powerful model for narration).

The engine works with **zero agents** — all agent execution is optional.

## Document References

- [ADR-009: Agent Trait and Registry Architecture](../adr/adr-009-agent-trait-registry.md) — `Agent` trait + `AgentRegistry` + extension procedure
