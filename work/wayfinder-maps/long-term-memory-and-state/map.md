# Map: Long-term memory and state

Labels: wayfinder:map
Status: open

## Destination

Long play keeps the facts that left the history, the time that passed, and what each Character knows. The size and the point of the history cut are decided first, from research. Research and trials choose the memory design for this engine, and the design is built test first. A Character card can change inside one Game. A Character card separates what anyone may know from what only the Character knows. An author chooses, for each Character and each lore entry, whether it is always in the prompt or loads when a key appears. A person can view and edit the memory, the state and the card changes in the dashboard. A 100-Cycle trial shows that the judge finds fewer of the three memory faults than in the reference trial.

## Notes

- **Origin.** The [seed](idea.md). [Prompt flexibility](../prompt-flexibility/map.md) kept room for this effort, and [Layout trials](../layout-trials/map.md) found the faults that this effort must fix. The judge pilot of Layout trials ([the inconsistencies of two trial stories](../layout-trials/assets/story-inconsistencies.md)) gave three kinds of fault to this effort. In this map, **the three memory faults** are these kinds:
  - a fact that had left the history (section 1 of that asset);
  - no story clock (section 5);
  - who knows what (section 6).
- **The mechanism is open (owner, charting).** "Long-term memory" is a goal. The owner does not yet know what it means for this engine or how to get there, and it is not a solved problem. The order of the work:
  1. Research: Marinara Engine in depth, and the general field.
  2. The size and the point of the history cut (owner, 2026-11-03).
  3. A memory test of candidates on stored stories.
  4. Trials.
  5. The design for this engine.
- **The working view of memory (owner, charting).** The research and the trials test this view. They do not take it as fixed.
  - The story memory and the Character memories are different in what they are and in their structure.
  - The story memory changes with everything that happens, whoever is present. A memory for the Persona is most likely the same as the story memory.
  - Owner: "when characters leave a story they will keep to have what happened to them updated. However the story will need to be updated regardless of who is in the scene." Whether a Character memory changes while the Character is absent is open. [Choose the memory candidates to try](issues/05-memory-candidates.md) asks it.
- **Execution is in this map (owner, charting).** Decide, build test first, and check each change with a trial, as in Prompt flexibility and Layout trials. A decision ticket only decides. A build ticket makes the change test first (`/tdd`) and ends with `/simple-after-plan`: the comment pass, the doc review, `/code-review`, the full `python build.py`, and the retro. Each build ticket changes the docs that its change makes wrong (`docs/AGENTS.md`, "Direction of edits").
- **Scope (owner, charting).** The three memory faults. Other kinds of state become tickets only when a trial shows a fault that they cause (see Out of scope).
- **Cards in the prompt (owner, 2026-11-03).** Also in scope:
  - the structure of a Character card, and a label that says who knows each part. A hidden motive stays visible to the model, labelled as known only to its Character. A Character memory uses the same label. Decided as a minimal form: one `secrets` field and one rule text ([Decide the structure of a Character card and who knows each part](issues/10-card-structure.md));
  - the short entry of an absent Character (section 4 of [the inconsistencies](../layout-trials/assets/story-inconsistencies.md));
  - a mode for each Character and each lore entry: always in the prompt, or loaded by a key. It changes how a Character is designed, so it is decided with the card structure.

  The tickets are [Decide the structure of a Character card and who knows each part](issues/10-card-structure.md), [Make the card probe](issues/11-card-probe.md) and [Decide how a Character or a lore entry enters the prompt](issues/12-prompt-entry-modes.md). Their Context holds the facts found on 2026-11-03.
- **Card changes (owner, charting).** In scope.
  - A card change applies to one Game. It is saved against the Character of that Game in storage, for example in a table for each Game and Character. The World card stays as the author wrote it.
  - A writer changes the card of the Game directly. No person approves the change. A person can view, edit and revert it in the dashboard.
  - Each card change records the Message and the Swipe that made it. A Retry, a swipe switch or a delete that removes that Swipe from the story also removes the change. The list of changes is the version history of the card in that Game.
