# Re-read of the memory research, 2026-11-14

Two read-only digests, written on 2026-11-14 by two `reviewer` subagents for the coordinator. The owner had said that the map lost sight of where it wants to get in principle. Each digest reads the research assets in full and lists what the map carried forward and what it dropped. The text of each digest is unchanged. The coordinator did not check them line by line.

## Digest: story tools (Marinara Engine, SillyTavern and others)

Session `be6b92db-c587-41a4-ad88-156a9b519e3e`.


Each sentence ends with **K** (the file states it), **I** (I concluded it) or **G** (guessed). The files are:
- **[MM]** `long-term-memory-and-state/assets/marinara-memory.md`
- **[LM]** `prompt-flexibility/assets/long-term-memory.md`
- **[LS]** `prompt-flexibility/assets/long-term-state.md`

### 1. The parts

None of these tools has one memory. Marinara Engine (ME) has eight separate writers, and each one owns its own store. **K** [MM, Short answer]

- **Scene recap (ME Advanced Memory).** A model call finds where a scene ends, checking every 5 Messages. A second model call then writes one recap of each closed scene, with the list of Characters present and a story timeframe. **K** [MM §1.1]
- **Recollection (ME).** It is 50–100 words in the first person, one for each Character present in the scene. It is written in the same call as the recap. **K** [MM §1.1, §3.1]
- **Append-only summary entries (ME Chat Summary).** A model call adds an entry every 5 Messages. The prompt says "Do not rewrite or repeat the previous summary." **K** [MM §1.1, §1.4]
- **Rolling summary (SillyTavern Summarize).** One text, rewritten by a model call every 10 Messages. **K** [LM §2.1]
- **Vector recall.**
  - SillyTavern Vector Storage embeds each Message. **K** [LM §2.1]
  - ME Memory Recall embeds chunks of 5 Messages. Neither one makes a model call. **K** [MM §1.1]
- **Lorebook entries.** The ME Lorebook Keeper writes durable facts every 8 Messages and pins each entry to its Message and Swipe. **K** [MM §1.1]
- **Reminder vault (ME Memory Nag).** Promises, debts, injuries and similar, each tied to Characters. A scan runs every 20 Messages. **K** [MM §1.1, §1.4]
- **Typed notes (ME Long-Term Memory package).** Timeline events, Character facts, relationships, world facts, open threads, anchors and tone. A person starts the extraction and reviews it. **K** [MM §1.1, §9.1 item 8]
- **Trackers.** A model call after each reply extracts the date, time, place, who is present and relationships. The result is stored once for each Message and Swipe. **K** [LS §2.1]
- **Hand-written texts.** Author's Note, AI Dungeon Plot Essentials, NovelAI Memory. **K** [LM §2.1]
- **AI Dungeon.** A summary of every 6 Actions, embedded, plus a Story Summary refreshed every 15 Actions. **K** [LM §2.1]
- **Generative Agents (a research paper).** A memory stream for each agent, plus "reflections" written when the summed importance passes 150. **K** [LM §2.1, LS §2.6]

### 2. Selection

- **Always in the prompt:**
  - SillyTavern Summarize, placed before the history. **K** [LM §2.1]
  - ME Chat Summary and the Advanced Memory summaries, before the history. **K** [LM §2.1]
  - ME trackers, placed before the newest history block. **K** [MM §2.2]
  - AI Dungeon Story Summary and Memory Bank, fixed positions 4 and 5 of 9. **K** [LM §2.1]
- **Chosen for each call:**
  - ME recalled scenes are ranked by text match, embeddings and an optional model call. Then they are filtered by who was present. They go in at a marker in the preset. **K** [MM §1.1, §3.1]
  - SillyTavern vectors: the top 3 by similarity, at depth 2. **K** [LM §2.1]
  - ME Memory Recall: similarity of at least 0.25, at most 8 chunks, placed before the first history Message. **K** [MM §10.1 M2]
  - Lorebook entries: by key match. **K** [LM §2.1]
  - Memory Nag: a model picks at most 3 from a shortlist of 5 for each Character. **K** [MM §1.4, §2.4]
  - Generative Agents: a score of recency plus importance plus relevance. **K** [LM §6]

### 3. Bounding

- **ME Advanced Memory:**
  - When the summaries pass 70 % of their budget, a model call combines and shortens them. **K** [MM §7.2]
  - The live history resets to the start of the latest scene. **K** [MM §7.2]
  - When recall runs out of space, excerpts shrink first and whole recollections go next. **K** [MM §7.2]
- **Chat Summary:** at most 200 entries, and the oldest go first. **K** [MM §7.2]
- **SillyTavern Summarize:** a target length, 200 words by default. **K** [LM §2.3]
- **Memory Nag:** a cap on each scan. I found no cap on the whole vault. **I** [MM §7.2]
- **Trackers:** one snapshot for each Message, so they do not grow. **I** [LS §2.1]

