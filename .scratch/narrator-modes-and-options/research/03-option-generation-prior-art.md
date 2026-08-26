# Research: option/choice-generation prior art

Asset for wayfinder ticket `03-research-option-generation-prior-art` (map: `narrator-modes-and-options`).
AFK research — produces findings only, makes no product decisions. Verbatim quotes for every mechanism found; "no prior art found" is recorded where so.

Scope reminder: the Chronicler Engine question is about **GENERATED** choices — the engine produces a pickable option set the player chooses from. **AUTHORED** choices (human-written nodes) are flagged as such; they are prior art for the *shape* of choice affordances but not for *generation*.

**Revision note (2026-08-26):** Section 1 originally concluded "no prior art found" among the three *steering* ST extensions surveyed for the parent map. Section 1b (added later) corrects the record: two **non-steering** SillyTavern extensions — `Sillytavern-CYOA` and `SillyTavern-Roadway` — do generate a pickable option set. These are the strongest generated-choices prior art in the whole survey.

---

## 1. The three steering repos (SillyTavern, Marinara-Engine, GuidedGenerations-Extension)

Re-scanned the parent-map research assets (`.scratch/steering-and-guided-generation/research/01-marinara-engine.md`, `02-guided-generations-extension.md`, `03-sillytavern-core.md`).

**Finding: none of the three has an option/choice-presentation affordance.** Their shared "swipe" mechanism is **regeneration retry** (produce an alternative to the same turn), not choice presentation. A swipe creates a new alternative on the *last* message; it does not present a set of distinct forward options for the player to pick one from.

- **SillyTavern swipes = regenerate alternative.** Retry/swipe/regenerate are listed alongside `regenerate` as generation types that do *not* save a new user message: `if (type !== 'regenerate' && type !== 'swipe' && type !== 'quiet' && !isImpersonate && !dryRun && !depth)` (`public/scripts/script.js:4340`, quoted in `03-sillytavern-core.md`). The ephemeral guide inject "does not survive the end of the previous attempt" on retry (`03-sillytavern-core.md`). There is no surface that offers the player N pickable in-story choices.
- **Marinara-Engine swipes = regeneration with replay.** Messages carry `activeSwipeIndex: integer("active_swipe_index").notNull().default(0)` (`01-marinara-engine.md:221`); a regeneration stores a `generationReplay` blob in the previous message's `extra` and replays the guide into the new request (`01-marinara-engine.md:159`). This is retry-of-the-same-turn, not choice presentation.
- **GuidedGenerations-Extension "Guided Swipe" = guided regeneration.** `guidedSwipe.js` builds an inject then calls `context.swipe.right()` to "create a new swipe on the last (AI) message" (`02-guided-generations-extension.md:69-74`). Again, retry, not choices.

