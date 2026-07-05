# Agent System

> **Related Decisions**: [ADR-009](../adr/adr-009-agent-trait-registry.md)
> **Parent Spec**: [multi-agent-architecture-overarching-spec.md](../plans/multi-agent-architecture-overarching-spec.md)

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
| `backend_selector()` | Which LLM backend the agent uses |
| `execute()` | Run the agent; returns `AgentResult` |

---

## Execution Phases

```rust
pub enum ExecutionPhase {
    PreGeneration,   // Before main LLM call (future: prompt injection)
    PostGeneration,  // After main LLM call (current: quantifier)
}
```

**Pipeline flow** (simplified):

1. Load state from snapshot
2. Run **PreGeneration** agents
3. Generate main narration via LLM
4. Run **PostGeneration** agents (`QuantifierAgent` analyzes narration)
5. Apply agent results → `execute_freeaction_impl` → save snapshot

---

## Agent Results

```rust
pub enum AgentResult {
    PromptDirective(String),  // Inject text into prompt (for future pre-gen agents)
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

The `QuantifierAgent` returns `StatePatch` with the NPCs it detected in the narration and any movement destination. `DefaultGameService` translates this patch back into a `QuantifierResult` for `action_processing.rs`.

---

## Agent Registry

`AgentRegistry` is constructed at startup from `AppSettings.agents`:

```rust
let registry = AgentRegistry::from_configs(&settings.agents)?;
```

If no agent config exists, defaults are injected for backward compatibility:

- `quantifier` agent enabled, `PostGeneration`, `UseNamed("quantifier")` backend

### Config Format (settings.json)

```json
{
  "agents": [
    {
      "name": "quantifier",
      "agent_type": "quantifier",
      "enabled": true,
      "backend": { "type": "use_named", "value": "quantifier" },
      "phase": "post_generation"
    }
  ]
}
```

**Fields:**

- `name` — Display name
- `agent_type` — `"quantifier"` (unknown types fail fast at startup)
- `enabled` — `true` to register; `false` skips
- `backend` — `{"type": "use_main"}` or `{"type": "use_named", "value": "<connection_id>"}`
- `phase` — `"pre_generation"` | `"post_generation"`

---

## Backend Selection

```rust
pub enum BackendSelector {
    UseMain,           // Use the main narration backend
    UseNamed(String),  // Use a named connection from settings
}
```

**QuantifierAgent**: Resolves its backend via `AgentRegistry::from_configs_with_storage()`, which receives `&AppSettings` from the caller (no file I/O). The `quantifier_connection_id` is read from the passed settings. `UseMain` falls back to the default narration backend.

---

## Per-Agent Backends

Each agent can use a different LLM connection:

- Main narrator → `narration_connection_id`
- Quantifier → `quantifier_connection_id` (or a custom connection via `UseNamed`)

This enables cost optimization (cheap model for quantifier, powerful model for narration).

The engine works with **zero agents** — all agent execution is optional. See [ADR-009](../adr/adr-009-agent-trait-registry.md) for the authoritative extension procedure.