**Going back to the original Messages: yes.** The vector stores return the original Message text. ME recall includes Message excerpts. ME also turns the old part of an open scene into an excerpt. **K** [LM §2.1; MM §7.2]

### 4. Who knows what, and time

**Who knows what (ME Advanced Memory):**
- The recap call writes the list of who was present. A Character who is "merely mentioned … is NOT a participant." **K** [MM §1.4]
- One recap can hold private parts for named readers. **K** [MM §3.1]
- A start point can be set for what each Character knows. A Message can be hidden from named Characters. **K** [MM §3.1]
- A four-word copy test checks for leaks. It does not catch a fact retold in other words. **K** [MM §3.1]
- It works only in Roleplay chats. The start points and the hidden Messages work only in Individual group chats. **K** [MM §3.1]
- An absent Character gets nothing. The Relationship Tracker, Memory Nag and the typed notes do change while a Character is absent. **K** [MM §3.2]

**Time:**
- The ME World State tracker writes a date and a time as free text for each Swipe. The prompt sets the rule: a minute or two for each beat of talk, a few hours at a scene change. **K** [MM §4]
- A scene recap gets its time from a regex over the Message text, or else "unknown (use message order)". **K** [MM §4]
- No code does arithmetic on time. **I** [MM §4]
- The Continuity Checker rewrites a reply that breaks time. **K** [MM §4]

### 5. Evidence

- **ME:** many plumbing tests, all with a scripted model. **K** [MM §8.1]
- **Memory quality:** none of the files found any benchmark or user report for any tool. **K** [MM §8.1, §11; LS Limits]
- **Real faults:** the ME changelog records them, for example scenes that never ended and the wrong scene recalled. **K** [MM §8.2]
- **Research papers:** models use the middle of a long context worse than its ends (*Lost in the Middle*). The working window is smaller than the stated window (*RULER*). Long conversations are still hard for models (*LoCoMo*). Generative Agents shows by ablation that reflection matters. **K** [LM §6]
- **Every other claim in sections 1–4** describes how a mechanism works. None is evidence that it works. **I**

### 6. The rolling summary

- **Who uses it.** SillyTavern Summarize rewrites one text. Older summaries "stay as text inside the newest one." **K** [LM §4]
- **ME does not.** Its Chat Summary appends instead of rewriting. **K** [MM §1.4]
- **Known weaknesses:**
  - ME's own docs: "summaries can miss nuance … No system can reconstruct details never recorded." **K** [MM §8.1]
  - In SillyTavern, a Retry of the last Message deletes its summary. **K** [LM §4]
  - It is one text for the whole chat, with nothing for each Character. **K** [LM §3]
- **What the tools pair it with:**
  - SillyTavern: vectors, World Info, Author's Note. **K** [LM §2.1]
  - AI Dungeon: the embedded Memory Bank and Plot Essentials. **K** [LM §2.1]
  - ME Advanced Memory: its summaries sit next to scene recaps, recollections and excerpts. **K** [MM §2.4]

### 7. What the map carried forward and what it dropped

**Carried forward (K, from the map Notes and the ticket 05 Answer):**
- The rolling summary: the base the engine built.
- The summary placed before the history and tied to the Swipe, and a failed writer keeps the previous text.
- Scene entries as change 1.
- The Character memory as change 2. It also changes while the Character is absent, which goes against ME's main writer.
- Loading an entry by a key as change 3.
- The clock as ticket 06, still open.
- The card change pinned to a Message and a Swipe as ticket 07, still open.
- A label for who knows each card part (`secrets`).

**In none of the map's plans:**
- **Recall of original Messages by search.** The word-search candidate (C4) was withdrawn. No vector recall or excerpt recall is planned. **K**
- **A scene end as the cut point.** It was open in ticket 05's Context. The Answer did not take it up. **K**
- **Who was present, written by the memory writer.** ME takes this from the recap call, not from the tracker. The map does not say where its list will come from. **I**
- **Per-Character visibility of Messages, start points for knowledge, and a leak check.** The who-knows-what faults went to ticket 12, which is about cards only. **I**
- **A combine step.** Here ticket 15 (story memory size) is open, so the idea is not yet dropped. **I**
- **A reminder vault for promises and debts, and open threads that carry a resolver.** **K**
- **A consistency checker that rewrites the reply.** **K**
- **Ranking by importance and recency, and reflection** (Generative Agents). **K**
- **The main finding about structure:** the working tools pair a summary with several separate stores, each with its own way to be chosen. The map's order treats the parts as changes, one after another, to one summary. **I**

## Digest: the field research and the faults of our own stories