**No prior art found** in these three for an option/choice-generation feature. (This is the expected result — they were researched for steering, and swipes were already characterized as retry in the parent map's `04-design-synthesis.md:19,22`.)

---

## 1b. Two generated-choice SillyTavern extensions (CYOA, Roadway) — GENERATED choices

Primary sources: `https://github.com/vitorfdl/Sillytavern-CYOA` (repo cloned to `/tmp/pi-github-repos/vitorfdl/Sillytavern-CYOA`), `https://github.com/bmen25124/SillyTavern-Roadway` (cloned to `/tmp/pi-github-repos/bmen25124/SillyTavern-Roadway`). Verbatim quotes from `index.js` / `src/index.ts` and the READMEs below.

**Both extensions generate a pickable option set on demand and let the player turn a chosen option into their next input.** This is direct generated-choices prior art — the closest mechanistic analogue to the Chronicler Engine's planned options-autogeneration.

### 1b.1 SillyTavern-CYOA (`vitorfdl/Sillytavern-CYOA`, archived)

README: "This extension adds Choose Your Own Adventure (CYOA) style responses to your SillyTavern chats. It generates multiple response options for the user to choose from, enhancing interactivity and allowing for branching narratives."

- **Trigger: on-demand.** A slash command, `/cyoa`, triggers generation (`index.js` exposes the command; README "Usage" → "Use the `/cyoa` slash command in chat to generate CYOA options"). No auto-on-every-turn mode.
- **Generation: a SEPARATE quiet LLM call, distinct from narration.** It calls SillyTavern's `generateQuietPrompt(prompt, false, !useWIAN, null, "Suggestion List", responseLength)` (`index.js`, `requestCYOAResponses`), not the main generation path. The prompt is user-configurable with a `{{suggestionNumber}}` placeholder. Default prompt (verbatim from `index.js:23`):
  > `Stop the roleplay now and provide a response with {{suggestionNumber}} brief distinct single-sentence suggestions for the next story beat on {{user}} perspective. ... Each suggestion surrounded by \`<suggestion>\` tags. ... Do not include any other content in your response.`
- **Option count: 1–10, user-configurable** (`response_length` slider; README "Number of Responses").
- **Parsing: the model is instructed to wrap each option in `<suggestion></suggestion>` tags**, and the parser also accepts `Suggestion N: text...` as a fallback (`index.js:51` regex: `/<suggestion>(.+?)<\/suggestion>|Suggestion\s+\d+\s*:\s*(.+)|...`).
- **Selection flow: picking an option impersonates it as the user's input.** Clicking a suggestion button runs `/impersonate await=true [Event Direction... {{suggestionText}}]` (`index.js:181`), substituting the chosen text into the impersonate prompt. The impersonate output becomes the user's next message. An **Edit** button copies the suggestion to the input box instead, for the player to modify before sending.
- **Retry: re-run `/cyoa`.** No swipe-style regeneration of the option set itself. The previous CYOA suggestion-list message is removed before a new generation (`removeLastCYOAMessage`, `index.js`).
- **Persistence: transient.** The suggestion list is injected as a pseudo-user message with `extra.model: 'cyoa'` (`index.js:151`) and deleted on the next `/cyoa` run. Options are not persisted as a reusable artifact.
- **Context inclusion: optional World Info / Author's Note** toggle (`apply_wi_an`, README "Apply World Info / Author's Note").
- **Inspired by:** LenAnderson's gist (`https://gist.github.com/LenAnderson/7686604c9da30dee21b76a633a0027f4`).

### 1b.2 SillyTavern-Roadway (`bmen25124/SillyTavern-Roadway`, active)

README: "A SillyTavern extension that helps you to make decisions about the story. It could give an idea." Author's framing: "is this just a simple LLM request? Yes... This extension is a shortcut." (README FAQ)

- **Trigger: on-demand button OR auto-on-every-AI-message.** A per-message button (`.mes_magic_roadway_button`) generates options for that message. An `autoTrigger` setting auto-fires the button on `CHARACTER_MESSAGE_RENDERED` (`src/index.ts:795`), including after `swipe`/`continue` in group chats (`allowed_group_types: ['normal','continue','swipe']`, `src/index.ts:807`). So Roadway supports **both on-demand and always-on**.
- **Generation: a SEPARATE request through a dedicated connection profile, distinct from the main API.** It builds the prompt via `buildPrompt(...)` and sends through `ConnectionManagerRequestService.sendRequest(profileId, ...)` (`src/index.ts:~435`). A key design point: the author uses a **cheap/local profile for option-generation** while the main chat runs on an expensive model (README FAQ: "your main API can be Claude Sonnet... But you can use this extension with some cheap/local API"). This is a two-agent / two-model pattern.
- **Prompt structure: asks for a numbered list, fixed count of 6, with a domain-diversity instruction.** Default prompt (verbatim, `src/index.ts:58`):
  > `Output ONLY a numbered list of possible actions... Prioritize *varied* actions that span multiple domains: {Observation/Investigation; Dialogue/Persuasion; Stealth/Intrigue; Combat/Conflict; Crafting/Repair; Knowledge/Lore; Movement/Traversal; Deception/Manipulation; Performance/Entertainment; Technical/Mechanic}. ... Generate *exactly* 6 actions.`
  Extraction is configurable: a `bullet` extraction strategy pulls bullet points from the response (`extractBulletPoints`, `src/index.ts`).
- **Option count: 6 by default** (hardcoded in the default prompt; configurable by editing the prompt preset).
- **Selection flow: three per-option actions.** Each rendered option has three buttons (`src/index.ts` `attachRoadwayOptionHandlers`):
  1. **Impersonate (✍️):** runs the impersonate preset with `{{roadwaySelected}}` substituted into `DEFAULT_IMPERSONATE` (verbatim, `src/index.ts:51`): `Your task this time is to write your response as if you were {{user}}, impersonating their style... This is what {{user}}'s focus: {{roadwaySelected}}`. Can run through the main API or a dedicated impersonate profile (`impersonateApi: 'main' | 'profile'`).
  2. **Use action (▶️):** copies the option text into the input box; with `autoSubmitUseAction` on, it also clicks send (`src/index.ts:711`).
  3. **Edit:** opens an inline textarea to edit the option before using it (`src/index.ts:733`).
- **Retry: re-run the button (or autoTrigger refires on swipe/continue).** No swipe-style regeneration of the option *set* as a first-class gesture; re-running the button overwrites the existing roadway message for that target (`existMessage` branch, `src/index.ts`).
- **Persistence: a system message is stored on the chat.** Options live in `message.extra[KEYS.EXTRA.OPTIONS]` and `RAW_CONTENT` on a system message targeted at the source message (`src/index.ts:462`). Survives across re-renders; overwritten on regeneration.

### 1b.3 Cross-cutting — what the two extensions agree on

- **Generated, separate call, not the narration call.** Both use a distinct generation path (CYOA: `generateQuietPrompt`; Roadway: `ConnectionManagerRequestService.sendRequest`), not the main narration generation. (Inference: the surveyed generated-choices prior art uniformly separates option-generation from narration.)
- **Selection-as-input, via impersonate or paste-and-send.** Both turn a chosen option into the player's next input — CYOA via `/impersonate`, Roadway via impersonate OR "use action" (paste + optional auto-send). Neither treats the option as a branch selector; it becomes *player input* that the narrator then continues from.
- **Tag/list prompting, not freeform.** CYOA: `<suggestion>` tags. Roadway: numbered list + bullet extraction. Both instruct the model to output *only* the options.
- **Retry = re-run, not swipe.** Neither regenerates an option set as a swipe gesture; both just re-run the generation (Roadway overwrites; CYOA deletes-then-regenerates).
- **Divergence worth flagging for ticket 04:**
  - **Trigger model:** CYOA = on-demand only; Roadway = on-demand **and** auto-on-every-message (incl. swipe/continue). Roadway is the precedent for "both".
  - **Two-model / cheap-profile pattern:** Roadway routes option-generation to a separate (cheaper) connection profile — a concrete instance of "options as a separate agent/phase with its own model."
  - **Domain-diversity instruction:** Roadway's prompt explicitly demands options spanning distinct action-domains to avoid the "models suggest the same things" failure the author calls out in the FAQ — a documented technique for *distinct, non-overlapping* options. (Inference: this is the closest prior art to the "N coherent, distinct, non-overlapping options" design question.)

---

## 2. ChoiceScript (Choice of Games) — AUTHORED choices

Primary source: Choice of Games official docs, `https://www.choiceofgames.com/make-your-own-games/choicescript-intro/` and `.../important-choicescript-commands-and-techniques/` (fetched to `/tmp/choicescript_intro_text.txt`, `/tmp/choicescript_commands_text.txt`).

**Finding: ChoiceScript choices are fully human-authored, not generated.** The author writes the option set and the branch each option jumps to.

- The `*choice` command defines an option set; each `#option` line is a choice, followed by its body and a `*finish` or `*goto`:
  ```
  *choice
  #Make pre-emptive war on the western lands.
  If you can seize their territory, your kingdom will flourish. ...
  *finish
  ```
  (`choicescript_intro_text.txt`)
- `*fake_choice` is "a convenience command [that] behaves exactly like *choice, but no commands are allowed in the body of the choice; thus no *goto / *finish is required" (`choicescript_commands_text.txt`).
- Selection flow: picking an option runs its body then jumps via `*goto`/`*finish` to the next authored node — it does **not** submit the option text as free input to a generator. The choice *is* the branch selector.
- Option gating is authored: `*selectable_if (president) #Abuse my presidential powers...` greys an option; `*if (president) #...` hides it; `*hide_reuse`/`*disable_reuse` prevent reuse (`choicescript_commands_text.txt`).
- Option count: unbounded, author-defined. No retry/regenerate of an option set.

**Design-question answers (authored, not generated):**
- Trigger: always (every `*choice` node presents options). Author-decided placement.
- Generation: **none** — authored nodes.
- Selection: jumps to an authored branch; not submitted as input.
- Retry: none.
- Count: author-defined.
- Persistence: the choice is structural (part of the scene file), not a runtime-generated artifact.

---

## 3. Twine / SugarCube — AUTHORED passage links

Primary source: SugarCube v2 docs, `https://www.motoslave.net/sugarcube/2/docs/` (fetched via `fetch_content`).

**Finding: Twine choices are authored passage links, rendered as clickable links/buttons. Not generated.**

- Passage links are authored in passage text and rendered as `<a>`/`<button>`:
  ```
  <a data-passage="PassageName">Do the thing</a>
  <button data-passage="PassageName">Do the thing</button>
  ```
  (SugarCube docs, "HTML & SVG Attribute → Special Attribute" section)
- SugarCube provides `<<link>>` and `<<button>>` macros (listed under "Interactive Macros" / "Links Macros" in the docs TOC). Links carry an optional Setter: `[img[Title|Image][Link][Setter]]`.
- Selection flow: clicking a link navigates to the linked passage (an authored node). The link does not submit its text as input to a generator; it is a branch selector, like ChoiceScript.
- No native option-generation. A Twine author *could* procedurally populate a link list from variables, but the engine offers no LLM/procedural choice-generation feature.

**Design-question answers (authored, not generated):**
- Trigger: always (links appear where the author placed them).
- Generation: **none** — authored links.
- Selection: navigates to an authored passage.
- Retry: none (no option-set regeneration).
- Count: author-defined.
- Persistence: structural (passage content).

---

## 4. Inform 7 — parser IF, no choice affordance in the core

Primary sources: Inform 7 official docs `https://ganelson.github.io/inform-website/book/WI_18_30.html` ("Clarifying the parser's choice of something"); Inform 7 Handbook `https://inform-7-handbook.readthedocs.io/en/latest/chapter_10_advanced_topics/helping_the_parser`.

**Finding: Inform 7 is parser-IF — the player types commands, the parser matches them to actions. There is no first-class pickable-choice affordance in the core loop.** ("Clarifying the parser's choice" in WI_18_30 refers to disambiguating which object a typed noun refers to, not to presenting story choices.)

- The player types imperative commands (e.g. "take the lantern", "go north"). The parser maps typed text to defined actions. No option set is presented to the player by the engine.
- Inform 7 does have a `menu` system (used for hint menus, conversation topic menus, and help), but it is a UI widget for presenting authored lists, not a story-choice generation mechanism. It is not the core interaction and is not LLM-driven.

**Design-question answers:**
- Trigger: N/A (parser input, not choices).
- Generation: **none** — typed input.
- Selection: N/A.
- Retry: N/A.
- Count: N/A.
- Persistence: N/A.

**No prior art found** in Inform 7 for generated choices (it predates LLMs and is parser-based by design).

---

## 5. AI Dungeon — GENERATED choices (Classic), then freeform text (v2+)

Primary sources: Wikipedia, `https://en.wikipedia.org/wiki/AI_Dungeon`; Jimmy Maher / IF50, `https://if50.substack.com/p/2019-ai-dungeon`.

**Finding: AI Dungeon Classic (GPT-2, May 2019) generated a numbered option set each turn; the player picked one and it became the next story beat. AI Dungeon 2 (Dec 2019) replaced the generated menu with a freeform text field (Do/Say/Story).** This is the clearest generated-choice prior art in the survey.

**Classic — generated numbered options (the key finding):**
> Options:
> 0) You attack at the moment both small and great piles of corpses and gnomes.
> 1) You use the "hidden tunnel" in order to escape...
> 2) You tell the creature in front of you that you receive the retribution...
> 3) You go through a passage with other ghouls, but discover that there are two other people there...
> Which action do you choose? **3**
>
> You go through a passage with other ghouls... (the chosen option becomes the next narration) (IF50)

