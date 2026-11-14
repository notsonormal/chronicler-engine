# Brief for ticket 05: choosing the memory candidates

Evidence only, no decision. `MM` = `assets/marinara-memory.md`, `RES` = `assets/llm-memory-research.md`, `BF` = `assets/baseline-faults.md`, `MT` = `assets/memory-test.md`, `CP` = `assets/card-probe.md`, `PF` = `../../prompt-flexibility/assets/long-term-memory.md`. Claims are marked **known**, **inferred** or **guessed**.

## 1. What the evidence says a memory must fix

**None of the confirmed faults is the kind a memory of dropped entries fixes.**

- A: 0 of kind 1 (a fact that left the history), 4 clock faults, 5 who-knows-what. B: 0, 1, 3. Raw counts were 19 and 16 for A; 9 confirmed. (`BF` "Summary")
- "A memory of the dropped entries fixes none of the confirmed faults." Several come before the first cut: A-3.1, A-2.1 and A-2.2 are before #611; B-2.1 comes 13 entries after the cut with its anchors held. (`BF` "Observations")
- The asset's guessed fixes: a story clock; a record of who was present at each talk (A-3.1, A-3.2, A-3.5, B-3.1, B-3.2); a rule that keeps card facts from a Character who has not learned them (A-3.3, A-3.4, B-3.3). (`BF` "Summary")
- All five of A's who-knows faults happen while the text is still in the history. A-3.5: Tomas uses "Ardo Voss", yet "#676 was in the history (oldest #615), so this is not kind 1". (`BF` "Kind 3")

**What the test can and cannot measure.**

- It scores recall, not use in play. (`MT` "Limits")
- 10 who-knows-what questions, 4 kept; the 6 dropped are the "No" answers: "F11, F15, F28, F38, W31, W35: every 'No' question". So the "does not know" side is untested: with no memory the model says "No" to every access question. (`MT` "The question set", "How it works" step 5)
- **A "who was present" record cannot be scored directly.** The assets do not say that any question asks who was present at an entry. The nearest kept rows are who did a small thing (F22, F24), a reason (F19) and the four "Yes" access questions (F12, F14, F23, F36). (`BF` "The fact list")
- **A record about entries still in the history cannot be scored at all**: "A question is kept only when the no-memory control answers it wrong in every run". The draft's target, the 5 who-knows faults inside the history, is therefore outside the test. (`MT` step 5; `issues/05` Context) **inferred.**

## 2. The open questions

### 2.1 The trigger

The options: the history cut; a scene end; each Cycle; a mix.

- Cut, **known**: the rule is a size with a 75 % block, and "No logical point (a scene end) now. [05] can add one later"; `HistoryCut` "fires no hook yet". (`issues/08-history-cut.md` Answer Q1; map Notes)
- Cut, for and against, **inferred**: the writer sees exactly what the next call loses, and the cut already breaks the prefix; but cuts are 11 to 12 Cycles apart at 16,000 tokens, so the memory is stale. (`issues/09` Answer)
- Scene end, **known**: ME resets the live history "at the start of the latest scene. A size rule decides when. A model decides where." `MM` §9.1 item 2 adds "**inferred** The ME rule needs a scene boundary that our engine does not record." Against, **inferred**: it writes on cycles that drop nothing, so the test's ask points no longer line up. (`MT` "How it works" step 1)
- Each Cycle, **known**: most designs write after each turn (Mem0, Zep, Re3); a few at scene end. (`RES` "For ticket 05" item 6) That is about 100 writer calls against 5 or 6, and "A memory whose text changes on every cycle belongs after the history." (`PF` §5)
- A mix is ME: eight writers on different triggers, but two writer costs, and no Agent reads another's output. (`MM` §1.1)

### 2.2 The position, and its cache cost

`before_history` (end of the user message before the history) or `after_history` (a user message after it). (`MT` "How it works" step 2)

- **known**: before the history, a rewrite "changes the prefix on the cycle that rewrites it only"; after it, "the prefix survives, and only the tail differs. This is the cheap position for text that changes often". ST Summarize, ME `chat_summary` and AI Dungeon sit before; ST Author's Note and ME `recalled_scenes` after. (`PF` §5)
- **inferred**: with a cut trigger the text changes 5 or 6 times in 100 Cycles, so before costs 5 or 6 breaks and after none.
- **known**: the measured candidate used `before_history` and cost "about 733,000" prompt tokens, "60 to 90 %" cached. (`MT` "The result")
- **The assets do not say** the cache cost of each position, since no arm differs only in position; nor how large a block pushes the history space under 16,000 in the 32,768 probe window. (`issues/05` Context)