Session `910fa784-7810-4c0b-8851-7a802721a5d9`.

The sources do not support the rolling summary that the map built. The research rates that design weak, and in the trial stories the main loss is dropped threads, not contradictions. Details follow, one section for each of your questions.

Files: **R** = `assets/llm-memory-research.md`, **BF** = `assets/baseline-faults.md`, **SI** = `layout-trials/assets/story-inconsistencies.md`, **M** = `map.md`, **T05** = the Answer of `issues/05-memory-candidates.md`. Labels: K = known (the file states it), I = inferred (my conclusion), G = guessed.

### 1. The families
R §1 describes 12 families plus a full-context control. All evidence numbers are from the R §1 table and §2 (K).
- **A. Rolling summary.** One text, rewritten at each chunk from the old text and the new turns. On DMR it scored 35.3%, against 94.4% for full context.
- **B. Hierarchical summary** (RAPTOR, ReadAgent). Summaries at several levels, some with a link back to the source page. Tested on documents only: RAPTOR +20 points on QuALITY.
- **C. Embedding retrieval.** Raw turns with vectors, top-k at read time. LoCoMo accuracy peaks at k≈5. LongMemEval: units of one round beat whole sessions.
- **D. Extracted facts with add, update and delete** (Mem0). Mem0 scored J 66.9, below full context at 72.9, with 91% lower p95 latency.
- **E. Temporal knowledge graph** (Zep). Facts with valid-from and valid-to times; raw episodes are kept. Vendor result: +15.2 and +18.5% over full context on LongMemEval_S. Its LoCoMo score is disputed: 58.44, 65.99 or 75.14.
- **F. Memory the model edits** (MemGPT, Letta). MemGPT 93.4% on DMR.
- **G. Linked notes** (A-MEM). J 48.4 in the Mem0 re-run.
- **H. Reflection** (Generative Agents). One stream for each agent. Ablation results only.
- **I. Narrative plan and state** (Re3, DOC, RecurrentGPT, NstAgent). Human raters: Re3 +14 points coherence, DOC +22.5 over Re3.
- **J. Agentic search.** A Letta plain-file agent scored 74.0% on LoCoMo (vendor).
- **K. Hand-written or pinned memory.** Product claims only.
- **L. A filtered view for each reader.** A leak fell from 60.7 to 0 points.

### 2. What fails and why
- **A, the rewritten summary** (K):
  - **Loss.** "Old detail is lost at each rewrite" (R §1). On LoCoMo, retrieval over summaries "does not help although its recall is high", and the authors blame information loss (R §2.2, §7.4).
  - **Facts only.** LongMemEval: reducing the history to facts only hurts the total score (R §2.1).
  - **Growth, drift, hallucination.** R names no measured result on these. It notes that the F writer "can drift, and nothing checks it" (I: the same applies to A).
  - **Our own test** (`issues/14-build-base-story-memory.md`, ticket 14 Decisions line in M): the summary size is not under control.
  - **Our own memory test** (M Decisions, ticket 04): the summary loses "who did small things, reasons and who learned what".
- **B:** a tree does not mark who saw what (I, R §1).
- **C:** the turn must name the entity. Weak for time and for "who". More items hurt (K).
- **D:** delete loses the old value (I, R §1.1).
- **E:** the heaviest cost (K).
- **F and J:** they need a model that uses tools well (K/I).
- **I:** the Re3 checker gave "a sea of false positive contradictions". Its Edit step "makes little difference" (K, §3).
- **L:** tested only in short template scenes (K).
- **Full context:** 30–60% below the oracle on long histories (K).
- **After retrieval:** a correct retrieval followed by a wrong answer is 40–50% of LongMemEval errors (K, §7.5).

### 3. Write time against read time
R never frames this question (I). The related evidence is all known:
- Retrieval over write-time observations beat raw turns by about 5 points (LoCoMo).
- Write-time summaries gave no gain (LoCoMo).
- Adding extracted facts to the search keys gave +9.4 recall and +5.4 accuracy (LongMemEval).
- Full context, where nothing is selected at write time, scores highest while it fits (Mem0, Zep).
- Read-time precision matters: small k wins.

My reading (I): compressing at write time loses facts. Selecting at read time over kept raw text does better, and write-time extraction helps most as an extra index.

### 4. Changed facts, time, who knows what
- **Changed facts** (K, §2.6, §5, §7.1):
  - This is unsolved. Every method scored at most 28% on multi-hop forgetting.
  - Only Zep keeps the old value, with valid times.
  - About 10% of LongMemEval failures came from the retriever finding the new fact and missing the old one.
- **Time** (K, §5, §7.2):
  - This is the weakest question type. On LoCoMo, models lag humans by 73% on time questions.
  - Memories with no timestamp scored below 15 J.
  - Time-aware query expansion gave +6.8 to +11.3 recall.
  - No source keeps a clock inside the fiction.
