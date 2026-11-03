# Choose the memory candidates to try

Type: grilling
Status: open
Blocked by: 01, 02, 08

## Question

Which memory designs does this map try in the memory test? For each candidate: what it stores, which call writes it and when, where the memory goes in each call, and what it costs.

## Context

- **The history cut comes first (owner, 2026-11-03).** The size of the history space was never decided. It decides how much leaves the prompt, and so what a memory must hold. This ticket waits for [Decide the size and the point of the history cut](08-history-cut.md).
- **What leaves the history (known, from ticket 08).** The default history size becomes 65,536 tokens, a Connection field. With the default, nothing leaves the history before about Cycle 155 (inferred). The trials of this map run at 16,000 tokens, as the stored stories did, so about 5 cuts happen in a 100-Cycle trial. The cut stays a size rule with a 75 % block. A scene end as the cut point is open to this ticket.
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
- If [Make the memory test from the stored stories](04-memory-test.md) is resolved, read how a candidate plugs in.

## Output

A resolution with 2 to 4 candidates. Each candidate is sharp enough for one prototype ticket that writes its memory on a stored story and runs the memory test. Name the candidate that the map tries first. Create the prototype tickets.
