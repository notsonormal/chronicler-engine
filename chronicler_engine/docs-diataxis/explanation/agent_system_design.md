---
diataxis: explanation
title: Agent System Design
---

> **Diátaxis mode:** Explanation. This document is *understanding-oriented*: it explains why the Chronicler Engine has a trait-and-registry agent abstraction when there is only one agent in production. It is the companion to `../reference/agent_system.md`, which describes the machinery as it is. The reader problem solved here is *understanding*: why the abstraction is shaped this way, and which tradeoffs that shape encodes.

## Why an abstraction at all

The pipeline that runs after narration could have been written as direct calls — a function that runs the quantifier here, a function that runs a continuity check there, a function that scores prose quality somewhere else. Each new post-processing concern would add a branch in the post-generation phase, a new argument threaded through the pipeline, and a new entry in the test fixtures that construct it.

The `Agent` trait plus `AgentRegistry` makes the pipeline's shape indifferent to which post-processing steps exist. Adding, removing, or reordering concerns is a config change rather than a refactor. The pipeline's `PostGeneration` dispatch is a single iteration over whatever the registry was built with, including the empty set; the engine runs correctly with zero agents configured.

The tradeoff is the abstraction's surface — trait, registry, phases, wiring — paid up front so the pipeline never has to change for a post-processing concern.

## Why trait objects

The registry stores agents as `Box<dyn Agent>` rather than `AgentRegistry<T: Agent>`. The registry holds a heterogeneous collection — each agent is a distinct concrete type — and a generic registry would thread a type parameter through every call site that holds the registry, even though only the collection's storage needs the type.

The tradeoff is a vtable call per agent per phase dispatch. That cost is negligible relative to the LLM call each agent makes.

## Why a per-agent backend

The narration LLM call and the quantifier LLM call have different needs. Narration wants a capable, often expensive model: it produces the prose the player reads. The quantifier wants a cheaper, more predictable model: its job is structured extraction over a fixed schema — NPCs detected, movement destination — not creative writing. Running both through the same model would either under-spend on narration or overspend on the quantifier.

The wiring keeps the two paths independent. `AppSettings` carries separate `narration_connection_id` and `quantifier_connection_id` strings; each resolves to its own `LlmProviderConfig`; the bootstrap builds a dedicated `LlmCallRecorder` for each. The `AgentRegistry` receives the quantifier's recorder at construction and binds it to the quantifier agent at wiring time. `GameService` keeps the narration recorder for its own use.

The `BackendSelector` enum (`UseMain | UseNamed(String)`) is a third column that the dispatch path does not consult. The recorder is bound at wiring time, not at dispatch time, because the wiring already resolved which connection to use when it built the recorder. The selector is metadata today — vestigial in the dispatch path, retained on the trait because it carries intent the wiring can honour in future. This is a seam the design has not closed; acknowledging it is more honest than pretending it is load-bearing.

## Why two phases, with one unused

`ExecutionPhase` has two variants: `PreGeneration` and `PostGeneration`. Only `PostGeneration` is dispatched against today — `PreGeneration` exists in the enum but no caller asks for it.

The two-phase split separates post-narration concerns (detecting NPCs, detecting movement — clearly `PostGeneration`, after the narration LLM has spoken) from pre-narration concerns, which are harder to name in the abstract. The design reserved the phase rather than invent a use for it, on the basis that reserving is cheaper than adding a variant later. `PreGeneration` has no dispatcher by current state, not by oversight.

The quantifier predates the `Agent` trait — it was promoted into the trait abstraction later. Agent constructors carry `Option<Arc<Storage>>` directly under the storage-direct exemption documented in the hexagonal-architecture ADR.

## Document References

- [ADR-009: Agent Trait and Registry Architecture](../../docs/adr/adr-009-agent-trait-registry.md) — historical decision record for the agent abstraction.
- [ADR-006: Quantifier-Driven Game Systems](../../docs/adr/adr-006-quantifier-systems.md) — the quantifier predates the Agent abstraction; the trait later absorbed it.
- [ADR-027: Hexagonal Architecture Migration](../../docs/adr/adr-027-hexagonal-architecture-migration.md) — agent constructors carry `Option<Arc<Storage>>` under the storage-direct exemption.
- [`../reference/agent_system.md`](../reference/agent_system.md) — reference description of the agent machinery.