- **Who knows what** (K, §4, §7.7):
  - In FANToM, models do worse on the "who knows?" list questions than on choice questions.
  - In the Curse paper, 11 open models leak at 67–72 points.
  - Changing only the layout of the input breaks the boundary.
  - A view that the system builds outside the model fixed the leak.
  - A prompt that tells the model to filter itself cut the leak by 64.7%, but it also dropped allowed content.

### 5. Story-specific findings
- In story methods the memory serves the writer, not a reader's question (K, §3).
- Re3's ablation: the Plan and Rewrite modules are critical. RecurrentGPT combines an editable plan, a short-term memory rewritten at each step, and a vector store of paragraph summaries (K).
- ConStory: errors grow about linearly with length. Its taxonomy has "abandoned plot elements", duration, knowledge and quantity subtypes (K).
- NstAgent tracks "future requirements" (K, abstract only).
- LoCoMo: long-context models misassign events to the wrong speaker (K, §4).
- No benchmark exists for an interactive story with several Characters (K/I, §7.9).
- The story methods were tested on stories of 2,000–6,000 words (K).

### 6. The faults in our own stories
- **Pilot stories, SI:**
  - §1: 7 facts that had left the history (P 4, Q 3) (K).
  - §2: 5 contradictions of a fact that was in the prompt. A stronger model or a list of fixed facts may help (G).
  - §5: 6 clock faults (I).
  - §6: 3 who-knows-what faults (I).
  - §4: 2 short-entry inventions (K).
- **Checked baseline, BF:**
  - Story A: 0 facts that had left the history, 4 clock faults, 5 who-knows-what faults. Story B: 0, 1 and 3.
  - A further 9 date conflicts inside the history (A 8, B 1) were moved to another fault kind (K).
  - BF guesses these fixes: a story clock (A-2.3, A-2.4, B-2.1); a record of who was present at each talk (A-3.1, A-3.2, A-3.5, B-3.1, B-3.2); and a rule that keeps card facts from a Character who has not learned them (A-3.3, A-3.4, B-3.3).
  - BF also says the Draught dates need a list of fixed facts.
  - "A memory of the dropped entries fixes none of the confirmed faults" (I, BF).
- **Old threads:** "The story loses its old facts by dropping threads, not by contradicting them". The login, the master-key handoff, Gabriella and Jezebel are never used again (I, BF Summary). In my count, 31 of the 40 fact rows have no later use (I).

### 7. What the map used and dropped
**Carried forward.** All of these are known from M and T05:
- From R: full context wins while it fits, which sets the 65,536 default (ticket 08). The memory-test methods: the controls and the noise runs (ticket 04).
- From the Marinara Engine survey (ticket 01): the scene recaps (change 1).
- A Character memory (change 2) and loading by a key (change 3).
- Who knows what moved to ticket 12. The clock is in open ticket 06.
- The base chose family A, which R rates "Weak" (I).

**Appearing nowhere in the plan** (I, checked against M and T05):
1. Keeping the old value and a valid time when a fact changes (Zep).
2. Read-time retrieval over kept raw text. The word search (C4) was withdrawn.
3. A plan or outline with future requirements (Re3, DOC, RecurrentGPT, NstAgent). This bears on the dropped threads and ConStory's "abandoned plot elements".
4. A link from each memory item to its source text (ReadAgent, Zep).
5. Summaries at several levels.
6. A record of who was present at each talk. It is in the withdrawn draft only.
7. Chain-of-Note and a structured format for the reader.
8. An attribute dictionary for each Character (Re3).
9. Agentic or file search.
10. Writing the memory in the background or between turns.
11. ConStory's taxonomy for the judge.

The finding that a summary loses detail (LoCoMo) is not cited against the base. T05 cites only the 67% score.

## Coordinator correction: Marinara Engine does rewrite its always-on summary

Added on 2026-11-14, from the user guide of Marinara Engine (`docs/agents/memory.md` at commit `457e827a`) and section 7.2 of [marinara-memory.md](marinara-memory.md).

Section 6 of the first digest says that Marinara Engine does not use a rewritten summary. That is true only of the plain Chat Summary writer. Advanced Memory has two levels:

- **Always sent.** One Chat Summary entry for each scene that left the live history. When these entries pass 70 % of the memory budget, a background call combines and shortens them. This step loses detail, the same as the base story memory.
- **Sent when chosen.** The recap of each scene stays in the archive, with the original Messages. At most 3 scenes go into a call, each with its recap and an excerpt of 3 to 10 original Messages. A text match with the newest Messages chooses them.

So the design does not avoid a lossy summary. It adds a second path that brings the detail back.
