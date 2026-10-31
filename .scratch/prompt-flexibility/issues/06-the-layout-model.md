# Decide which parts of a Narrator-role call an author can change

Type: grilling
Status: open
Blocked by: 01, 02, 04, 12, 13, 14, 16

## Question

For each fixed part of a Narrator-role call (Narration, Event Narration, the regenerate of a user Input, Impersonate) that [List what reaches the model in each call](01-what-reaches-the-model.md) found: does it become changeable by an author, or does it stay an engine rule with a written reason? Which layout model carries the changeable parts?

## Context

- Candidate models, to test against the findings: (A) an ordered template of blocks, where each block is author text or a marker for an engine layer (as ST's Prompt Manager and ME's sections); (B) a fixed layout with a switch for each layer and author text for tags and labels; (C) a fixed layout where only named texts move into preset fields (for example the length rule, the history labels, the empty direction block).
- The engine rules in `docs/diataxis/explanation/prompt_system_design.md` already have written reasons (system and user halves, the style and format sections after the history, XML tags as content labels, two tiers of NPC cards). Each one either stays a rule with that reason or becomes changeable.
- The Quantifier and Options are a separate ticket: [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md).
- The card fields that [Compare our card and World format with ST and ME](04-card-and-world-format.md) finds missing may need a place in the layout (for example a card's own post-history text).
- Resolved findings of that ticket ([asset](../assets/card-and-world-format.md), sections 1, 2 and 6): no card field holds dialogue examples, a per-card system or post-history text, a depth prompt or keyword lore; `example_dialogue` is stored but no prompt part reads it; the prompt label `Context:` gives `scenario` the meaning of a backstory.
- A provider cache reads only a prefix, so the order of the parts sets the cost of each call: see [Find how prompt caching works for our providers and our layout](12-prompt-caching.md).
- The history cut is one of the fixed parts. [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md) measures what the cut loses in a long story, and lists the memory methods that could replace or add to it. [Find how ST, ME and other tools keep long-term state](14-long-term-state.md) lists the state that long play changes. The layout must have room for both, even if this map does not design them.
- Caching results ([Find how prompt caching works for our providers and our layout](12-prompt-caching.md)): today the shared prefix is about 28% of a Narration call. The cheap rules are: sort the cards by id, drop the `(in room)` / `(elsewhere)` marks from `KnownCharacters`, put the parts that change after the stable parts, and move the history cut in steps. Decide here also whether `CharactersPresent` is sorted: today it keeps the Quantifier's order (`game_rules.rs:67`). Read the coordinator notes in that answer: the recency rule for the style and format sections still fits, and `GameState` after the history may be the better place.