- **UI (owner, charting).** A person can view and edit the memory, the state and the card changes in the dashboard. Unlike the two earlier maps, this map adds new UI. Use `/impeccable`, and read `docs/diataxis/reference/frontend/dashboard.md` and `docs/diataxis/explanation/dashboard_design.md`.
- **The size of a trial (owner, 2026-11-03).** The default history size (65,536) and the size of a trial are separate. Every trial of this map, and the check trial, runs at 16,000 tokens ([Decide the size and the point of the history cut](issues/08-history-cut.md)).
- **The outside check (owner, charting).** One 100-Cycle trial of `scripts/trial_setups/redmist_julian.json` on the built change, against a reference trial on the code before the change. A judge counts the three memory faults. The check passes when those counts go down and no other kind of fault goes up. The setup and the model are those of Layout trials (`deepseek-v4.1-flash` on OpenCode Go). The judge method is that of [Decide how to read an output trial](../layout-trials/issues/05-read-an-output-trial.md).
- **The counts are small (known).** After the check of [Count the memory faults of the newest trial](issues/03-baseline-faults.md), the newest story has 0, 4 and 5 of the three memory faults, and the reference story has 0, 1 and 3. The raw judge counts were more than twice as high, so each trial needs a check step. With one trial for each side, a change of this size is inside the noise (inferred). So the memory test chooses between candidates, and the trial confirms.
- **The memory test comes first (owner, charting).** A memory candidate first runs the memory test on stored stories. The test asks questions whose answers already left the history. It is the memory version of [`prefix_measure.py`](../layout-trials/assets/prefix_measure.py) of Layout trials. It needs no trial, but it is not cheap and not stable (known, [the memory test](assets/memory-test.md)). One run of 42 questions costs about 640,000 to 1,200,000 prompt tokens for each arm, with 60 to 90 % cached. At temperature 0 the answers still change from run to run, so a candidate needs 3 runs, and a gap under about 10 points is noise. Only a candidate that passes gets a full trial.
- **Cost (owner, charting).** No limit now. Each trial reports the extra calls and tokens of the new writers. The design tickets set a cap on the memory text in each call. The limits come later.
- **Branch (owner, charting).** The build tickets work on the branch `long-term-memory`, made from `main` after `layout-trials` merges. The research, the memory test and the grilling tickets need no branch. A trial runs in a linked worktree, as in Layout trials.
- **The room that Prompt flexibility kept** ([Decide whether this map designs memory and state, or only keeps room for them](../prompt-flexibility/issues/16-memory-and-state-scope.md)):
  - The layout has an open list of block types and no fixed place for a new block.
  - `HistoryCut` (`src/domain/model/history_cut.rs`) is the one history cut. It drops whole Messages from the front, in blocks, and records which Messages each block dropped. It fires no hook yet.
  - A writer is a new Agent on `AfterNarrationAgent` or a new hook trait. A failed or missing writer keeps the previous state ([Keep the state when a state writer fails](../prompt-flexibility/issues/23-keep-state-when-a-writer-fails.md)). "State writer" is not a glossary term yet.
  - Memory and state that depend on the story are pinned to the Swipe. New `SwipeState` fields are optional JSON keys.
- **The engine today (known, `layout-trials` at `1989d3fe`).**
  - Each history entry of a Narrator-role call is one chat message with a role and no label (the ME format).
  - The user prompt runs KnownCharacters, Persona, WorldLore, the history, GameState, CharactersPresent, then the splice.
  - Each Game and agent has its own OpenCode session.
  - In the newest 100-Cycle trial, five history cuts dropped 114 of 201 entries. After the first cut the history held 62 to 89 entries.
  - Retry and delete act only on the last Message. An edit can change the active Swipe text of any Message (`GameWriter::edit`).
  - Two separate limits (owner, 2026-11-03). The dashboard shows the newest 50 entries (`STORY_WINDOW_SIZE`), and storage keeps every Message. The prompt history is at most the history size of the Connection (`max_history_tokens`; empty uses `DEFAULT_MAX_HISTORY_TOKENS`, 65,536), since [Make the history size a Connection field](issues/09-history-size-field.md). The probe DB OpenCode Go Connection sets no `max_context_tokens`, so its window is 32,768; `data/settings.json` gives 1,000,000 to a fresh DB only.
