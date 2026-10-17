---
diataxis: explanation
title: Agent System Design
---

## The agent abstraction

Each agent implements a hook trait: `AnalysisAgent` or `OptionsAgent`. The `AgentRunner` holds the set that the bootstrap wired in. The analysis hook iterates over the analysis Agents in the runner, including the empty set. The Options hook holds at most one Agent. The pipeline runs correctly with zero agents configured.

The trait-plus-runner shape keeps the pipeline indifferent to which post-processing concerns exist. Adding an agent of a known type is a wiring change in bootstrap; a new agent type also needs an arm in `AgentRunner::from_configs`. Removing one is removing it from the runner. Reordering is changing the iteration order. None of these changes require editing the pipeline itself. Each hook has one output type. The compiler rejects an Agent whose answer the hook cannot save.

## Per-agent LLM backends

The narration LLM call and the quantifier LLM call have different needs. Narration produces the prose the player reads and runs against a capable, often expensive model. The quantifier extracts structured facts (NPCs detected, movement destination) from a fixed schema and runs against a cheaper, more predictable model. Routing both calls through the same connection would either under-spend on narration or overspend on the quantifier.

The wiring keeps the two paths independent. `AppSettings` carries separate `narration_connection_id` and `quantifier_connection_id` strings; each resolves to its own `LlmProviderConfig`; the bootstrap builds a dedicated `LlmCallRecorder` for each. The `AgentRunner` receives the quantifier's recorder at construction and binds it to the quantifier agent at wiring time. The `Narrator` holds the narration recorder. The recorder is bound at wiring time, not at dispatch time, because the wiring already resolved which connection to use when it built the recorder.

`BackendSelector` (`UseMain | UseNamed(String)`) is metadata today. The selector is retained on the config because it carries intent the wiring can honour in future — it is a seam the design has not closed.

## Hooks, and the ones that wait for their first Agent

The split separates what a generation's own analysis lands in (the patch that the game rules apply) from what outlives the generation (the offered set that the Options dock shows). The design names two more hooks, before narration and rewrite. Each comes with its first Agent, because a hook with no Agent never runs.

## The quantifier's role

The quantifier prompt returns a JSON object with two fields: `npcs_in_room` (the list of NPC ids present in the current room) and `movement` (entering, leaving, or null). The model reports current presence. NPC `Entered` and `Left` events are derived by the engine, by diffing the previous quantifier result against the current one.

Presence detection and transition reasoning are different jobs. Presence detection is structured extraction over a fixed schema: the model names the NPCs it can see in the scene. Transition reasoning would require the model to track prior state across calls and infer who moved since the last result, which is less reliable than a set diff the engine computes over two lists it already holds. Asking the model for the harder job would couple observation and inference in the same call.

The orchestration layer couples the quantifier call to the NPC reconciliation step: the engine remembers the previous result, diffs the two lists, and feeds the resulting events into the trigger pipeline. The deterministic source of truth for NPC transitions is the diff, which runs over inputs the engine already holds.

## Document References

- `../reference/narrative/agent_system.md` — reference description of the agent machinery.
