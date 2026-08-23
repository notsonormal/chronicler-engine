# Research: option/choice-generation prior art

Type: research
Status: pending
Blocked by: (none)

## Question

Survey prior art for option/choice generation — an engine generating pickable options the player can choose from — to inform the options-autogeneration design (ticket 04). This is an AFK research ticket; it produces a markdown summary as a linked asset and does not make product decisions.

### Scope

1. **The three repos already researched for steering** (parent map tickets 01–03): SillyTavern, Marinara-Engine, GuidedGenerations-Extension. Do any have an option/choice-generation feature? SillyTavern has "swipes" (regenerate alternatives) — check whether swipes or any other affordance is used to *present choices to the player*, not just retry. Marinara and GG are steering extensions — check for any choice/options surface.

2. **Parser-IF engines and engines with explicit choice affordances:** Inform 7, Twine (ChoiceScript-adjacent), Choice of Games / ChoiceScript, AI-driven IF (AI Dungeon, NovelAI's "lore" or options, anything that generates choices). How do they generate, render, and let the player pick options? What is the prompt structure, the count, the selection flow?

3. **The key design questions to extract prior-art answers for** (these feed ticket 04, this ticket does NOT decide them):
   - Trigger model: on-demand (player asks for choices) vs. always (every turn offers choices) vs. both. What do existing systems do?
   - Generation: separate LLM call? Same call as narration? New agent/phase? What prompt structure produces N coherent, distinct, non-overlapping options?
   - Selection flow: does picking an option submit it as the player's input (and trigger narration), or does it do something else?
   - Retry: can an option set be regenerated (swipe-style)?

## Notes for the session

- Parent map research assets: `.scratch/steering-and-guided-generation/research/01-marinara-engine.md`, `02-guided-generations-extension.md`, `03-sillytavern-core.md` — re-scan these for any option/choice mentions before re-fetching the repos.
- Repos: https://github.com/notsonormal/Marinara-Engine, https://github.com/Samueras/GuidedGenerations-Extension, https://github.com/SillyTavern/SillyTavern.
- Produce `research/03-option-generation-prior-art.md` as a linked asset (verbatim quotes for any mechanism found; "no prior art found" is a valid, useful result for each area).
- Skills: `/research`.
