# Decide whether this map designs memory and state, or only keeps room for them

Type: grilling
Status: resolved
Blocked by: —

## Question

Does this map design long-term memory and long-term state, so the build plan includes them? Or does it only keep room for them in the layout and the Agents, and a new effort designs them?

## Context

- The owner sees memory and state as needed for long play and Autoplay: memories added to Characters, and state that changes (map Notes).
- [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md): no stored story reaches the history cut; this World's prose passes it at about cycle 39 (inferred). ME is the only production tool with a memory for each Character. One cheap call per scene is the cheapest shape that still names who was there. Memory text is always under a cap (10 per cent of the context in ME).
- [Find how ST, ME and other tools keep long-term state](14-long-term-state.md): ME writes typed JSON state per Message and Swipe in a second call after each reply. Game 6 records 3 of 11 visible state changes.
- [Find how prompt caching works for our providers and our layout](12-prompt-caching.md): a memory or state block after the history keeps the history cached (inferred, see the coordinator notes).
- What "keep room" would mean, at least: a place in the layout for a memory block and a state block, an Agent seam that a writer can use (a Hook and a failure rule, see [Decide what the state does when a state writer fails](15-failed-state-writer.md)), and Swipe-pinned storage.
- The coordinator leaned to "keep room here, design in a new effort", because this map's destination is about changeable call parts, and memory is a feature the size of Autoplay. The owner decides.

## Output

A resolution comment with the choice. If the choice is "keep room", list the minimum room this map must keep. If it is "design here", redraw the Destination and add the tickets that the design needs.


## Answer

Resolved with the owner, 2026-10-31.

**Choice: keep room.** This map does not design long-term memory or long-term state. A new effort designs them: [seed](../../long-term-memory-and-state/idea.md). The Destination does not change.

**The minimum room this map keeps:**

1. **The layout ([ticket 06](06-the-layout-model.md), [ticket 10](10-quantifier-options-layout.md)).** The layout model does not fix a closed list of block types. It does not fix where a new block goes (before or after the history). Reason (owner): the position of a memory block depends on how often its text changes, and that depends on the memory design. Text that changes rarely can sit before the history and breaks the cache only on the cycle that changes it. Text that changes on every cycle goes after the history ([long-term memory asset](../assets/long-term-memory.md), section 5). The owner rejected empty `memory` and `state` slots.
2. **The history cut (ticket 06).** The stepped history cut (cheap rule 4 of [Find how prompt caching works for our providers and our layout](12-prompt-caching.md)) is one explicit engine step. The step knows which Messages it dropped and on which cycle. It fires no hook and saves nothing. Reason (owner): a step of the cut is the compaction point, as in a coding harness. A part of the story leaves the prompt at that moment, so it is the natural moment to save the part into memory. A memory written only at a step does not break the cache between steps (inferred). Whether the new effort saves at compaction or after each scene (the ME shape) stays open.
3. **The Agents (tickets 06, 07, 10).** The build plan adds no new hook. No connection slot, preset or failure rule is tied to exactly the Quantifier and Options. A writer is a new Agent on `AfterNarrationAgent` (`src/application/agents/trait_def.rs`), or a new hook trait that the new effort adds. It keeps the failure rule of [Decide what the state does when a state writer fails](15-failed-state-writer.md).
4. **The storage.** `SwipeState` and the schema do not change. Written rule: memory and state that depend on the story are pinned to the Swipe, so a Retry discards them. Later fields are optional JSON keys, as `trigger_context` is today.

**Known effect.** Trials still lose the oldest story after about cycle 39 of this World's prose. A comparison of two setups is still fair, because both lose the same part.
