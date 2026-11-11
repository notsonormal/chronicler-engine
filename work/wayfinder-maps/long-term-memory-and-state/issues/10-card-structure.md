# Decide the structure of a Character card and who knows each part

Type: grilling
Status: resolved
Blocked by: —

## Question

Which kinds of text does a Character card hold? How does a prompt label who knows each part? What does each call see? The structure must work for both modes of [Decide how a Character or a lore entry enters the prompt](12-prompt-entry-modes.md): always in the prompt, and loaded by a key.

## Context

- **The owner's position (2026-11-03).** Structure and a label fix a leak, not hiding. The narrator writes every Character and the story, so it must see the internals of a Character. A label says that only that Character knows them, and the internals still drive her behaviour. The other Characters do not know a hidden motive, but the story must know it. A Character memory follows the same rule: the model sees it, and only the Character knows it. So one owner label covers card internals, Character memories ([Choose the memory candidates to try](05-memory-candidates.md)) and card changes ([Decide what a card change of one Game holds and who writes it](07-card-change.md)).
- **The card in the prompt today (known, `long-term-memory` at `20d6d23c`).**
  - `<KnownCharacters>`, before the history: for every Character, the name and `summary`, or the first three lines of `description` (`render_known_character_entry`, `src/application/prompting/builders/sections.rs`).
  - `<CharactersPresent>`, after the history: for each Character present, `description`, `personality`, `scenario`, and the relationship lines to the other Characters present only (`render_present_relationships`).
  - `example_dialogue` reaches no prompt.
  - The World's `global_rules` holds a second cast list, with one line for each Character, in every call.
  - The Impersonate call drops only the Persona layer (`src/application/prompting/assembler.rs`). So it gets the full card of every Character present, with her internals.
  - The Quantifier and Options prompts read the `description` of the Characters.
  - A Narration reads the Characters present of the Swipe state before it (`src/application/pipeline/job.rs`). So the Narration in which a Character enters has only her short entry. The stored call `tmp/lt15/new/trial-1/calls/01060-event-narration.md` shows it: the "Gabriella Introduction" event Narration had only Carla in `<CharactersPresent>`.
- **The text is mixed today (known).** Gabriella's `summary` ends "plans to seduce Julian to keep her luxurious lifestyle", in every call. Her `scenario` mixes public facts (Bernard's widow, left out of the will) with internals (she cut contact with her family; "Goals: … she plans to seduce {{user}}").
- **A sketch of the kinds (coordinator, 2026-11-03, not decided).**

  | Kind | Who knows it | Example | Today |
  |---|---|---|---|
  | Public facts | Anyone in the story may know | Full name, role, widow of Bernard, left out of the will | Mixed into `summary`, `description` and `scenario` |
  | Internals | Only the Character. They drive her behaviour | Gabriella's plan, her cut-off family | Mixed into `summary` and `scenario` |
  | Writing guide | Not a fact. It tells the narrator how to write her | Personality, voice, example dialogue | `personality`. `example_dialogue` is not used |

  Open: a fourth kind, story facts that no Character knows yet (a World secret). It probably belongs to the World or to a lore entry ([12](12-prompt-entry-modes.md)), not to a card.
- **The faults that this ticket targets.**
  - Card leaks. 3 of the 8 checked who-knows-what faults of [Count the memory faults of the newest trial](03-baseline-faults.md) (A-3.3, A-3.4, B-3.3). Two of the leaked facts, Louise's surname and Lisette's decade of service, are in the short entries since `dfc11a70` (2026-11-02). So every call holds them.
  - The short entry of an absent Character (section 4 of [the inconsistencies of two trial stories](../../layout-trials/assets/story-inconsistencies.md)). 2 in the pilot story Q, before the data fix. 0 and 1 (low confidence) in the lt15 stories X and Y (`tmp/lt15/judge/`). Judge X: the absent Characters "are almost never mentioned, so there was little chance for this fault".
  - The entry Narration with only the short entry (the mechanism above). Count unknown.
