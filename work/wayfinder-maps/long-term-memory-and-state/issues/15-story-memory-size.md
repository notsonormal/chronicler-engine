# Decide how the story memory keeps its size

Type: prototype (HITL)
Status: resolved
Blocked by: 14

## Question

How does the story memory stay inside a fixed space for a story of any length, without the loss of what the narrator needs?

The answer is one concrete change to the base story memory, tried on the engine and measured. It states what the engine stores, what each writer call gets and writes, and what happens to the oldest part at the limit.

## Context

- **The base** ([Build the base story memory](14-build-base-story-memory.md)). At each history cut, the writer rewrites one text from the previous text and the Messages that leave. The limit is 2,000 tokens, and the prompt asks for 600 words. An answer at the limit is a failed writer call that keeps the previous text.
- **The problem (known).** The size of the rewritten text is not under control with `deepseek-v4.1-flash`. The model adds the new events and shortens little. Five of nine runs of the memory test reached the limit at the fourth or fifth cut. Two trials in the engine stayed under the limit. No wording of the prompt fixed this, and a wording with two sections made the model return the old memory unchanged.
- **The effect in play (inferred).** After a writer call fails at the limit, the story memory stays at its last good text. The Messages of that cut are in neither the history nor the story memory. The next writer call gets more Messages, so it is likely to fail again.
- **The evidence**: [Evidence: the size of the story memory](../assets/story-memory-size.md). It holds the sizes of every run, the writer calls of both trials, and what is not known.
- **The planned next change** ([Choose the memory candidates to try](05-memory-candidates.md)). The story memory as a list of scene entries: each cut adds entries, and no old entry is rewritten. This removes the rewrite, but a list also grows. It is one possible answer here, not a decision.
- **The owner's rules** (map Notes). The player's story stays whole. The coordinator chooses the design details from the research, states them, and lets a trial judge. The owner gives a go or no-go on the concrete change, and then gets the results.
- **The research**: [Find how Marinara Engine keeps memory with its agents](01-marinara-memory.md) and [Find what is known about long-term memory for LLMs, and what is still open](02-llm-memory-research.md) describe how other systems bound a memory (a second level of summary, a cap on entries, loading by a key).

## What the answer needs

- **A measurement over more cuts.** The stored story of the memory test has 5 cuts, and a 100-Cycle trial has 5 or 6. The problem shows at the fourth cut and later. A smaller history size gives more cuts on the same story.
- **The engine's rule in the measurement.** The memory test keeps a text that was cut off and continues. The engine counts it as a failed call. A score that hides this is not the score of the engine.
- **More than one run for each change.** Runs differ, and at temperature 0 some runs repeat.
- **The cost.** The writer calls of the base use under 1 % of the tokens of a trial, and the cached share of the Narrator-role calls stays at 85.5 %. The change must report both.

## Output

The answer names the change, its measured sizes and scores against the base, and the build ticket that follows.

## Answer

Resolved on 2026-11-14, with the owner, in a conversation. The measurement that this ticket asked for did not run. The answer is a change of the form of the story memory, and the measurement moves to [Decide the form of the scene entries](16-scene-entries.md).

**The answer.** No part of the memory is a text that a writer rewrites from its own earlier version. The story memory becomes a list of scene entries. The writer writes each entry once, from the Messages of that scene, and the entry never changes. When the list passes its space, the oldest entry leaves the prompt and stays stored. A person can keep an entry in the prompt or edit it in a memory view of the dashboard. What must outlast a scene goes to two other lists: the Character memory and the world facts.

**Why the question changed.** The owner said that the map had lost sight of where it wants to get in principle. Two readers read the research again in full ([re-read of the research](../assets/research-reread-2026-11-14.md)). Three findings changed the question:

- One rewritten summary is the weakest form in the research. It scored 35.3 % against 94.4 % for the whole history in one benchmark ([LLM memory research](../assets/llm-memory-research.md), section 1, family A). The base took this form on 2026-11-11 as the simplest plumbing, and nobody held the research against it afterwards.
- The size problem of this ticket is an effect of the rewrite. The writer gets its own earlier text and must choose what to lose, and the model avoids the choice. A writer that sees only the Messages that leave cannot grow its answer from cut to cut.
- The owner (2026-11-14): a rewritten text does not work over several cuts, because the second summary is a summary from a summary. A separate entry for each scene makes more sense.

**The parts.**

| Part | What it holds | Shown | Leaves the prompt | Step |
|---|---|---|---|---|
| The opening | The opening Narration of the Game, word for word | Always, at the head of the story memory | Never | 1 |
| Scene entries | One entry for each scene: the story time, the place, who took part, what happened, and the range of its Messages | All of them, before the history, while they fit their space | The oldest first, unless a person keeps it | 1 |
| The memory view | Every scene entry, also the ones that left the prompt. A person can read, edit and keep an entry | In the dashboard | | 2 |
| Character memory | What a Character did, was told and learned, also while she was absent | With her card | Not with the scenes | 3 or 4 |
| World facts | What is true: a number, a date, a name, where an object is. Lines with a story time, under a subject | When its subject is named | Not with the scenes | 3 or 4 |

Removal by age is the right rule for events and the wrong rule for the other two lists. An old scene matters less with time. A fact, or what a Character carries, does not. Most of the seven facts that had left the history in the pilot stories were facts of the world: the debt of 740,000, the date that Bernard stopped paying, the open desk, the revolver ([the inconsistencies of two trial stories](../../layout-trials/assets/story-inconsistencies.md), section 1).

