# Decide how a Character or a lore entry enters the prompt

Type: grilling
Status: open
Blocked by: 10

## Question

For each Character and each lore entry, an author chooses whether it is always in the prompt or loads when a key appears. What are the modes, the keys and the scan rules? What is a lore entry? Where does each kind of text go in the prompt? Which mode does each Redmist Character use in the check trial?

## Context

- **The owner's position (2026-11-03).** Keyed Characters and lore entries are in this map, because they change how a Character must be designed. A Character named in always-on text loads anyway. So to keep a Character out of the story, the author must not name her there. A Character that nobody mentions may never appear, and that can be better than a Character referenced with little knowledge.
- **The modes (coordinator sketch, not decided).**

  | Mode | What the narrator sees | Fits | Risk |
  |---|---|---|---|
  | Always on | The full card, in every call | The main cast, a small World. All five Redmist Characters (design A) | Tokens grow with the cast |
  | Keyed | Nothing until a key matches ("Louise", "cook", "kitchen"), then the full card | Minor Characters, a large World | A substitute Character. A match in passing |
  | Keyed, with a cast line | Name and role only, then the full card when a key matches | A middle ground | The narrator invents her look at her first entry |

- **What the scan reads (known, SillyTavern at `06bde939`).** `world_info_recursive` is `false` by default (`public/scripts/world-info.js:75`). Each entry must opt in to scan the persona description, the character description or the scenario (`matchPersonaDescription` and the others, `:121-125`). With recursion, the text of a loaded entry can load other entries. Inferred: under a scan of the story only, a name in always-on text gives a thin reference. With recursion, it makes the entry always on.
- **Louise is named in three always-on places (known).** The World's `global_rules`, and the relationship lines of Carla and Lisette. `global_rules` holds a cast line for all five Characters. Today a relationship line shows only when both Characters are present (`render_present_relationships`).
- **Substitutes (known counts, guessed cause).** The lt15 stories already invent staff: "driver" 14 times in X and 7 in Y, a gardener, a gatekeeper and "the maid" (`tmp/lt15/judge/story-*.md`). A narrator that does not know a keyed Character may invent someone in her role.
- **Timing (known).** A key in the Input works at once. A key in the narrator's own reply, or a Room change, works only on the next call. The Quantifier sets the Room and the Characters present after the Narration (`src/domain/model/utils/game_rules.rs`, `src/application/pipeline/job.rs`). A Trigger names its Character (`EventRecord.trigger.character_id`), so a keyed design can load her card for the event Narration. In the stored call `tmp/lt15/new/trial-1/calls/01060-event-narration.md`, the "Gabriella Introduction" event Narration had only her short entry.
- **ME (known, Marinara Engine at `457e827a`, Marinara-Agents at `f9a05f76`).** Knowledge Retrieval and Knowledge Router are Agents that run before generation. They choose lorebook entries for "present or mentioned characters". No ME Agent checks a reply against a card. The Continuity Checker fixes place, time and physical faults only. Inferred (`dry-run-route.ts`): ME puts the full card of each Character of a chat in the prompt.
- **Progressive disclosure (known, Anthropic Agent Skills docs).** A short description is always in the prompt, the full text loads when it is relevant, and linked files load when needed. There the model chooses what to load. Here a rule chooses: keys, a mode, a Room or a Trigger. I do not know whether `deepseek-v4.1-flash` on OpenCode Go handles tool calls well.
- **The cache (inferred, [prompt caching](../../prompt-flexibility/assets/prompt-caching.md)).** Always-on text can sit before the history. Text that loads and unloads from Cycle to Cycle breaks the cache before the history, so it goes after the history, unless it stays loaded.
- **The World text today (known).** `<WorldLore>` holds the World name and description. `global_rules` goes into the system prompt. All of `world.json` is about 600 tokens (inferred, characters ÷ 4). No lore entry exists.
- **The link to memory.** A lore entry that a writer creates is one shape of story memory: the ME Lorebook Keeper pins each entry to a Message and a Swipe, and ST MemoryBooks saves scene summaries as lorebook entries ([LLM memory research](../assets/llm-memory-research.md)). See [Choose the memory candidates to try](05-memory-candidates.md).
- **The card structure (decided, [10](10-card-structure.md), Answer).** The minimal form: the card keeps its SillyTavern fields, one new field `secrets` holds what she hides, `summary` is removed, and `example_dialogue` is always in the card. One rule text heads the card block. Every call except the Quantifier sees the same card block. No card field gives a role, so a cast line or a key must use existing text, for example the first line of `description`. A lore entry says in its own words who knows it ("Only Jezebel knows …"). No `known_by` field exists.
- **The probe result (known, [Make the card probe](11-card-probe.md), Answer; [card probe](../assets/card-probe.md)).** The rule text and the labels did not hold on this model. With every card always on, the entry faults went from 3 of 30 Narration answers to 0 or 1. But when Julian asked Carla about an absent Gabriella, Carla told Gabriella's `secrets` in 8 of 9 runs. Julian used Lisette's decade of service, a plain `scenario` fact, in all 12 runs of one directed Impersonate scene (3 runs in each arm). So this ticket must also decide which text stays out of which call. Examples: the `secrets` of a Character who is not present, and the other Characters' cards in the Impersonate call. The probe tool can test a candidate layout with no build (`python3 assets/card_probe.py`, a new arm).
- **Authoring (owner, 2026-11-03).** The build ticket moves existing text into the new fields. It may add keys and remove names from always-on text. It writes no new card prose.

## Output

A resolution with:

- the modes, and the card fields that each mode needs (mode, keys, cast line);
- the keys: name, role and Room, and how a match is made (word boundary, case);
- what the scan reads, and whether a loaded entry can load another;
- the always-on mention rule, and its check in `scripts/validate_data.py`;
- the relationship lines that name a keyed Character;
- the first-mention lag: accepted or fixed, and how. The Trigger case;
- a lore entry: its fields, its owner label and its keys;
- the place of each kind of text in the prompt, and the cache cost;
- a cap on the loaded entries;
- the mode of each Redmist Character in the check trial, and how the judge counts a substitute Character;
- whether the keyed mode needs its own probe. If it does, create that ticket.