- **The evidence on labels (known, [LLM memory research](../assets/llm-memory-research.md), section 4).** The Curse of Knowledge preprint: a prompt that tells the model to keep to what a Character can perceive cut leaks by 64.7 % on average, and it also dropped content that the Character was allowed to use. A view built outside the model cut leaks to 0. A change of the input layout alone changed the leak rate a lot. The scenes were short templates. So the label needs the measure of [Make the card probe](11-card-probe.md).
- **Hiding still fits one call (coordinator, 2026-11-03).** The Impersonate call plays only Julian. For it, a view built outside the model is simple: the public facts of the other Characters and Julian's own internals.
- **The Narration is also a channel.** The player reads each Narration, and the Impersonate call reads it in the history. A Narration that states an internal ("she planned to seduce him") gives it to Julian from then on. Guessed: the rule may depend on `narrative_perspective`.
- **The size (inferred, characters ÷ 4).** The five Redmist cards (`description`, `personality`, `scenario`, relationship lines) are about 2,600 tokens. The five summaries are about 300. A fixed block before the history is in the cached prefix (86 % cached in the lt15 trial). Today the cards of the Characters present sit after the history, where nothing is cached.
- **The working layout for the probe.** Every card always on, in one block before the history, in place of `<KnownCharacters>` (design A). [12](12-prompt-entry-modes.md) decides the final place of each kind of text in both modes.
- **Authoring (owner, 2026-11-03).** The build ticket moves existing text into the new fields. It may add keys and remove names from always-on text. It writes no new card prose.

## Output

A resolution with:

- the kinds of card text, and the new field of each part of today's fields, checked against the five Redmist cards and `julian.json`;
- the form of the owner label in the prompt. It also fits text that changes in play: Character memories and card changes;
- the rule for the Narration on internals;
- what each call sees: the narrator, the Impersonate call, the Quantifier and the Options call;
- whether the Persona card gets internals;
- whether a fourth kind (story facts that no Character knows) exists, and where it lives;
- the forms of the label that [Make the card probe](11-card-probe.md) compares.

## Grilling notes (in progress, not the answer)

Round 1 (owner, 2026-11-04):

- Known: the leak A-3.3 came from the Impersonate call `tmp/lt15/new/trial-1/calls/01210-impersonate.md`, which had Lisette's full card and summary.
- Levels: three levels and a writing guide. *Seen* (what anyone sees on meeting her), *known to the cast* (true facts that the other Characters know, the Persona learns them in play), *only her* (internals). Owner: "known to the cast" is coarse; the model must infer from the context and the story who knows it.
- Relationship lines: *only her*. They are her view and drive her behaviour. The model can infer how others see her from that behaviour.
- Narration: owner correction. The narrator knows an internal whether or not a Narration writes it, and it decides whether Julian knows it. A written internal does not by itself give it to Julian.
- Fourth kind: a World secret is a World rule (a clause) that carries in itself who knows it and who does not.
- Persona card: the same levels as a Character card. Julian's text moves to *seen* and *known to the cast*, and to *only him* where the World does not state it. No new prose.

Round 2 (owner, 2026-11-04):

- Who learned a fact in play: the engine records nothing in this ticket. The narrator works it out from the story. The memory tickets may add a record.
- Label form: accepted a `known_by` value (`everyone`, `seen`, `cast`, a list of ids), set on a World rule and a lore entry; a card uses the levels of its fields. Reopened in round 3 after `/consider-the-bitter-lesson`.
- What each call sees (owner): every call sees everything, with labels, except the Quantifier. Reason: a call that adds and removes parts is hard to understand, and the cache. Trade-off: the label alone must hold on the Impersonate call, where A-3.3 came from.
- Narration: no rule. The probe counts how often a Narration states an internal.
- Owner question: what `example_dialogue` "stays out of the prompt" means, and what "writing guide" means. Known: no prompt builder reads `example_dialogue` (only the domain and storage models hold it). SillyTavern at `06bde939` has three modes: "Gradual push-out" (the default: examples go in after the history, only if they fit), "Always include" and "Never include" (`public/index.html:5363`, `public/scripts/openai.js:1336-1343`).
- Draft mapping of the six cards: [card structure mapping](../assets/card-structure-mapping.md).

Round 3 (owner, 2026-11-04, after `/consider-the-bitter-lesson`):

- The minimal form is the default build: the cards keep `description`, `personality`, `scenario` and `example_dialogue` as written; one new field `secrets` holds only what she hides; `summary` is removed; one rule text explains how to read the cards. The full split of [the mapping](../assets/card-structure-mapping.md) is one probe arm. The build uses it only if it has clearly fewer leaks (more than about 10 points).
- Who knows a World rule: words in the text of the rule. No `known_by` field (this replaces round 2). Memories and card changes only record whose they are.
- `example_dialogue`: always in each card, with a header that it is an example of her voice, not an event of the story. Risk: repeated first meetings (guessed); the check-trial judge reads for it.
- Probe arms: today; minimal without the rule text; minimal with the rule text; full split with labels. 3 runs each, on Narration and Impersonate calls.
- Quantifier: keeps `description` of each Character, as today.

Round 4 (owner, 2026-11-04):

- `secrets` holds the Gabriella and Jezebel sentences of the table in the Answer. Carla's drinking stays in `scenario`.
- Carla's instruction line "Carla will never speak for {{user}} under any circumstances." is dropped (Option B).
- Rule text: at the head of the card block. Owner: the secret sentence says what the others know, not what she tells.

