# Decide which parts of a Narrator-role call an author can change

Type: grilling
Status: resolved
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
- Room for memory and state, from [Decide whether this map designs memory and state, or only keeps room for them](16-memory-and-state-scope.md): the layout model has no closed list of block types and no fixed place for a new block (before or after the history). The stepped history cut is one explicit engine step that knows which Messages it dropped and on which cycle; it fires no hook and saves nothing (it is the future compaction point). No connection slot, preset or failure rule is tied to exactly the Quantifier and Options.
- Caching results ([Find how prompt caching works for our providers and our layout](12-prompt-caching.md)): today the shared prefix is about 28% of a Narration call. The cheap rules are: sort the cards by id, drop the `(in room)` / `(elsewhere)` marks from `KnownCharacters`, put the parts that change after the stable parts, and move the history cut in steps. Decide here also whether `CharactersPresent` is sorted: today it keeps the Quantifier's order (`game_rules.rs:67`). Read the coordinator notes in that answer: the recency rule for the style and format sections still fits, and `GameState` after the history may be the better place.

## Answer

Resolved with the owner, 2026-10-31 (grilling, three rounds).

**Basis (owner).** This ticket fixes only the measured faults. No finding so far shows that a different layout gives a better story: no trial changed the order, the roles or the labels, and the criteria and the judge are not yet defined. So the layout stays an engine rule, and a layout change comes only from a trial that shows an effect on the output or on the cache. This is model C with no new text fields. Models A (an ordered block list in each preset) and B (a switch for each layer) are not chosen. The shared reason "fixed until a trial shows a change" is a plan, so it lives on the map, not in `prompt_system_design.md`.

**The parts that stay engine rules:**

- The order of the layers, the system and user split, and the history as one text block in the user message. No block gets a role or a depth.
- The tags and the data labels (`Current Room:`, `Description:`, `Personality:`, `Narrator:`, the Persona name), under the written rule "XML tags as content labels".
- The instruction sentences: the Event Narration frame, the Guide sentence and the cut marker.
- The history cap, the sanitizer, `single_user_message`, and the layer set of each call kind (Impersonate drops `Persona` and `Guide`).
- No preset variables. The length words stay preset text ([Decide the parameters of each call](07-call-parameters.md)).

**Fixes for the build plan:**

1. **Sort `KnownCharacters` and `CharactersPresent` by Character id.** Reason: a random order has no purpose, breaks the cache and makes two trials of one setup differ. Today `CharactersPresent` keeps the Quantifier's order (`game_rules.rs:67`), and both card lists come from a `HashMap`.
2. **The stepped history cut.** The cut is one explicit engine step that knows which Messages it dropped and on which cycle ([Decide whether this map designs memory and state, or only keeps room for them](16-memory-and-state-scope.md)). It moves in steps, so a trial past the cap keeps its prefix cached. The build plan chooses the step size, and the check trial confirms it.
3. **A layer with no content renders nothing, not even its tags.** One rule for every layer. It removes the empty `<PlayerInput>` block at the end of an Impersonate call with no direction (both trials show it). Whether other layers render empty tags today is for the build plan to check (inferred).
4. **Rename the card label `Context:` to `Scenario:`.** Reason: the label gives the card's `scenario` field the meaning of a backstory ([Compare our card and World format with ST and ME](04-card-and-world-format.md)).

**Docs.** `prompt_system_design.md` keeps its four reasons (the system and user halves, the recency of style and format, content labels, two tiers of cards) and gets the reasons of fixes 1 to 4. It does not state plans.

**Room for memory and state (a build-plan constraint).** The assembler renders an ordered list of layers in code, so a new layer is one new entry at any place in that list. No slot is fixed for a new layer.

**Layout trials (to the map's fog).** These candidates are not changed now. A trial must test them, in a sequence that the build plan and the first check trial will inform:

1. Remove the `(in room)` / `(elsewhere)` marks from `KnownCharacters` (caching rule 2).
2. Move `GameState` after the history (caching rule 3 and the coordinator note).
3. Send `example_dialogue` (it holds the ST `first_mes`; no prompt reads it today).
4. Author-editable instruction sentences (the Event Narration frame, the Guide sentence, the cut marker).
5. The history as chat messages, not one text block.
6. An author block order (model A).

How a trial runs a layout change (a code branch on a probe server, or switches in the test setup file) waits for the build plan (owner: hard to know before it).
