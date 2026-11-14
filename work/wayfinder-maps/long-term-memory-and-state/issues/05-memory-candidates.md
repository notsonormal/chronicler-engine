# Choose the memory candidates to try

Type: grilling
Status: resolved
Blocked by: 01, 02, 08

## Question

Which memory designs does this map try in the memory test? For each candidate: what it stores, which call writes it and when, where the memory goes in each call, and what it costs.

## Context

- **The history cut comes first (owner, 2026-11-03).** The size of the history space was never decided. It decides how much leaves the prompt, and so what a memory must hold. This ticket waits for [Decide the size and the point of the history cut](08-history-cut.md).
- **What leaves the history (known, from tickets 08 and 09).** The history size is the Connection field `max_history_tokens`. An empty field uses `DEFAULT_MAX_HISTORY_TOKENS` (65,536, `src/domain/model/settings.rs`). With the default, nothing leaves the history before about Cycle 155 (inferred). The cut stays a size rule with a 75 % block. A scene end as the cut point is open to this ticket.
- **The trials run at 16,000 tokens (known, ticket 09).** The trials of this map run at 16,000 tokens, as the stored stories did. In a 100-Cycle trial, 5 or 6 cuts happen, one every 11 to 12 Cycles, the first at about Cycle 41, and each drops about 21 entries. The check trial of [Make the history size a Connection field](09-history-size-field.md) measured it (`tmp/ltm09/redmist_julian/trial-1`), and that pack is the reference trial of this map. The setup file is the only place that sets the size: `scripts/trial.py` removes `max_history_tokens` when `history_tokens` is absent. So a candidate setup must keep `history_tokens: 16000`, or nothing leaves the history in 100 Cycles.
- **Which calls have a history space (known, ticket 09).** Every call that writes a Narration, an Impersonate Input or an event Narration (`CONTEXT.md`, **History space**). So a memory block is needed in all three, not only in the Narration call.
- **A memory block can move the cut (known, ticket 09).** The history space is the smaller of the history size and the free space in the window. A memory block is a fixed part of the prompt, so it takes free space. The probe Connection `conn-1793449507061-3a1cb15f` has no `max_context_tokens`, so its window is 32,768. The 1,000,000 of `data/settings.json` reaches a fresh DB only. At 32,768, a large memory block can push the free space under 16,000, and the cut then falls earlier than in the reference trial (inferred). Each prototype ticket checks this, or sets the window of the probe Connection to 1,000,000 on both sides.
- **A draft from the paused session (coordinator, not decided).** Round 1 of this ticket proposed these answers. The owner did not answer them, because the history cut came first. Read them again after ticket 08.
  - The candidates also give a record of who was present at each entry, and the record also marks the entries that the history still holds. 5 of the 9 confirmed faults of story A are who-knows-what faults inside the history, so a memory of dropped entries cannot fix them.
  - The trigger is the history cut: a fact leaves only at a cut, and the cut already breaks the cache at the start of the history.
  - A Character memory holds what the Character took part in, as the story tells it (also a phone call, a letter or a reported off-stage event), and not a talk that it did not hear. The writer decides who took part, from the text, not from the Characters present.
  - Four candidates: C1 a rolling summary at each cut; C2 scene recaps that name who took part; C3 C2 plus a short section for each Character; C4 a word search (BM25) over the dropped entries, with no writer. C2 first, with C1 in the same batch.
- **The owner's working view** (map Notes). The research tests it, and this ticket keeps or changes it.
  - The story memory and the Character memories are different in what they are and in their structure.
  - The story memory changes with everything that happens, whoever is present. A memory for the Persona is most likely the same as the story memory.
  - A Character memory keeps track of what happened to that Character, also after the Character leaves the scene.
- **Open questions from charting, for this ticket:**
  - **The trigger.** When a memory is written: at a step of the history cut (`HistoryCut`, the compaction point), after each scene (the ME shape), after each Cycle, or a mix.
  - **The position.** Before the history (the text changes rarely, and breaks the cache only on the Cycle that changes it) or after the history (the text changes often). The position follows from the trigger ([how other tools keep long-term memory](../../prompt-flexibility/assets/long-term-memory.md), section 5).
  - **A Character memory while the Character is absent.** Does it change with what happens to that Character off stage (for example, "Carla phoned: the police held her overnight")? Or does it change only with what happened while the Character was among the Characters present? At charting the coordinator recommended "only while present". The owner chose to research first.
  - **What a Character memory holds.** Everything that the Character saw and heard, or only what happened to it. A who-knows-what fault: Carla knew the whole talk about the wax after she heard one word of it.
  - **One call or two.** Whether the story memory and the Character memories share one writer call.
- **The Characters present are not exact (known).** The Quantifier drops a Character who is on stage, and keeps a Character who left. The judges of ticket 15 of Layout trials counted 14 to 20 runs of a wrong State line in one story (`tmp/lt15/judge/`). A Character memory that follows the Characters present inherits this fault.
- **The baseline (known, [Count the memory faults of the newest trial](03-baseline-faults.md)).** No confirmed fault in the two newest stories is a fact that had left the history. The story drops old threads and does not contradict them. The who-knows-what faults point to a record of who was present at each talk. Three of them are card leaks: Julian knows facts that only a card states.
- **The research.** [Find how Marinara Engine keeps memory with its agents](01-marinara-memory.md) and [Find what is known about long-term memory for LLMs, and what is still open](02-llm-memory-research.md).
- **The owner label and lore entries (owner, 2026-11-03).** A Character memory records whose it is, the same as a card ([Decide the structure of a Character card and who knows each part](10-card-structure.md), Answer): the model sees it, and the rule text of the card block says that she knows her own card. A lore entry that a writer creates is one possible shape of story memory, as in the ME Lorebook Keeper and ST MemoryBooks ([Decide how a Character or a lore entry enters the prompt](12-prompt-entry-modes.md)). Neither ticket blocks this one, because the memory test changes only `<StoryMemory>`.
- [Make the memory test from the stored stories](04-memory-test.md) is resolved. Read how a candidate plugs in: it changes only a `<StoryMemory>` block.