## Answer

Resolved with the owner, 2026-11-04, in four rounds. The bitter lesson (round 3) changed the design from a full split of each card to a minimal form. The probe decides whether this model needs more.

**The card (the minimal form).** The Character card and the Persona card have the same structure.

- `description`, `personality`, `scenario` and `example_dialogue` stay as the author wrote them. This keeps SillyTavern import: our cards come from SillyTavern cards (`scripts/extract_sillytavern_png.py`, `scripts/refine_character_json.py`).
- One new field, `secrets`, holds only what the Character hides. A fact that is private but not hidden stays where it is.
- `summary` is removed. It is a copy of the other fields, and it put Gabriella's plan in every call. [Decide how a Character or a lore entry enters the prompt](12-prompt-entry-modes.md) decides whether a cast line exists and which existing text it uses.
- `relationships[].static` stays. It is her view of another Character and drives her behaviour. The model can infer from that behaviour how the others see her.

**What moves.** The build moves these sentences from `scenario` to `secrets` and writes no new prose:

| Card | `secrets` |
|---|---|
| Gabriella | "Instead, Gabriella moved out and cut contact with her family the moment she turned 18." "Gabriella moved to the big city … lavish houses to live in." "Gabriella quickly took to seducing Bernard … richest younger women in the city." Goals: "Gabriella only follows money. … luxurious lifestyle." |
| Jezebel | "and had many conversations about his past regrets and worries about his will" (the second half of one sentence) |
| Carla, Lisette, Louise, Julian | Nothing |

The build also drops Carla's line "Carla will never speak for {{user}} under any circumstances." The system preset already gives that rule.

**The label in the prompt.** No `known_by` field and no levels in the data. A card block holds each card with a header that names its Character. `secrets` has its own header in the card. `example_dialogue` is always in the card, with a header that says it is an example of her voice, not an event of the story. One rule text heads the card block:

> A card is the author's view of one Character. She knows her own card. Another Character knows only what the World rules state, what she saw, and what she learned in the story. Other Characters do not know her secrets until they learn them in the story. {{user}} starts with the World rules and his own card only.

The last sentence fixes the class of the confirmed card leaks (A-3.3, A-3.4, B-3.3): Julian used Lisette's years of service and Louise's surname from the cards. Known: A-3.3 came from the Impersonate call `tmp/lt15/new/trial-1/calls/01210-impersonate.md`, which had Lisette's full card. The Impersonate preset already says "Do not act on information {{user}} could not have". So a rule in the preset alone did not hold on this model. The probe tests the rule text in the card block.

**Text that changes in play.** A Character memory ([Choose the memory candidates to try](05-memory-candidates.md)) and a card change ([Decide what a card change of one Game holds and who writes it](07-card-change.md)) record whose they are, the same as a card. The rule "she knows her own card" covers them. A card change can change `secrets` like any other field.

**Who learned a fact in play.** The engine records nothing for this ticket. The narrator works it out from the story. The memory tickets may add a record.

**The Narration.** No rule. The narrator knows an internal whether or not a Narration writes it, and it decides whether Julian knows it (owner). [Make the card probe](11-card-probe.md) counts how often a Narration states a secret outright. A rule comes only if that count is high, or if it goes with leaks in Julian's Input.

**What each call sees.** The Narration, the event Narration, the Impersonate call and the Options call see the same card block, with everything in it. A call that adds and removes parts is hard to understand, and it breaks the cache (owner). The Quantifier keeps what it reads today, the `description` of each Character, which holds no secret. Trade-off: the label alone must hold on the Impersonate call. If the probe shows that it does not, the build or [12](12-prompt-entry-modes.md) must hide more.

**The Persona card.** The same structure. Julian has no `secrets`. The rule text keeps his past from the others: they know only what the World rules state.

**A fourth kind (story facts that no Character knows).** It is a World rule, or a lore entry of [12](12-prompt-entry-modes.md). It says in its own words who knows it, for example "Only Jezebel and the lawyer know of the second clause." All of today's `global_rules` are facts that everyone knows.

**The probe arms** ([Make the card probe](11-card-probe.md)). Each arm runs 3 times, on Narration calls and on Impersonate calls:

1. Today's prompt.
2. The minimal form without the rule text.
3. The minimal form with the rule text.
4. The full split, with a label on each part ([the card structure mapping](../assets/card-structure-mapping.md)).

The build uses the minimal form with the rule text. It uses the full split only if arm 4 has clearly fewer leaks than arm 3, by more than the noise of about 10 points.

**Not decided here.** The place of the card block, the mode of each Character, and the cast line belong to [12](12-prompt-entry-modes.md). The working layout of the probe is every card always on, in one block before the history, in place of `<KnownCharacters>`.
