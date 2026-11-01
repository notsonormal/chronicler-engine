# Decide how to read an output trial

Type: grilling
Status: open
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