- **How a trial runs.** As in Layout trials: read its Notes "How a trial runs", "Worktree limits" and "Failure modes" ([Layout trials](../layout-trials/map.md)). An agent may not read `.env` files, and never reads or prints a key.
- **Standing rules (from the earlier maps).** Write plans directly in the repo, not through Plannotator. Any new API takes a `GameId` and adds no read of `ActiveGame`. A migration takes the next free step, adds a column with `ALTER TABLE … ADD COLUMN` guarded by `column_exists` or adds a new table, never edits an old `CREATE TABLE`, raises `LATEST_VERSION`, and gets its test in `plumbing_tests.rs`. A new unit test goes only where `tests/STRATEGY.md` ("The one rule") allows it.
- **Names.** "Story memory", "Character memory", "card change", "memory test", "memory candidate", "the three memory faults", "internals", "owner label", "secrets", "the rule text", "lore entry", "keyed" and "card probe" are working names. A term enters `CONTEXT.md` when its first code lands (`CONTEXT.md`, Notes). Use `/domain-modeling` then.
- **ST and ME.** They are references: read their source, and do not run them. Pinned commits: SillyTavern `06bde939fb1e9c4c8d8641d810f0a916b5bce127`, Marinara Engine `457e827a3d3291be978121f8c50c98c5dd9d644d`. ME has no prompts for its downloadable Agents: they are in the Marinara-Agents repo, read at `f9a05f76` ([Find how Marinara Engine keeps memory with its agents](issues/01-marinara-memory.md)). Clones were at `/tmp/research/st` and `/tmp/research/marinara` on 2026-11-02. `/tmp` can be cleared, so clone again outside the repo when they are gone.
- **Skills.** `/research` for a research ticket, `/grilling` and `/domain-modeling` for a grilling ticket, `/prototype` for a prototype ticket, `/chronicler-ui-investigator` for a probe server, `/impeccable` for the dashboard, and `/simple-after-plan` for a build ticket. Read `docs/diataxis/reference/narrative/prompt_system.md`, `docs/diataxis/explanation/prompt_system_design.md` and `docs/diataxis/reference/narrative/agent_system.md` first.
- **Sources. Start from these; do not redo them.**
  - From Prompt flexibility: [how other tools keep long-term memory](../prompt-flexibility/assets/long-term-memory.md), [how other tools keep long-term state](../prompt-flexibility/assets/long-term-state.md), and [prompt caching](../prompt-flexibility/assets/prompt-caching.md).
  - From Layout trials: [the inconsistencies of two trial stories](../layout-trials/assets/story-inconsistencies.md), [the judge prompt of the pilot](../layout-trials/assets/judge-prompt-pilot.md), and the judge prompt and verdicts of ticket 15 (`tmp/lt15/judge/`).

## Decisions so far

