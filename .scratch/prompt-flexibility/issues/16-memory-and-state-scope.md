# Decide whether this map designs memory and state, or only keeps room for them

Type: grilling
Status: open
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