The option set was **generated** by GPT-2 as part of the continuation output (not authored), and **picking an option submitted it as the next story input** — the chosen option's text was then narrated forward. "As the user selected from each option list, new text would scroll slowly in to mask the long delay for the server-side code to come up with the next response" (IF50).

**AI Dungeon 2 — generated options replaced by freeform text:**
> He made a few changes to the game's structure and interface, including fine-tuning its training using multiple-choice stories scraped from digital gamebook portal ChooseYourStory.com, and **replacing the generated menu options with a freeform text field**: the user could type their own narration (a "Story" input) or give a command ("Do") like a classic text adventure. (IF50)

**Later input model (current AI Dungeon), per Wikipedia:**
> After beginning an adventure, four main interaction methods can be chosen for the player's text input:
> - **Do**: Must be followed by a verb, allowing the player to perform an action.
> - **Say**: Must be followed by dialogue sentences, allowing players to communicate with other characters.
> - **Story**: Can be followed by sentences describing something that happens to progress the story...
> - **See**: Must be followed by a description... (Wikipedia)

Retry: "Providing blank inputs can be used to prompt the AI to generate further content, and the game also provides players with options to **undo or redo** or modify recent events" (Wikipedia). Undo/redo is regeneration of the last output, not regeneration of an option set.