### 2.3 Does a Character memory change while the Character is absent?

The options: only while present; yes, with what happens to it off stage; present-only perception plus a second record for what happens to it.

- The owner's working view is the second: "when characters leave a story they will keep to have what happened to them updated." (map Notes)
- **known**: ME's Advanced Memory is present-only: "Never assign an absent character just because the scene is about them." ME is split: the Relationship Tracker ("Do not gate tracking from scene presence") and LTM update by name. (`MM` §3.2; `issues/01` Answer)
- **known**: the literature does not answer it: "I found no source that tests this." (`RES` "For ticket 05" item 7; `issues/02` Answer)
- Against present-only, **known**: "The Quantifier drops a Character who is on stage, and keeps a Character who left", with "14 to 20 runs of a wrong State line in one story". (`issues/05` Context)
- Against off-stage: it needs "a writer that reads the whole story and names the Character", and ME has no measure of them. (`MM` §9.1 item 6) For it: A-3.1, where Carla was at the gate. (`BF` "Kind 3")
- **inferred**: the test cannot choose between them; no question asks what a Character learned off stage.

### 2.4 What a Character memory holds

The options: all it saw and heard; only what happened to it; what it could perceive plus what the story tells it outside the scene. The draft says the third. (`issues/05` Context)

- **known**: ME's recollection is the first in form, the third in rule: 50–100 words, first person, "only the messages they could know". (`MM` §3.1, §9.1 item 5)
- **known**: the one measured fix for "knows too much" is a filtered view built outside the model: it "drops each inaccessible event and replaces it with what the character can still observe". The leak fell "from 60.7 to 0 points", permitted use "+0.6", on short template scenes; prompted self-filtering cut the leak but "dropped permitted content". (`RES` "Who knows what")
- Against a narrow memory: the 4 kept "Yes" rows are what it loses (F12, F14, F23, F36). **inferred**: a memory that is too wide is invisible to the test, because the "No" side is not scored; only the trial's kind 3 sees it. (`MT` "The question set"; `BF` "The judge prompt")

### 2.5 One writer call or two

The options: one call for both; one call per reader; one for the story memory, one for all Character memories.

- **known**: ME is the first. "One call per closed scene writes the recap, the audience ... and ... a recollection for each Character present." It splits "when cards would force more parts than the recap needs". (`issues/01` Answer; `MM` §9.1 item 4)
- **known**: the tool writes "one text with one writer call per schedule step"; "Keep the result one text for the `<StoryMemory>` block". One call per cut cost "5 writer calls ... and about 90,000 prompt tokens". (`MT`) **inferred**: two calls per cut about doubles that; one per reader multiplies by the cast.

### 2.6 Does a record of who was present at each entry belong in the candidates?

The options: no record, so the cards and the rule text must hold; a record of the dropped entries only; a record of every entry, marked where the history still holds it (the draft). (`issues/05` Context)

- The dropped-entries record cannot fix them: 5 of A's 9 faults are who-knows-what and all are inside the history. **known**: a held text does not stop the fault: A-3.5 and B-3.1 both reuse one. (`BF` "Kind 3")
- No record is refuted on this model: the rule text "has no measured effect" and the label "does not hold"; Julian says "you've been in this house near a decade" "in all 12 runs, in every arm". (`CP` "Summary") It is also unsafe: with every card on, Carla gives an absent Gabriella's secret past "in 8 of the 9 runs of arms 2 to 4". **inferred**: more who-knows text is not safe merely because it is labelled.
- The full record cannot be scored by the test (section 1), only by the trial's kind 3; **inferred**: it also costs tokens on every call, which can move the cut.

## 3. The four draft candidates

The bar is 67 % (57 to 77 over 3 runs): one 600-word summary of all dropped entries, `before_history`; under about 10 points is noise. (`MT` "The result"; `issues/04` Answer)

**What the baseline lost.** Its writer prompt asked for "who was present at each conversation, and who learned which fact", and the 500-word memory did not keep it. The answers lost F19 a reason, F22 who took Campion's call, F24 who readied the sitting room, F27 where the codicil is kept, F34 Louise's dish, and F23 and F36 who learned what. (`MT` "What the simple candidate lost") A prompt-only change to the same summary is already refuted.

**C1, a rolling summary at each cut.** The measured 67 % used `once`, not `each_cut`. (`candidates/one_summary.json`) For: rewriting a short memory each step is published and preferred by raters (RecurrentGPT). Against: summaries lose detail (LoCoMo "gave no gain although recall was high"), and ME's Chat Summary is append-only ("Do not rewrite or repeat the previous summary") for that reason. (`RES` "Stories", "Open problems" item 4; `MM` §1.1) Cost: 5 writer calls per run. **guessed**: inside the noise of 67 %, with silent loss at each rewrite.

