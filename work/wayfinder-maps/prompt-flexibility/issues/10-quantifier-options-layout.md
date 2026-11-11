# Decide which parts of a Quantifier or Options call an author can change

Type: grilling
Status: resolved
Blocked by: 01, 02, 06, 13, 14

## Question

For each fixed part of a Quantifier call and an Options call that [List what reaches the model in each call](01-what-reaches-the-model.md) found: does it become changeable by an author, or does it stay an engine rule with a written reason? Does the layout model of [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md) also fit these calls?

## Context

- [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md) chose "faults only": the Narrator-role layout stays an engine rule until a trial shows a change helps the output or the cache. Apply the same basis here unless a finding argues otherwise.

- Both calls have their own prompt code (`application/agents/quantifier/prompt.rs`, `application/agents/options/prompt.rs`) with their own tags (`<available_rooms>`, `<RecentHistory>`, `<NpcsInArea>`…).
- Both parse the model's answer (`agents/quantifier/utils/parser.rs`, `agents/options/utils/parser.rs`). The parts that a parser depends on may need to stay engine rules.
- The Quantifier failed every cycle in the first Autoplay run (`content_filter` 400), and the game state stayed stale with no visible sign. That run used `openai/gpt-4o-mini` through OpenRouter, and Azure's filter rejected the prompt. On OpenCode Go `deepseek-v4.1-flash`, 4 of 178 Quantifier calls failed (see [Decide what the state does when a state writer fails](15-failed-state-writer.md)). Which model it uses is authoring and out of scope; how a failure shows is in [Shape the test setup and the trial](08-test-setup-and-trial.md).
- The Quantifier is today the only Agent that writes state. A memory writer or a state writer would be another Agent at a Hook: see [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md) and [Find how ST, ME and other tools keep long-term state](14-long-term-state.md).
- Room for memory and state, from [Decide whether this map designs memory and state, or only keeps room for them](16-memory-and-state-scope.md): no new hook; no connection slot, preset or failure rule is tied to exactly the Quantifier and Options; the layout model has no closed list of block types. A future writer is a new Agent on `AfterNarrationAgent` or a new hook trait.
- From [Shape the test setup and the trial](08-test-setup-and-trial.md) (known): `<LatestNarration>` always starts with the player name, also when the entry is a Narration (`Julian: Gabriella received…`). The Quantifier also runs after each Impersonate Input. See [the trial prototype](../assets/trial-prototype.md), findings 1 and 9.

## Answer

Resolved with the owner, 2026-10-31 (grilling, five rounds).

**Basis (owner).** The same basis and model as [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md): faults only, model C, no new text fields. The tags, the order of the blocks and the closing sentences stay engine rules. A layout change comes only from a trial. The one new author handle is the Options count (fix 4), because the owner does not accept a count in the engine text.

**Fixes for the build plan:**

1. **`<LatestNarration>` becomes `<LatestMessage>`, with no label.** The body is the new text only. The `Julian: ` prefix goes away, and no `sender` attribute is added (owner: the XML only structures data, and attributes make it harder to read). Reason: the Quantifier reads a Narration, an event Narration or an Impersonate Input, so the old tag and the old label were wrong for some of these. The closing block and the default Quantifier preset (`data/prompt_presets/quantifier/default.json`, which names the tag four times) change to the new name. Stored preset rows keep the old name, because the seed writes only absent rows. The build plan covers them.
2. **The block stays separate from the history (owner).** The new text does not become the last `<Entry>` of `<RecentHistory>`. Two reasons. First, the shape does not change the cache: on the 174 good Quantifier calls of trial B, with the card list sorted, the shared prefix is 45 % in both shapes. It ends in `<RecentHistory>` 132 times, because the window of 4 moves by one Message on every call (measured). Second, ME keeps the same split: the new reply goes in a separate `<assistant_response>` block after the history, and the history ends before the reply (`agent-executor.ts` `buildAgentMessages`, `generate.routes.ts:5626`, pinned commit `457e827`).
3. **Card text renders its placeholders.** The card `description` in `<PreviousCharactersPresent>` (Quantifier) and in `<CharactersPresent>` (Options) uses the same placeholder render as the Narrator-role calls. Today `{{user}}` reaches the model as-is (`the 30-year-old bodyguard of {{user}}`). It is one render rule for card text in every call.
4. **The Options count comes from the Options preset.** The Options preset gets an `option_count` field. If the field is absent, the count is 3. `{{option_count}}` and the parser use this value, and the parser keeps at most that number. `DEFAULT_OPTION_COUNT` is used only as the value when the field is absent. The dock shows any number (known). The field can be set in `data/` JSON. Its UI field goes to the map's fog item "A UI for the new changeable fields".
5. **No count in the engine text.** The Options closing line drops "exactly 3": "Based on the scene above, respond with options in the format the system instructions specify. Each option is a possible player action — something {persona} could do next — not a story outcome." The fallback sentence for a blank preset keeps `{{option_count}}`, so with a blank preset the count is 3.
6. **One sort rule for every card list in every call.** Sort by Character id: `<available_character_ids>` (today a `HashMap`), `<PreviousCharactersPresent>` and Options `<CharactersPresent>` (today in the order of the Swipe state). This is the same rule as ticket 06 fix 1. `<available_rooms>` keeps the map order, because that order is author data and is stable.

**The parts that stay engine rules, with their reasons:**

- **The tags and the order of the blocks** in both calls. Rule "XML tags as content labels". No trial shows that a change helps.
- **The history window of 4** (`AGENT_HISTORY_WINDOW`), one value for both calls (owner). No measured fault. ME has a setting for each Agent (`contextSize`, default 5). A future state writer sets its own window ([Decide whether this map designs memory and state, or only keeps room for them](16-memory-and-state-scope.md)).
- **The closing sentences.** The Quantifier closing block (with the new tag name) and the Options closing line (without the count). Item 4 of the layout-trial list ("author-editable instruction sentences") covers these calls too.
- **The Options fallback sentence** for a blank or unreadable preset. Reason: the parser needs a known answer shape.
- **The answer format stays preset text (owner, option A).** The Quantifier JSON keys and the Options `<suggestion>` tags are in the preset, and an author can change them. A changed format makes every parse fail. The state then stays as it was, and the banner shows ([Decide what the state does when a state writer fails](15-failed-state-writer.md)). `docs/diataxis/reference/narrative/agent_system.md` lists the formats that each parser accepts. ME does the same: the format is in the Agent's author template.

**Not decided here.**

- **Authoring.** The default Options preset lists 5 aims for 3 Options, and its "next story beat" conflicts with the closing line "a possible player action … not a story outcome" (inferred). Both belong to the authoring effort.
- **Out of this ticket.** Why the Quantifier runs after an Impersonate Input and not after a typed Input (`is_analysed` in `domain/model/generation.rs`) is a plan rule, not a part of a call. Facts for later: in trials A and B, 11 of 28 room changes came from an Impersonate Input.
