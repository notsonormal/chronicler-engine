# Idea: Long-term memory and long-term state

Status: seed (not charted)

## Why

Long play and Autoplay lose the story. The history cap drops the oldest story at about cycle 39 of the current World's prose, or about cycle 10 with the 8192-token Ollama default. The engine has no long-term memory. Its only state is the Room, the Characters present and the encounters, and Game 6 records 3 of 11 visible state changes.

## What the prompt-flexibility map leaves for this effort

From [Decide whether this map designs memory and state, or only keeps room for them](../prompt-flexibility/issues/16-memory-and-state-scope.md):

- The layout has an open list of block types and no fixed place for a new block.
- The stepped history cut is one explicit engine step that knows which Messages it dropped and on which cycle. It fires no hook yet.
- A writer is a new Agent on `AfterNarrationAgent` or a new hook trait. It keeps the previous state when it fails.
- Memory and state that depend on the story are pinned to the Swipe. New `SwipeState` fields are optional JSON keys.

## Open questions

- **The trigger.** Save into memory at compaction (a step of the history cut, as in a coding harness), after each scene (the ME shape), or both?
- **The position.** Before the history (changes rarely, breaks the cache only on the cycle it changes) or after it (changes often)? It follows from the trigger.
- Memory for each Character (ME's audience list and first-person recollections) or one summary for each Game (ST)?
- Typed state: which fields, written by which call, and how a card change gets a person's approval and a version.
- Caps and cost of each writer call.

## Research to start from

- [Find how ST, ME and other tools keep long-term memory](../prompt-flexibility/issues/13-long-term-memory.md) ([asset](../prompt-flexibility/assets/long-term-memory.md))
- [Find how ST, ME and other tools keep long-term state](../prompt-flexibility/issues/14-long-term-state.md) ([asset](../prompt-flexibility/assets/long-term-state.md))
- [Find how prompt caching works for our providers and our layout](../prompt-flexibility/issues/12-prompt-caching.md) ([asset](../prompt-flexibility/assets/prompt-caching.md))
- [Decide what the state does when a state writer fails](../prompt-flexibility/issues/15-failed-state-writer.md)