**C2, scene recaps that name who took part.** The ME Advanced Memory shape. (`MM` §9.1 item 1) For: ME writes the recap and the audience in one call, so presence is a written field, and the baseline shows that asking for it in prose is not enough. Against: "The ME rule needs a scene boundary that our engine does not record" (`MM` §9.1 item 2, inferred), plus a check call every 5 Messages and a new schedule in the tool. The assets do not say any measured quality for an ME memory. Risk: scene ends do not line up with cuts, so ask points and cost change.

**C3, C2 plus a short section for each Character.** For: the filtered view is the only family built for who knows what, and its one result is large (60.7 to 0 points); ME's per-Character record is 50–100 words. (`RES` "Who knows what"; `MM` §3.1) Against: the 4 kept "Yes" questions cannot show the gain and the 6 "No" ones are dropped; the tool needs per-reader output; and a labelled text leaked when Carla was asked about an absent Character. (`MT` "The question set"; `CP` "Summary") Cost: the cast times 50–100 words on every call. **guessed**: no better than C2 on the 35 kept questions, better on kind 3 in a trial.

**C4, BM25 over the dropped entries, no writer.** For: a plain-file search scored 74.0 % on LoCoMo, and finer retrieval units beat summaries (fact-expanded keys gave +9.4 recall points). Cost: "embeddings only, no completion call" for that family. (`RES` "For ticket 05" items 1 and 4) Against: a retrieved set changes on almost every Cycle, so it must sit late or break the prefix (`PF` §5 rule 3); the tool has no retrieval schedule; and retrieval can miss a fact ("in about 10% of right answers retrieval had found only the new value"). (`MT`) **guessed**: competitive and cheap, with high variance per question.

**Strong candidates the draft misses.** First, no memory at all at the default size. **known**: a 100-Cycle story is about 41K tokens and `max_history_tokens` defaults to 65,536, "so a 100-Cycle story fits whole", first cut at about Cycle 155; full context beats every memory system when it fits (72.9 against 66.9), and the whole-story control scores 95 % (94 to 97) against the summary's 67 %. (`issues/08` Answer Q4; `RES` §2.0; `MT` "The result") The 16,000-token setting is what forces cuts. Second, the filtered view of the prompt, the only measured fix for who knows what: it is not a `<StoryMemory>` text and needs its own arm. (`RES` "Who knows what")

## 4. How a candidate plugs into the memory test

A candidate is one JSON file: `name`, `schedule` (`once` or `each_cut`), `position` (`before_history` or `after_history`), `writer_system`, `writer_user` (with `{entries}`, `{previous}`, `{first}`, `{last}`) and `max_tokens` (default 2048). The tool puts the written text in a `<StoryMemory>` block in the stored Narration request of each ask point and changes nothing else. One run of 42 questions costs about 733,000 prompt tokens and 50 calls for the summary candidate, against 642,000 for no memory. Three runs are needed, because temperature 0 is not stable; a gap under about 10 points is noise. Per-Character or presence-list memories need new code in `Memories.for_call`. (`MT`)

## 5. Contradictions and gaps

1. **The cut figures come from two packs.** The ticket says "5 or 6 cuts ... the first at about Cycle 41" and cites `tmp/ltm09/redmist_julian/trial-1`. (`issues/05` Context; `issues/09` Answer) The test's cut points come from story A (`tmp/lt15/new/trial-1`), which has **5** cuts, the first at #611, dropping 25, 20, 22, 24 and 23 entries. (`MT` "The result"; `BF` "The fact list") So a candidate writes 5 memories.
2. **C2 and C3 do not run in the tool as it stands.** The ticket says the test "changes only a `<StoryMemory>` block". True of the request, not of the tool: such memories "need a new schedule in `Memories.for_call`". (`MT` "How to add a candidate")
3. **The draft's record for held entries cannot be scored by the test**, which keeps only questions the history cannot answer. (`MT` step 5) The map may intend this, but a test-led choice cannot see the draft's main new part.
4. **The map's check is insensitive to what the test scores.** Each checked count is 5 or less with kind 1 at 0, so one judge run per side "cannot show a change of any one kind on its own". (`BF` "Summary") **inferred**: a candidate can win the test and move no number.
5. **Gaps, not contradictions:** every measured number comes from one story, one model and 3 runs (`MT` "Limits"), and no asset says how large a memory block must be before it moves the cut at 32,768.
