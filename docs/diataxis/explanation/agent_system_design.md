---
diataxis: explanation
title: Agent System Design
---

## The agent abstraction

The trait-plus-runner shape keeps the Job indifferent to which post-processing concerns exist. Adding an agent of a known type is a wiring change in bootstrap; a new agent type also needs an arm in `AgentRunner::from_configs`. Removing one is removing it from the runner. None of these changes require editing the Job itself. Each Agent gives one kind of result, and each kind of result has one landing. The compiler rejects an Agent whose answer the hook cannot save.

## Per-agent Connections

The narration LLM call and the quantifier LLM call have different needs. Narration produces the prose the player reads and runs against a capable, often expensive model. The quantifier extracts structured facts (Characters detected, movement destination) from a fixed schema and runs against a cheaper, more predictable model. Routing both calls through the same connection would either under-spend on narration or overspend on the quantifier.

Each role therefore has its own recorder, and each recorder resolves its own connection.

## Two hooks

The two hooks separate what a generation's own result lands in (the Quantifier result that the game rules apply) from what outlives the generation (the offered set that the Options dock shows). A before-narration hook gets code with its first Agent, because a hook with no Agent never runs.

## The quantifier's role

The model reports which Characters are present now. The engine derives who entered and who left, from a diff of the previous result and the current one.

Presence detection and transition reasoning are different jobs. Presence detection is structured extraction over a fixed schema: the model names the Characters it can see in the scene. Transition reasoning would require the model to track prior state across calls and infer who moved since the last result, which is less reliable than a set diff the engine computes over two lists it already holds. Asking the model for the harder job would couple observation and inference in the same call.

## Document References

- `../reference/narrative/agent_system.md` — reference description of the agent machinery.
