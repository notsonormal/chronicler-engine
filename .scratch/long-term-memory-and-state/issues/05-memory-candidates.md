# Choose the memory candidates to try

Type: grilling
Status: open
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
- [Make the memory test from the stored stories](04-memory-test.md) is resolved. Read how a candidate plugs in: it changes only a `<StoryMemory>` block.

## Output

A resolution with 2 to 4 candidates. Each candidate is sharp enough for one prototype ticket that writes its memory on a stored story and runs the memory test. Name the candidate that the map tries first. Create the prototype tickets.
