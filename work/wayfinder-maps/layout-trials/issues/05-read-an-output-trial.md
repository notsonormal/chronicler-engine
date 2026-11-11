# Decide how to read an output trial

Type: grilling
Status: resolved
Blocked by: 01

## Question

How does the owner read a comparison of two output trials, and what is enough to keep a candidate?

## Context

- The output candidates change what a reader sees: [Try `example_dialogue` in the prompt](06-example-dialogue.md), [Try the history as chat messages](07-history-as-chat-messages.md) and [Try the role of the Impersonate prompt](08-impersonate-prompt-role.md). The cache candidates do not need this ticket.
- The owner: "better or worse is hard to determine anyway". This ticket finds a reading that the owner can do and trust. Real packs are the material, not criteria written in advance (map Notes).
- Material: the pack of [Run the baseline trial](01-baseline-trial.md), and the older packs `tmp/pf08/` (trials A and B) and `tmp/pf20/redmist_julian/trial-1`. A 100-Cycle story has about 200 entries. In the last trial, a Narration had a median of 175 words and an Input 126. So one trial is about 30,000 words (inferred).
- [The review of the Impersonate output](../../prompt-flexibility/assets/impersonate-output-review.md) is one earlier reading. It looked at the length against "one beat", the repetition between Inputs, the scripting of NPCs and the perception limits.
- The questions to settle, one round at a time:
  - Which parts of a story to read: all of it, fixed windows (for example the first, a middle and the last Cycles), or only the part that the candidate targets.
  - How many Cycles a trial plays, and how many trials each side needs. [Shape the test setup and the trial](../../prompt-flexibility/issues/08-test-setup-and-trial.md) said 2 trials for each side when comparing. Some faults show only in long trials, for example repetition and voice drift over the Cycles.
  - What is enough to keep a candidate.
  - Whether the role candidates (the history as chat messages, and the role of the Impersonate prompt) also run on a second model.
  - Where the criteria are written. Proposed: one asset of this map, `assets/reading-criteria.md`, which each output reading extends.
  - Whether the script can help the reading, for example by putting the targeted part of two trials side by side. A change to `scripts/trial.py` is allowed.

Done when the answer states the method and the first criteria are in the asset.

## Progress

- **2026-11-02, change of direction (owner, grilling).** A person cannot compare two trials entry by entry, because the stories diverge after 1 or 2 entries (the model samples, and no preset sets `temperature` or a `seed`). Reading 90,000 words with no clear question is not a task the owner can do. The owner: an LLM can judge one story by itself (is it consistent, does it follow the story, does it understand the Characters), and that is valuable. Several runs for each side are needed against the noise, but that is not important yet. First step: try a judge on two trials.
- **Pilot of the judge.** Two trials that already exist, `tmp/lt01` (baseline) and `tmp/lt03` (changing blocks after the history, a cache change only), so a near A/A pair: a good judge gives them about the same scores. Story-only copies, neutral names P and Q, the key in `tmp/lt05/key.txt`. One `reviewer` judge reads one whole story, with no knowledge of the other, and scores 5 dimensions (consistency, follows the story, the Characters, the Persona Inputs, repetition) from 1 to 5 with quoted findings, plus an overall score from 1 to 10. Verdicts: `tmp/lt05/judge-P.md`, `tmp/lt05/judge-Q.md`.
- **Result of the pilot (2026-11-02).** Both judges gave the same scores (consistency 2, follows the story 3, the Characters 3, the Persona Inputs 2, repetition 3, overall 6 of 10). Every finding that was checked against the story and the data was real. The findings are worth more than the scores. The breakdown of the inconsistencies and their causes: [asset](../assets/story-inconsistencies.md). About half come from the history cut, the missing story clock and who knows what (Long-term memory and state), and some from our own data.
- **The judge is ad hoc (owner, 2026-11-02).** No predefined judge: each check asks a different question, so each trial writes its own judge prompt for the question of its candidate.
- **Data fixes (owner, 2026-11-02).** The father/grandfather conflict is fixed in `data/personas/julian.json` and in the probe DB. The other data conflicts go to [Remove the data conflicts that distort a trial](12-remove-data-conflicts.md), which the output trials now wait for.

## Answer

- **No person reads two stories side by side.** Two trials diverge after 1 or 2 entries, so an entry-by-entry reading gives the owner no question to answer (owner, 2026-11-02).
- **The method: an ad hoc LLM judge.** One judge reads one whole story, alone, with a prompt written for the question of that check. Both sides get the same prompt and neutral names, and the story copy keeps the history-cut line. The coordinator checks a sample of the findings against the story and the data before the owner reads them. The owner reads the findings, not the stories, and decides. There is no predefined judge, because each check asks a different question (owner). The prompt of the pilot is an example, not a standard: [judge prompt of the pilot](../assets/judge-prompt-pilot.md).
- **The first criteria are the fault types the pilot found**, with their causes: [the inconsistencies of two trial stories](../assets/story-inconsistencies.md). A later check picks from that list or adds to it.
- **Evidence for the method.** On the near A/A pair (`tmp/lt01`, `tmp/lt03`), the two judges gave the same scores on all six lines, and every finding that was checked was real. The findings are worth more than the scores. The pilot does not show that a judge can tell a better story from a worse one; the method relies on concrete findings and counts, not on scores.
- **The model (owner, 2026-11-02).** `deepseek-v4.1-flash` is a popular model for roleplay, so it is taken as good enough. No second model and no model check. An inconsistency that is inherent to the model is accepted, not engineered around. The work is to separate the inherent ones from those that our structure or our data cause: [Sort the inconsistencies by their cause](13-sort-inconsistencies-by-cause.md).
- **The output candidates (owner, 2026-11-02).** The pilot shows that the layout is not the main lever for story quality: about half of the faults come from the history cut, the missing story clock and who knows what, and some from our data. [Try `example_dialogue` in the prompt](06-example-dialogue.md) and [Try the role of the Impersonate prompt](08-impersonate-prompt-role.md) have no measured fault behind them and are out of scope. [Try the history as chat messages](07-history-as-chat-messages.md) targets the one layout fault the pilot found, the role breaches, and a cheaper test replaces it: [Stop the role breaches with stop sequences](14-stop-role-breaches.md).
- **Trials and the noise.** Several runs for each side would be needed against the noise, but this is not important now (owner). A check starts with one trial.
- **Data first (owner).** Conflicts in our data distort every trial: [Remove the data conflicts that distort a trial](12-remove-data-conflicts.md). The father/grandfather conflict is already fixed.