**Design-question answers (AI Dungeon Classic — the generated-choices variant):**
- Trigger: **always** — every turn presented an option set.
- Generation: **generated** by GPT-2, as part of the continuation output (same call, not a separate agent). The model produced both the narration and the numbered options in one output.
- Selection: **picking an option submitted its text as the next input** and the model narrated it forward. (Selection-as-input.)
- Retry: not documented for the option set itself in Classic; later versions had undo/redo on outputs.
- Count: 4 options (numbered 0–3) in the documented Classic extract.
- Persistence: not documented; options were transient (consumed by selection).

**Note for ticket 04:** AI Dungeon's evolution is itself a design data point — the product *moved away* from generated menus to freeform text, citing improved model coherence. The reasons are worth weighing when designing Chronicler's trigger model.

---

## 6. NovelAI — freeform text + redo, no generated choice set

Primary sources: NovelAI official docs `https://docs.novelai.net/en/text/textadventure`, `.../editor/storysettings`, `.../lorebook`; community knowledge base `https://tapwavezodiac.github.io/novelaiUKB/Starting-your-first-story-in-NovelAI.html`.

**Finding: NovelAI has no in-story generated-choice feature.** Its AI features are text generation (generate/continue/redo), context injection (Memory, Author's Note, Lorebook), and a Text Adventure input mode (Do/Say). "Options" in NovelAI docs refers to AI-model/config options and to "redo those lines" (regenerate), not to a pickable generated choice set.

- Text Adventure mode has two typed input modes (Do / Say) plus wildcards and direction shortcuts:
  > "There's an AI module made specifically to assist the AI at writing in a Text Adventure style... ## The Two Input Modes" (`docs.novelai.net/en/text/textadventure`)
  > "Just inputting `n`, `w`, `s`, `e`... will make your character go to a specific direction" (same).
- Redo (regenerate the last output) is the retry mechanism:
  > "NovelAI then fills in a few more lines. You can either ask it to redo those lines, or add more text yourself." (`zuliewrites.com` review, corroborated by official docs' redo/undo tooling)
- Memory, Author's Note, and Lorebook are **context-injection** mechanisms, not choices:
  > "Author's Note is text that gets injected into the story text three lines/paragraphs up from the current line in the story context every time you hit 'Send'" (`blog.novelai.net`)
  > "The Lorebook is... a repository for supplemental information that's added to the AI's context as each entry comes up in your story." (`docs.novelai.net/en/text/lorebook`)

**Design-question answers:**
- Trigger: N/A (freeform text, no choice set).
- Generation: **none** for choices — text completion only.
- Selection: N/A.
- Retry: redo (regenerate last output) — output-level, not option-set-level.
- Count: N/A.
- Persistence: N/A.

**No prior art found** in NovelAI for generated choices.

---

## Cross-cutting findings

| Design question | ChoiceScript | Twine/SugarCube | Inform 7 | AI Dungeon Classic | AI Dungeon v2+ / NovelAI | ST-CYOA | ST-Roadway |
|---|---|---|---|---|---|---|---|
| Authored or generated? | authored | authored | N/A (parser) | generated | N/A (freeform text) | **generated** | **generated** |
| Trigger model | always (author-placed) | always (author-placed) | N/A | always (every turn) | N/A | on-demand (`/cyoa`) | on-demand **and** auto-on-message (incl. swipe/continue) |
| Generation mechanism | none — authored nodes | none — authored links | N/A | same GPT-2 call as narration (continuation output) | N/A | separate quiet call (`generateQuietPrompt`) | separate request via dedicated (cheap) connection profile |
| Selection flow | jump to authored branch (`*goto`/`*finish`) | navigate to authored passage | N/A | **option text submitted as next input**, narrated forward | N/A | `/impersonate` with chosen text | impersonate OR "use action" (paste + optional auto-send) OR edit |
| Retry of option set | none | none | N/A | not documented (output undo/redo later) | redo = regenerate last output | re-run `/cyoa` (deletes prev. list) | re-run button / auto-refires on swipe (overwrites) |
| Option count | author-defined | author-defined | N/A | 4 (0–3) in documented extract | N/A | 1–10 (user slider) | 6 (hardcoded in default prompt) |
| Persistence | structural (scene file) | structural (passage) | N/A | transient (consumed by selection) | N/A | transient (pseudo-msg, deleted on rerun) | system message w/ `extra.options` (overwritten on rerun) |

**Patterns:**
- **Authored-choice engines (ChoiceScript, Twine) converge on: always-on, author-placed, selection-as-branch-navigation, no retry.** The choice is a structural branching device, not input. This is the dominant IF pattern, but it does not help with *generation*.
- **Three surveyed systems generate pickable choices: AI Dungeon Classic, ST-CYOA, ST-Roadway.** All three are SillyTavern-ecosystem or AI-IF; the authored-choice IF engines (ChoiceScript, Twine, Inform 7) do not generate.
- **Separate-options-call is now attested, not open.** Both ST extensions use a separate generation path distinct from narration (CYOA: `generateQuietPrompt`; Roadway: a dedicated connection profile). Only AI Dungeon Classic folded generation into the narration call — and it later abandoned that. The dominant generated-choices pattern separates the option call from narration.
- **Selection-as-input is universal among generated-choices systems.** AI Dungeon Classic, ST-CYOA, and ST-Roadway all turn a chosen option into the player's next input (via impersonate or paste-and-send). None treats an option as a branch selector. (Inference: for the Chronicler Engine, selection-as-input is the clear precedent; a branch-jump model would be a departure.)
- **On-demand, always-on, and hybrid triggers are all attested.** AI Dungeon Classic = always-on; ST-CYOA = on-demand; ST-Roadway = both (on-demand button + auto-on-message, refiring on swipe/continue). The hybrid is not uncharted — Roadway ships it.
- **Retry = re-run, not swipe, is universal.** No surveyed system regenerates an option set as a swipe gesture; all re-run the generation (overwriting or deleting the previous set). Swipe-style option-set retry remains unattested.
- **A two-model / cheap-profile pattern is attested (Roadway).** Routing option-generation to a cheaper model than narration is a shipped design, not speculation.
- **Domain-diversity prompting for distinct options is attested (Roadway).** Roadway's prompt explicitly enumerates action-domains to force variety — a documented technique for the "N coherent, distinct, non-overlapping options" question.

---

## Gaps (open for design ticket 04 to decide fresh)

These design questions have **no primary-source prior-art answer** in the survey — ticket 04 decides them without inherited precedent:

1. **Option-set retry (swipe-style regeneration of the choices themselves).** No surveyed system does this. All re-run the generation (overwrite/delete). Existing retry is output-level only. → Open.
2. **Persistence of a stable, reusable offered-option set across sessions.** ST-CYOA options are transient; ST-Roadway stores them on a chat message but overwrites on rerun; AI Dungeon Classic was transient. No system documents persisting an option set as a first-class, stable artifact. → Open (partial prior art from Roadway for in-session storage).
3. **Options × steering interaction** (can a player guide/narrator-action alongside options; options for an impersonate). No prior art — none of the surveyed systems combine a steering layer with a choice layer. ST-CYOA and ST-Roadway both *use* impersonate as the selection mechanism, but neither composes options with a separate guide/narrator-action steering layer. → Open (also flagged as fog on the map).

**Questions now CLOSED by the ST-CYOA / ST-Roadway additions** (previously listed as gaps):
- *Separate-options-call vs. same-call* — closed: separate call attested (both ST extensions); same-call attested (AI Dungeon Classic, later dropped). Separate is the dominant pattern.
- *On-demand vs. always vs. both* — closed: all three attested (CYOA=on-demand, AI Dungeon Classic=always, Roadway=both).
- *Prompt structure for N coherent, distinct, non-overlapping options* — closed: two attested shapes — tag-wrapped (`<suggestion>`, CYOA) and numbered-list-with-domain-diversity (Roadway).

---

## Sources

- Parent-map research assets (re-scanned, not re-fetched):
  - `.scratch/steering-and-guided-generation/research/01-marinara-engine.md`
  - `.scratch/steering-and-guided-generation/research/02-guided-generations-extension.md`
  - `.scratch/steering-and-guided-generation/research/03-sillytavern-core.md`
- ChoiceScript: `https://www.choiceofgames.com/make-your-own-games/choicescript-intro/`, `.../important-choicescript-commands-and-techniques/`
- SugarCube (Twine): `https://www.motoslave.net/sugarcube/2/docs/`
- Inform 7: `https://ganelson.github.io/inform-website/book/WI_18_30.html`
- AI Dungeon: `https://en.wikipedia.org/wiki/AI_Dungeon`, `https://if50.substack.com/p/2019-ai-dungeon`
- NovelAI: `https://docs.novelai.net/en/text/textadventure`, `.../lorebook`, `.../editor/storysettings`; `https://blog.novelai.net/kickstarting-your-first-ai-assisted-story-a-beginners-guide-2bd6b98d119b`
- SillyTavern-CYOA: `https://github.com/vitorfdl/Sillytavern-CYOA` (source `index.js`; README)
- SillyTavern-Roadway: `https://github.com/bmen25124/SillyTavern-Roadway` (source `src/index.ts`; README)