A world fact is a lore entry that belongs to one Game, where the lore entries of an author belong to the World. [Decide how a Character or a lore entry enters the prompt](12-prompt-entry-modes.md) decides how such an entry loads.

**The four rules that keep the three lists simple.** The owner's concern was that the lists overlap and that the whole becomes hard to reason about.

1. **One source.** Every entry of every list is written from the Messages of one scene, at the history cut, and it names that scene.
2. **One job for each list.** A scene entry says what happened. A world fact says what is true. A Character memory says what she carries.
3. **No text is rewritten, in any list.** A changed fact is a new line with its story time under the same subject: "locked (day 1)", then "Julian opened it (day 2)". Both values stay, and the engine needs no update logic.
4. **The same handling.** Each list is tied to the Swipe, keeps its previous value when a writer fails, and has the same controls in the memory view.

The same event can appear in two lists. That costs tokens, but both lines come from the same Messages.

**Who decided what.**

- The owner: a rewritten text is out, and scene entries are in. The starting scenario is the most important thing for framing. The oldest entry is removed, and a memory view lets a person keep or edit an entry. The Character memory is still a separate part. The owner is not sure that the world facts are needed, and was not sure about a load of old entries by a key.
- The coordinator, stated to the owner before the owner's go: the opening stays word for word, and is not summarized. "Removed" means out of the prompt, with the entry still stored, so that the memory view can bring it back. The history cut stays at a token count ([Decide the size and the point of the history cut](08-history-cut.md)), so a cut inside a scene gives two entries. The writer writes who took part. No old scene entry loads by a key. A world fact is a list of dated lines. The four rules. The order of the work.

**The opening (known at `a9801e4a`).** `opening_message` (`src/domain/model/utils/opening.rs`) makes the first Message of a Game from the scenario text of the World. The prompt assembler holds no other copy of that text. So the opening leaves at the first history cut, and after that the narrator has only what the summary kept of it. The text is 93 words in Redmist and 151 words in the test World.

**What stays of the base, and what goes** ([Build the base story memory](14-build-base-story-memory.md)).

- Stays: the writer call at a history cut, before the narrator call. The marker of the newest Message that the memory covers. The tie to the Swipe. The rule that a failed call keeps the previous value and that its Messages go to the next call. The block before the history. The step and its status line. The story memory in the trial pack.
- Goes: the prompt that rewrites one text, the one text, and its limit of 2,000 tokens as the limit of the whole memory.
- Open: the storage. The base copies its one text into every Swipe state. A list that grows cannot be copied in that way.

**The order of the work.**

1. The opening and the scene entries, with the removal of the oldest. The tickets: [Decide the form of the scene entries](16-scene-entries.md), then where they attach in the engine, then the build.
2. The memory view.
3. The Character memory or the world facts. The memory test of step 1 chooses: lost exact values point to the world facts, and lost "who learned what" points to the Character memory.

**Left out, and why.**

- **A rewritten "story so far" for the oldest part** (a coordinator proposal). The owner rejected it: it is what the base has.
- **A load of an old scene entry by a key.** Nobody has measured such a load, and a scene has no natural key. The range of Messages on each entry keeps it possible.
- **One shorter entry for a group of old scenes**, written once. It keeps the rules. It is built only if a trial shows that the narrator needs the far past.
- **A search of the original Messages, with no writer.** A Message has no time and no cast. A search by meaning needs a second kind of model, and the engine has no code for one (known at `a9801e4a`).
- **A list of facts that a writer updates in place.** This is the least solved problem in the research: every method scored at most 28 % on a changed fact that needs two steps ([LLM memory research](../assets/llm-memory-research.md), section 2.6). The dated lines of rule 3 avoid the update.
- **A narrator that looks things up with a tool.** It needs several calls for each Narration. It is not known whether `deepseek-v4.1-flash` uses tools well.
- **Open threads.** Promises, plans and appointments that the story has not closed. No part holds them yet (map, Not yet specified).

**Marinara Engine, for comparison** (its user guide `docs/agents/memory.md` at `457e827a`, and [the asset](../assets/marinara-memory.md), section 7.2). Advanced Memory keeps one summary for each scene that left the live history, always in the prompt. When these pass 70 % of the memory budget, a background call combines and shortens them, which is a rewrite. It also shows at most 3 old scenes for each reply, each with 3 to 10 of its original Messages, chosen by a text match. It has no measure of that choice. This design takes the scene entry and the list of who took part. It does not take the combine step or the choice by a text match.

**Not known.**

- Whether the writer finds the scene ends in the Messages that leave. Marinara Engine records faults here.
- The size of a scene entry, and so the number of Cycles that the list covers. A guess: 300 to 500 Cycles at the default sizes, from an entry of 150 to 250 tokens.
- Whether the narrator misses the far past after the oldest entries leave.
- Whether a model can tell a durable fact from a passing detail. No tool and no paper of the research measured it. The world facts are the part with the most risk.
- How to score a Character memory. The memory test cannot score what a Character does not know.
- Whether one writer call can write two or three lists. Marinara Engine does it in one call, and its code has patches for mixed-up answers.

**What this ticket asked for and did not get.** A measurement over more cuts, the rule of the engine in the measurement, more than one run, and the cost. All four are now requirements of [Decide the form of the scene entries](16-scene-entries.md). The evidence of the size problem stays in [Evidence: the size of the story memory](../assets/story-memory-size.md).
