# Task: design the game-state lifecycle from a behaviour spec

You are a software designer. After this task text comes a behaviour spec. It describes one part of an interactive fiction engine that tells a story with LLM calls: the game-state lifecycle. Design that part from the spec alone.

## What you have

- The spec below, and nothing else. You have no tools, no files and no other context.
- Some terms of the spec, for example "Swipe", also occur in other chat products. Do not copy the design of another product. If you take an idea from one, name the product and the idea in your Assumptions.
- You cannot ask questions during this task. Write them down in your output (section 12).

## What to design

Write a design that meets every requirement of the spec.

- Write no code and no programming-language types. Describe records, states, rules and sequences in words, lists and tables. A state list or a numbered sequence in plain text is fine.
- Do not design the black boxes of spec §3. Use them through their contracts.
- Do not add features that the spec does not ask for. If you think that one is necessary, put it in your Questions.
- If two requirements disagree, say so. Do not drop either one without a word.
- For each open question of spec §6 (OQ-nn), choose an answer and record it in your Assumptions with your reason, or leave it open and put it in your Questions. Do not leave one out.
- Do not ask about a point that the spec already states.

## Output

Write one Markdown document with these sections, in this order.

1. **Summary.** The main ideas of the design, in at most 300 words.
2. **What the design keeps.** Per Game, per Message, per Swipe, and any other record. For each item: what it holds, who writes it, and when it changes.
3. **States.** The states of a Game, and the states of one piece of engine work from its request to its end. For each state: what it holds. For each transition: the request, the black-box result, the failure or the restart that causes it.
4. **Swipes.** What binds to a Swipe, and what a swipe switch gives back.
5. **Retry and Retrigger.** For each kind of last generation in spec §4.8, and for Retrigger:
   - which part of the story the narrator reads;
   - which game state the redo starts from;
   - which steps run again;
   - what the request adds to the story;
   - what a failure leaves.

   Also state what a Retrigger can do after each kind of Retry.
6. **Writers.** For each request in spec §2, and for each piece of background work:
   - may it run while the Game is busy;
   - what it does to work that runs;
   - which Game its writes go to;
   - how the design prevents a lost update.
7. **Notices.** Where each kind of notice goes, who sees it, how long it stays, and how the design keeps it out of every LLM prompt.
8. **The room report.** Where the room that a generation arrives in is kept, and how the story log gets it.
9. **Restart and cut-off work.** What a restart finds, and what it does.
10. **Requirement trace.** A table with one row for each requirement R-nn: the section of your design that meets it, and "forced" (the requirement leaves you one choice) or "free" (you chose among two or more options that meet it).
11. **Assumptions.** Each point where the spec is silent or unclear and you made a choice. Include each OQ-nn that you answered, with your reason.
12. **Questions for the owner.** For each question: why it matters, and the answer that you would take if nobody answers. Include each OQ-nn that you did not answer.

Make the design as long as it must be. A length of 4,000 to 8,000 words is a good target.

The spec follows.

---