<!-- the index — one line per closed ticket: enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [Find how Marinara Engine keeps memory with its agents](issues/01-marinara-memory.md) — eight separate memory writers, each off by default; Advanced Memory writes one recap for each closed scene with an audience and a recollection for each Character present, and writes nothing for an absent Character; the leak check is a four-word copy test; story time is free text with no arithmetic; a card change needs approval and changes the card in every chat; the evidence is plumbing tests only ([asset](assets/marinara-memory.md)).
- [Find what is known about long-term memory for LLMs, and what is still open](issues/02-llm-memory-research.md) — not solved: while the history fits, full context beats every memory system on accuracy, and memory wins on cost; a changed fact and time are the weakest points; a filtered view for each reader is the one family built for who knows what (one short preprint); no clock method and no benchmark for a story with several Characters exist ([asset](assets/llm-memory-research.md)).
- [Count the memory faults of the newest trial](issues/03-baseline-faults.md) — after the check, story A has 0, 4 and 5 of the three memory faults, and story B has 0, 1 and 3: no confirmed fact that had left the history, because the story drops old threads and does not contradict them; the likely fixes are a clock, a record of who was present, and a rule that keeps card facts from a Character who has not learned them; 40 facts from the dropped entries for the memory test ([asset](assets/baseline-faults.md)).
- [Make the memory test from the stored stories](issues/04-memory-test.md) — the tool asks 42 questions in the stored Narration request at the cut, and a candidate changes only a `<StoryMemory>` block; on 35 kept questions the no-memory control scores 0 %, the whole story 95 %, and one 600-word summary of the dropped entries 67 % (57 to 77 % over 3 runs); the summary loses who did small things, reasons and who learned what; temperature 0 is not stable, so a gap under about 10 points is noise; the set does not test the "does not know" side ([asset](assets/memory-test.md)).
- [Decide the size and the point of the history cut](issues/08-history-cut.md) — the cut stays a size in tokens with a 75 % block; the size becomes a Connection field (`max_history_tokens`) with a default of 65,536, so a 100-Cycle story fits whole; OpenCode Go gets a 1M window; the trials of this map, and the check trial, run 100 Cycles at 16,000 tokens through a `history_tokens` field of the setup file ([model context asset](assets/model-context.md)).
- [Make the history size a Connection field](issues/09-history-size-field.md) — built on `long-term-memory`: `max_history_tokens` on the Connection, default 65,536, a form field, and `history_tokens` in the trial setup as the only place that sets a trial size; the check trial at 16K passes (6 cuts from Cycle 41, 85.6 % cached) and is the new reference trial (`tmp/ltm09/redmist_julian/trial-1`).
- [Decide the structure of a Character card and who knows each part](issues/10-card-structure.md) — after the bitter lesson, a minimal form: the cards keep their SillyTavern fields, one new `secrets` field holds what a Character hides (Gabriella's plan and past, Jezebel's talks with Bernard), `summary` goes, `example_dialogue` comes in, and one rule text heads the card block ("{{user}} starts with the World rules and his own card only"); who knows a World rule is in its words, with no field; every call but the Quantifier sees the same block; no Narration rule; the probe compares four arms, and the full split ([mapping](assets/card-structure-mapping.md)) is built only if it clearly wins.
- [Make the card probe](issues/11-card-probe.md) — the label does not hold on this model: 18 scenes of story A, 4 arms, 3 runs; the rule text changed nothing (Julian says Lisette's "near a decade" in all 12 runs of one scene), and the full split leaks more (6 against 3), so it is not built; full cards always on remove the entry faults (3 of 30 to 0 or 1) but Carla tells an absent Gabriella's `secrets` in 8 of 9 runs; no stated secret and no repeated meeting; the cards add about 2,600 tokens to each call; [12](issues/12-prompt-entry-modes.md) must decide which text stays out of which call ([asset](assets/card-probe.md)).

## Not yet specified

- **The trials of the candidates.** Which candidates get a full trial after the memory test, and in what order.
- **The design for this engine.** The memory, the state and the card change for this game, chosen from the research and the trials. With it: the seam of each writer (a new Agent on the after-narration Hook, a new Hook, or a step of the history cut).
- **The view and the edit in the dashboard.** For the memory, the state and the card changes. A prototype, when the design is known.
- **The limits.** The cap on the memory text in each call, and the cost of each writer.
- **An edit of an older Message.** An edit can change the text of any Message. What happens to a memory or a card change that the old text made.
- **The other calls.** Whether the Impersonate, Quantifier and Options calls read memory, and which part.
- **The reading pack.** How a trial pack shows the memory, the state and the card changes of each Cycle, so that a reader can see a change, a stuck field or a lost fact.
- **The build of the card structure and the keyed entries.** It waits for [Decide how a Character or a lore entry enters the prompt](issues/12-prompt-entry-modes.md), which now also decides which card text stays out of which call ([Make the card probe](issues/11-card-probe.md)).
- **More of the memory test.** A question set for a second story, and questions for the "does not know" side of who knows what (several forms of one fact, as in FANToM, or twin scenes). The [memory test](assets/memory-test.md) has neither (section "Limits").
- **The glossary entries.** The names of the new parts, when their first code lands.
- **The build tickets and the check trial.**

## Out of scope

- **The scroll-back of the story in the dashboard** (owner, 2026-11-03). The dashboard shows only the newest 50 entries. Loading older entries is a UI fix that depends on no memory decision, so it is a separate effort ([the seed](../../issues/pending/19-story-scroll-back.md)).
- **A card change for the World** (owner, charting). A card change applies to one Game. Moving a change of one Game to the World card is a later effort.
- **A Persona memory that a Persona carries into its next Game** (owner, charting). A later effort.
- **Watch: other kinds of state** (owner, charting). Mood, clothes, wounds, items, money, quests, needs, and the unused card fields `relationships[].dynamic` and `inventory` ([how other tools keep long-term state](../prompt-flexibility/assets/long-term-state.md), section 1). Each one becomes a ticket only when a trial shows a fault that it causes, or when [Decide what a card change of one Game holds and who writes it](issues/07-card-change.md) includes it.
- **The other fault kinds of the judge pilot** (owner, charting: the scope is the three memory faults).
  - A fact that the model contradicted while it was in the prompt (section 2): the attention of the model. Layout trials accepts it ("The model is good enough").
  - Our data (section 3). Section 4, the short entry of an absent Character, moved into scope on 2026-11-03 ([Decide the structure of a Character card and who knows each part](issues/10-card-structure.md)).
  - The wrong Room or the wrong Characters present in a State line. Known size: 14 to 20 runs in one story (`tmp/lt15/judge/`). It is a fault of the Quantifier, not of memory.
- The authoring of the World, the cards and the presets. Exception (owner, 2026-11-03): a build ticket may move existing card and World text into new fields, add keys, and remove names from always-on text. It writes no new card prose.
- **Authoring cards and lore entries in the dashboard** (owner, 2026-11-03). Authors edit the data files, as today.
