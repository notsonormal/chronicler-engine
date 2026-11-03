# Find what is known about long-term memory for LLMs, and what is still open

Type: research
Status: resolved
Blocked by: —

## Question

Which families of long-term memory for LLMs exist, and what evidence shows how well each one works? Which problems are still open? Which families fit a long interactive story with several Characters?

## Context

- The owner said that long-term memory is "not really a solved problem" (charting). This ticket gives the map the general field, so that the map does not copy one tool. [Find how Marinara Engine keeps memory with its agents](01-marinara-memory.md) covers ME in depth, so this ticket gives ME one row only.
- [The earlier survey](../../prompt-flexibility/assets/long-term-memory.md), sections 2 and 6, already covers ST Summarize, ST Vector Storage, AI Dungeon, NovelAI, Generative Agents and MemGPT, with LoCoMo, *Lost in the Middle* and RULER. Start from it, and do not redo it.
- The three memory faults (map Notes) are: a fact that had left the history, no story clock, and who knows what. [The inconsistencies of two trial stories](../../layout-trials/assets/story-inconsistencies.md) has examples of each.

Points to cover:

1. **The families.** For each family: what it stores, which call writes it and when, how it finds what goes into a prompt, and its cost. Cover at least these:
   - rolling and recursive summaries, and hierarchical summaries;
   - retrieval of past text by embedding;
   - memory as extracted facts that a writer adds, changes or deletes (for example Mem0);
   - knowledge graphs with time (for example Zep and Graphiti);
   - memory that the model edits itself (MemGPT, Letta);
   - linked notes (for example A-MEM);
   - reflection (Generative Agents).

   Add each family that the sources show and this list does not name.
2. **Evidence.** The benchmarks for long conversations (for example LoCoMo and LongMemEval), and their results for each family. Say what each benchmark measures and how it asks its questions. [Make the memory test from the stored stories](04-memory-test.md) copies the method.
3. **Stories.** The research on coherence in long generated stories (for example Re3, DOC and RecurrentGPT): how each one keeps its facts and its plan.
4. **Who knows what.** Methods that keep what one agent or Character knows apart from what another knows. Examples: one memory stream for each agent, and research on theory of mind in stories with several characters.
5. **Time.** How the methods record time and the order of events. How they handle a fact that changes (the debt was 740,000, then it was paid).
6. **Roleplay practice.** The maintained memory extensions of SillyTavern other than Summarize and Vector Storage. What roleplay products (for example Character.AI, Kindroid or Nomi) say about their memory. Keep the claim of a product apart from a measured result.
7. **Open problems.** What the sources say is not solved, with the source.

Use papers, official repositories and official docs. Record the date and the URL of each source. Label each claim known, inferred or guessed.

## Output

An asset `assets/llm-memory-research.md` with:

- one table of the families: what is stored, who writes it, when, how it is found, its cost, its evidence, and its fit for a story with several Characters;
- one section for each point;
- the open problems.

Keep each method to one table row and a short paragraph. Do not summarize a whole paper. Add facts for these tickets:

- [Make the memory test from the stored stories](04-memory-test.md)
- [Choose the memory candidates to try](05-memory-candidates.md)
- [Decide how the story keeps its time](06-story-clock.md)

This ticket finds facts. It does not choose a design.

## Answer

Asset: [What is known about long-term memory for LLMs, and what is still open](../assets/llm-memory-research.md). It holds the table of 12 families, one section for each of the 7 points, the open problems, and the facts for tickets 04, 05 and 06. Access date of every source: 2026-11-03.

- **known** On public benchmarks, a full-context run beats every memory system on accuracy when the history fits (Mem0 paper: 72.9 J against 66.9 for Mem0 and 68.4 for its graph variant; Zep paper: DMR 94.4% to 98.0% for full context). Memory systems win on tokens and latency. Both papers come from vendors.
- **known** Updating a changed fact and time are the weakest points. MemoryAgentBench: all methods reach at most 28% on multi-hop forgetting. LoCoMo and the Mem0 test: temporal questions are the hardest. Only Zep's design keeps the old value with a valid-from and valid-to time.
- **known** Summaries lose detail (LoCoMo: no gain from summary RAG despite high recall). Retrieval of finer units and of extracted facts helps (LongMemEval: rounds beat sessions; fact-expanded keys gave +9.4 recall points).
- **known** For who knows what, one family is built for it: a filtered view for each reader. In the Curse of Knowledge preprint (2026-10), 11 open models leaked hidden facts at 67 to 72 points, and a "character view" made by the system before the prompt cut the leak from 60.7 to 0 points. Prompted self-filtering was weaker. The test used short template scenes with one hidden fact.
- **known** FANToM and the Curse paper give the method for a who-knows-what test: a fact asked in several forms with a strict "all forms right" score, and twin scenes that differ in one hidden fact. LongMemEval and ConStory give the method for a fact test: labelled evidence, a judge prompt for each question type, oracle and full-context controls, and a checker that quotes both spans of a contradiction.
- **known** Story methods (Re3, DOC, RecurrentGPT) keep a plan and Character facts for the writer, and raters prefer them over a rolling window. Re3's per-Character attribute check "makes little difference". None was tested on a story of 100 Cycles.
- **known** Roleplay products publish claims and no benchmark. Kindroid says each Kindroid in a group keeps its own memory, with a toggle to share the short-term memory. Character.AI and Kindroid let a person see and edit the memory.
- **known** Benchmark numbers are not comparable across vendors. The Zep result on LoCoMo is reported as 58.44, 65.99 or 75.14. A plain-file agent scored 74.0% in a Letta blog. Surveys say benchmarks are underscaled and judges are sensitive.
- **inferred** The debt case ("740,000, then paid") is a knowledge-update case. A store that overwrites keeps only the new value. A store with a valid time answers both "then" and "now". A fact store with DELETE (Mem0) is risky for a story.
- **inferred** The three memory faults map to benchmark types: a fact that left the history (single-hop and multi-hop), no story clock (temporal, ConStory duration), who knows what (FANToM access questions, abstention by false premise).
- **guessed** A candidate that filters the memory by the Characters present in each Swipe state, and keeps source Message IDs, fits both the Character memories and the Retry rule. No source tests it.
- **I don't know** Any method that keeps an in-story clock, any benchmark for an interactive story with several Characters, and whether a Character memory should change while the Character is absent. I found no source that tests the last point.
