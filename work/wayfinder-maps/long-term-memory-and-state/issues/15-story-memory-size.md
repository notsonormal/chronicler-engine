# Decide how the story memory keeps its size

Type: prototype (HITL)
Status: open
Blocked by: 14

## Question

How does the story memory stay inside a fixed space for a story of any length, without the loss of what the narrator needs?

The answer is one concrete change to the base story memory, tried on the engine and measured. It states what the engine stores, what each writer call gets and writes, and what happens to the oldest part at the limit.

## Context

- **The base** ([Build the base story memory](14-build-base-story-memory.md)). At each history cut, the writer rewrites one text from the previous text and the Messages that leave. The limit is 2,000 tokens, and the prompt asks for 600 words. An answer at the limit is a failed writer call that keeps the previous text.
- **The problem (known).** The size of the rewritten text is not under control with `deepseek-v4.1-flash`. The model adds the new events and shortens little. Five of nine runs of the memory test reached the limit at the fourth or fifth cut. Two trials in the engine stayed under the limit. No wording of the prompt fixed this, and a wording with two sections made the model return the old memory unchanged.
- **The effect in play (inferred).** After a writer call fails at the limit, the story memory stays at its last good text. The Messages of that cut are in neither the history nor the story memory. The next writer call gets more Messages, so it is likely to fail again.
- **The evidence**: [Evidence: the size of the story memory](../assets/story-memory-size.md). It holds the sizes of every run, the writer calls of both trials, and what is not known.
- **The planned next change** ([Choose the memory candidates to try](05-memory-candidates.md)). The story memory as a list of scene entries: each cut adds entries, and no old entry is rewritten. This removes the rewrite, but a list also grows. It is one possible answer here, not a decision.
- **The owner's rules** (map Notes). The player's story stays whole. The coordinator chooses the design details from the research, states them, and lets a trial judge. The owner gives a go or no-go on the concrete change, and then gets the results.
- **The research**: [Find how Marinara Engine keeps memory with its agents](01-marinara-memory.md) and [Find what is known about long-term memory for LLMs, and what is still open](02-llm-memory-research.md) describe how other systems bound a memory (a second level of summary, a cap on entries, loading by a key).

## What the answer needs

- **A measurement over more cuts.** The stored story of the memory test has 5 cuts, and a 100-Cycle trial has 5 or 6. The problem shows at the fourth cut and later. A smaller history size gives more cuts on the same story.
- **The engine's rule in the measurement.** The memory test keeps a text that was cut off and continues. The engine counts it as a failed call. A score that hides this is not the score of the engine.
- **More than one run for each change.** Runs differ, and at temperature 0 some runs repeat.
- **The cost.** The writer calls of the base use under 1 % of the tokens of a trial, and the cached share of the Narrator-role calls stays at 85.5 %. The change must report both.

## Output

The answer names the change, its measured sizes and scores against the base, and the build ticket that follows.
