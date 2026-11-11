# Research: option/choice-generation prior art

Type: research
Status: resolved
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

## Answer

Research asset: [`research/03-option-generation-prior-art.md`](../research/03-option-generation-prior-art.md). Verbatim-grounded survey across the three steering repos, ChoiceScript, Twine/SugarCube, Inform 7, AI Dungeon (Classic + v2+), and NovelAI.

**Key findings (one-line gists; detail in the asset):**

- **Three steering repos (SillyTavern, Marinara, GG-Extension): no choice affordance.** Their shared "swipe" is regeneration retry of the same turn, not choice presentation.
- **Two generated-choice SillyTavern extensions (added in revision): ST-CYOA and ST-Roadway.** Both generate a pickable option set via a SEPARATE LLM call (not the narration call), and both turn a chosen option into the player's next input (via impersonate or paste-and-send). CYOA = on-demand `/cyoa`, 1–10 options, `<suggestion>` tags, transient. Roadway = on-demand button AND auto-on-every-message (incl. swipe/continue), 6 options, numbered-list + domain-diversity prompt, routes to a cheap/local connection profile, stores options on a system message. These are the strongest mechanistic prior art for the Chronicler Engine's options-autogeneration.
- **ChoiceScript & Twine/SugarCube: AUTHORED choices, not generated.** Author writes option set + branch; selection navigates to an authored node, does not submit as input.
- **Inform 7: parser IF, no choice affordance** in the core loop.
- **AI Dungeon Classic (GPT-2, May 2019): generated choices folded into the narration call (always-on, selection-as-input); abandoned in AI Dungeon 2** for freeform Do/Say/Story text.
- **AI Dungeon v2+ and NovelAI: freeform text + output-level undo/redo, no choice set.** NovelAI's Memory/Author's Note/Lorebook are context injection, not choices.

**Gaps left open for ticket 04:** option-set retry (swipe-style); stable cross-session persistence of an offered-option set; options × steering interaction (none of the surveyed systems composes a steering layer with a choice layer).

**Now CLOSED by the ST extensions (previously gaps):** separate-options-call (attested, dominant); on-demand/always/both triggers (all attested — Roadway ships the hybrid); prompt structure for distinct options (tag-wrapped + domain-diversity numbered list).
