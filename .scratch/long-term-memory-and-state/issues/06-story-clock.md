# Decide how the story keeps its time

Type: grilling
Status: open
Blocked by: 01, 02

## Question

What does the engine record about the time of the story, which call writes it, and where does it show? Or does a memory that records time remove the fault, so that no separate clock is necessary?

## Context

- **The fault** (section 5 of [the inconsistencies of two trial stories](../../layout-trials/assets/story-inconsistencies.md)). The World says "Bernard Redmist died two weeks ago". The engine records no story time, so the model invents durations: "four days" in the house when the story covers the second day, and "two weeks" in the house after one day. The judges of ticket 15 of Layout trials counted 4 such faults in the newest story and 2 in the reference story (`tmp/lt15/judge/`).
- **The baseline (known, [Count the memory faults of the newest trial](03-baseline-faults.md)).** After the check, the newest story has 4 clock faults: the time of day (twice), the time that passed, and a World time fact ("three weeks ago, after the funeral"). Eight more are date conflicts inside the history: the year of the Draught is 1905, then thirteen, fifteen and thirty years ago. A list of fixed facts can prevent those (guessed). This map puts them out of scope, so this ticket decides whether the clock covers them.
- **What a clock can record.** The day of the story, the time of day, the time since the start, the time since a key event, and the date of the World.
- **ME (known, [how other tools keep long-term state](../../prompt-flexibility/assets/long-term-state.md), sections 1 and 2.2).** The World State of ME has `date`, `time`, `weather` and `temperature`. The narrator model changes only `location` and `time`, by a tool in its own call. A tracker call owns every other field.
- **Where a writer can run.** In the Narration call (a tool or a field of the answer), in the Quantifier (it already reads each new text and gives typed JSON), in a new Agent, or in a memory writer.
- **The cache.** A state that changes on most Cycles goes after the history. `<GameState>` already sits after the history (Layout trials).
- **The rules of this map** (map Notes). State that depends on the story is pinned to the Swipe. A failed writer keeps the previous state.
- **The research.** [Find how Marinara Engine keeps memory with its agents](01-marinara-memory.md), point 4, and [Find what is known about long-term memory for LLMs, and what is still open](02-llm-memory-research.md), point 5.

## Output

A resolution with:

- the clock, or the reason that no clock is needed;
- its fields, its writer, and its place in the prompt;
- how the time questions of the fact list of [Count the memory faults of the newest trial](03-baseline-faults.md) test it.

If the clock needs its own candidate in the memory test, create that ticket.
