---
diataxis: explanation
title: Agent System Design
---

## The agent abstraction

The trait-plus-runner shape keeps the Job indifferent to which optional Agents exist. Adding an agent of a known type is a wiring change in bootstrap. A new agent type also needs an arm in `AgentRunner::from_configs`. Removing one is removing it from the runner. None of these changes require editing the Job itself.

Each Agent gives one kind of result, and each kind of result has one landing. The compiler rejects an Agent whose answer the hook cannot save.

## Per-agent Connections

The narration LLM call and the quantifier LLM call have different needs. Narration produces the prose the player reads and runs against a capable, often expensive model. The quantifier extracts structured facts (Characters detected, movement destination) from a fixed schema and runs against a cheaper, more predictable model. Routing both calls through the same connection would either under-spend on narration or overspend on the quantifier.

Each Connection slot therefore has its own recorder, and each recorder resolves its own Connection.

## The hooks

The after-narration and end-of-Job hooks separate what a generation's own result lands in (the Quantifier result that the game rules apply) from what outlives the generation (the offered set that the Options dock shows).

The before-narration hook exists for the story memory. The story memory writer must answer before the narrator call, because that call shows its memory entries. The Narrator holds no storage, so it can neither read the earlier memory entries nor save the new ones. So the Job calls the writer and carries its entries to the text save. The writer has its own Role, so the role health shows a failed writer call as a story memory failure.

The story memory writer reads the Messages that leave, the cast names and only the last memory entry written before. So its prompt does not grow with the story. A cut of many Messages goes to several writer calls of at most `MEMORY_WRITER_CALL_TOKENS` each, one after the other. Each call stays far inside any context window. Each later call reads the last entry of the call before it, so the scenes stay one story.

The model names only where each scene starts. The engine sets where each scene ends, so the entries of one call cover its Messages with no gap and no overlap. A model that misjudges an end cannot leave a Message out of the story memory.

A failed call adds nothing, and the calls after it wait. Its Messages go to the next cut, so the entries keep covering the story with no gap and the earlier entries stay. The writer tries a failed call once more with the same prompt, as the Quantifier and the Options agent do. A provider error or a malformed answer is often a one-off, and a failed call costs the Messages a wait.

The wiring builds the story memory writer outside `AppSettings` because the seed of the agent configuration adds no entry to an existing database. A writer configured in `AppSettings` would leave every older database with no story memory.

## The quantifier's role

The model reports which Characters are present now. The engine derives who entered and who left, from a diff of the previous result and the current one.

Presence detection and transition reasoning are different jobs. Presence detection is structured extraction over a fixed schema: the model names the Characters it can see in the scene. Transition reasoning would require the model to track prior state across calls and infer who moved since the last result. This is less reliable than a set diff. The engine computes that diff over two lists it already holds. Asking the model for the harder job would couple observation and inference in the same call.

## Document References

- `../reference/narrative/agent_system.md` — reference description of the agent machinery.