## Output

A resolution with 2 to 4 candidates. Each candidate is sharp enough for one prototype ticket that writes its memory on a stored story and runs the memory test. Name the candidate that the map tries first. Create the prototype tickets.

## Answer

Resolved on 2026-11-11, with the owner. The ticket chooses no candidates on paper. The engine gets a base story memory first, and each variant becomes a change to that base.

**The decision (owner, 2026-11-11).** "We have some things that we know we need to do, and it would be simple if we just did those things and then made modifications on top of them, rather than starting from a zero position." The basics are known: the cut part of the history must not be lost, the history rolls forward, and a summary holds the part that rolled out. The details are open. A variant without the basics in the engine is abstract.

**The base (owner agreed to this list).**

1. **Summarization at the cut.** When Messages leave the history, a writer call first summarizes them into the story memory.
2. **The story memory is in the prompt.** Every call with a history space shows it, before the history: the Narration, the Impersonate Input and the event Narration. The history is already one chat message for each entry (known: `render_history_entry` in `src/application/prompting/assembler.rs` at `4baba0a4`, and call 1670 of `tmp/ltm09/redmist_julian/trial-1`). So the story memory goes at the end of the first user message, before the history messages.
3. **Storage for each Game.** The story memory is saved with the Game and tied to the Swipe that caused the cut, so a Retry or a delete removes it.
4. **A failed writer call loses nothing.** The previous story memory stays ([Keep the state when a state writer fails](../../prompt-flexibility/issues/23-keep-state-when-a-writer-fails.md)).
5. **A trial shows it.** The trial pack shows the story memory of each Cycle.

**The player's story does not change (owner, 2026-11-11).** The story that the player sees is the chat history, whole, as if nothing was summarized. The story memory goes to the model only. It is not part of the story log. It can be visible for debugging. Known: storage keeps every Message, and the cut changes only what a call sends (map Notes, "The engine today"). In the base, a person reads the story memory in the stored call record and in the trial pack. The view and the edit in the dashboard come later (map, Not yet specified).

**The first form (coordinator).** One summary text that the writer rewrites at each cut from the previous text and the Messages that leave. It is the simplest summarization. The measured figure for one summary of all the dropped entries is 67 % on the memory test ([the memory test](../assets/memory-test.md)).

**The changes on top, in this order (coordinator, from the research).** Each one is a change to the built base. The memory test scores it, and a trial confirms it.

1. The form of the story memory: a list of scene entries that only grows (the time, the place, who was there, what happened), against the one rewritten summary. Marinara Engine writes one recap for each scene ([01](01-marinara-memory.md)). The one summary lost who did small things, reasons and who learned what.
2. The Character memory: for each Character, what she did, was told and learned. It also changes with what happens to her while she is absent (owner, charting, Q13 of the session of 2026-11-02). The prompt shows it inside her card.
3. Loading a memory entry by a key, with the mechanism of [Decide how a Character or a lore entry enters the prompt](12-prompt-entry-modes.md).

**What is not in this ticket.** The who-knows-what faults of the baseline are not caused by the cut: in each one the text was still in the history ([baseline faults](../assets/baseline-faults.md), "Kind 3"). A memory of the cut part cannot fix them. They go to [Decide how a Character or a lore entry enters the prompt](12-prompt-entry-modes.md), which decides which card text stays out of which call, and to the fog entry "More of the memory test".

**The draft candidates C1 to C4 of the Context are withdrawn.** C1 is the first form of the base. C2 and C3 are the changes 1 and 2 above. C4 (a word search over the dropped entries) is not planned. The keyed memory entry of change 3 covers its idea.

**How the grilling went (for the next coordinator).** Three question rounds failed before this answer. The owner did not want to choose a test method or a design detail by opinion: at charting the owner had set "research, then trials, then the design". The wiki page [Ask in the terms of the application](../../../wiki/owner-preference-questions-in-application-terms.md) holds the rules from this.

**Evidence.** A scout wrote [the evidence brief of this ticket](../assets/memory-candidates-brief.md) from the research assets. The coordinator did not check it line by line. Three points from it:

- A 100-Cycle story is about 41,000 tokens, and the default history size is 65,536. So at the default nothing leaves the history in 100 Cycles, and the whole-story control scores 95 %. The 16,000 tokens of the trials force the cuts.
- The memory test cannot score a record of who was present, and it cannot score the "does not know" side.
- The test's cut points come from story A (`tmp/lt15/new/trial-1`): 5 cuts, the first at entry #611 (Cycle 44). The "5 or 6 cuts, the first at about Cycle 41" of the Context come from `tmp/ltm09/redmist_julian/trial-1`.

**A tool fix.** The move of the work records to `work/` broke `memory_test.py` and `card_probe.py` (the repo root lookup). Fixed on 2026-11-11 ([wiki](../../../wiki/work-record-tools-after-the-move.md)).

**New tickets.** [Find where the base story memory attaches in the engine](13-base-memory-seam.md) and [Build the base story memory](14-build-base-story-memory.md).
