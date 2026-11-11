# Owner friction in lead sessions, 2026-10-28 to 2026-11-04

Source: `followup_user_turns.json` (399 owner turns after the first), `correction_candidates.json`, `sessions_with_effort.json`, `user_messages.jsonl`, and the raw session file where a quote needed its cause.

Known: `correction_candidates.json` holds 81 entries, not the 111 in the brief. Known: the `skill` field records `/writing-for-agents`, typed by the owner, as `technical-writing` (2026-10-30T21-21), so one row below may carry the wrong name. Inferred: a turn that needs a one-word answer is still a full round trip.

## Which skills the owner named

| Skill (message `skill` field) | Invocations |
|---|---|
| `wayfinder` | 51 |
| `improve-ai-plan` | 10 |
| `sharpen` | 9 |
| `understanding-ladder` | 4 |
| `code-review` | 4 |
| `document-review`, `grill-with-docs` | 2 each |
| `technical-writing`, `writing-for-agents`, `chronicler-after-plan-workflow-plus-review`, `consider-the-bitter-lesson`, `retro` | 1 each |


`wayfinder` is the owner's default entry point: 44 of 255 sessions, 294 owner turns, $183 of $503.

## Patterns, ranked by cost to the owner

**1. A grilling round asks more questions than the owner can answer.** 45 numbered-answer turns (`Q1 -> A`) in 16 sessions; 78 of 399 follow-ups are 25 characters or fewer; one message asked 10 questions (`2026-11-01T13-22`). Known: `2026-11-01T13-22` "you just asked me a ton of questions, so I'm not really confident in my answeres"; `2026-10-31T17-28` "For question nine, I am not sure what the question is."; `2026-10-31T17-44` "A. What are the other questions?". Cost: the owner decides while unsure, and a wrong decision costs a build. Change: `grilling/SKILL.md` caps a round at three decisions, each with one line of context and one recommendation. Matches: **adds evidence to ticket 88**, and names Won't do line **10 — Grilling questions cite opaque ids**. The volume is the new evidence; the recorded rule covers wording only.

**2. A research-then-grilling map spends its budget and drops every finding.** 16 sessions on `hex-comparison`, $11.40, no code. Known: `2026-11-01T19-59` "it basically brought up a huge list of findings and tend determined eventually that they were all 'already covered' or 'not worth doing'"; `2026-11-01T13-22` "there are a lot of grilling tickets. Not a lot of task or other AFK tickets". Cost: owner attention and model spend with no artifact. Change: the map Note names the evidence that accepts the destination, and a research ticket must return one decision the owner can act on. No open ticket; not in Won't do.

**3. The branch close-out is re-typed four times, and the merge is a fast-forward.** `2026-10-30T22-35`, `2026-10-31T12-35`, `2026-11-01T19-55`, `2026-11-03T21-46`, each "merge the current branch into main, switch to main and delete the branch". Three answered with a fast-forward and no merge commit; `2026-10-30T22-35` drew "???? do you not know what merge is?". `2026-11-03T21-44` shows the owner doing it himself. Cost: 4 re-types, 3 corrections, a `push` follow-up each time. Change: one `AGENTS.md` line for the close-out: `git merge --no-ff`, switch to `main`, delete the local branch, push `main`, delete the remote branch. Not covered. Won't do **61** is map close-out, a different operation.

**4. The owner fact-checks a claim the agent stated as known.** `2026-11-01T13-51` "Was 2048 the actual value we set for max tokens?" — the agent's "the same as before the ticket" was wrong for one Connection. `2026-10-31T16-52` "???? It should be using opencode go. Is it is using openrouter?" — the diagnosis named the wrong provider. `2026-10-31T15-46` "By the way, the message history has all the data, the llm_messages is just for like debugging" — a trial was read against a false premise. Cost: the owner holds the repo model instead of the agent. Change: a report rule — a "before ticket X" claim needs `git show X~1:<file>`, and a provider claim names the file it read. No ticket.

**5. The agent's report uses a term the owner must ask about.** 10 instances. Known: `2026-10-30T21-21` "`Drop the full gate from the inner loop; use --no-browser there` -> What do you mean?"; `2026-11-01T22-09` "`the layout-trials map has no \"resolved\" entry for ticket 09 yet` -> What do you mean no resolved entry?"; `2026-11-02T22-47` "What does painted mean?". Cost: one round trip each. Change: a report rule — state the effect and the count first, name an identifier after it. Adjacent to ticket 88, but the subject is report prose, not questions.

**6. The owner must ask for the verdict.** 4 instances: `2026-11-01T19-35` "Is there any benefit in A over B?"; `2026-10-31T21-09` "So, do you think Impeccable is good to use?"; `2026-11-01T13-22` "this kind of framing doesn't really help me understand whether I should use feature maps at all". Cost: the owner makes a comparison the agent had the facts for. Change: the question shape carries a `Recommended:` line, the convention the Plannotator Inbox uses. Matches: part of ticket 88.

**7. The owner must point the agent at the reference engines before it will check them.** `2026-10-31T16-42` "Q1 -> How does ST/ME handle it?" and "How does ST/ME handle these cases?" twice verbatim; `2026-10-31T17-28` "Can you check what the Marina engine does with stuff like this?"; `2026-11-02T19-47` "Check https://github.com/Pasta-Devs/Marinara-Engine to be sure" and "I'm a bit confused as I thought that I was modelling this all off ME." Cost: the owner supplies product knowledge that one fetch returns. Change: a `CONTEXT.md` entry that names SillyTavern and Marinara Engine with their URLs, and says to read their source before asking. No ticket.

**8. The owner cannot judge a visual artifact from text.** `2026-11-02T18-25` "Even if I read it, I'm not sure what you are expecting from me"; `2026-11-02T22-47` "visually you've got it wrong since she's female". 19 art sessions, $22, closed as a near no-op. `2026-11-04T17-40` gives the cause: "I am connecting to this machine via ssh … so when it tries to open up the plannator window I can't see it". Cost: each visual round is prose. Change: the Plannotator Inbox as the default hand-over for an image, plus a contact sheet of variants per round. Partly fixed this week.

**9. An example the owner gives becomes a hard constraint.** `2026-11-03T00-20` "I don't you took much stoke in my 'rules'. I guess that's why the art wasn't improving. So it would be better if you thought a bit more flexible" (sent twice); `2026-11-02T22-47` "I'm just saying how the other game does it. I'm not saying how we should do it in this case." Cost: several rounds with no movement. Change: a brief convention — an example is a reference, not a constraint. No ticket.

**10. The owner supplies a step the ticket template should carry.** `2026-11-01T13-27` "I notice there's no document review step in ticket twenty-four, so add that in."; `2026-10-31T23-19` "It sohuld end with a /simple-after-plan"; `2026-11-01T15-16` "And when done, can you archive any completed plans in docs/plans". Cost: the owner is the template's memory. Change: the wayfinder ticket templates carry the steps. Matches: decisions **57** and **80/72** say where the docs step goes, but no template carries it; **adds evidence to ticket 99**.

**11. The owner corrects the agent's model of who can run what.** `2026-11-01T22-09` "Why wasn't it able to run agents?" and "I don't understand, this is the top session?"; `2026-11-02T23-41` "I don't think subagents can call other subagents.". Cost: 4 rounds, and one brief reached a child that could not run its steps. Change: one line in the `AGENTS.md` delegation section — a child registers no `delegate` tool. Matches: **adds evidence to ticket 99**.

**12. A one-word answer costs a whole round trip.** 78 of 399 follow-ups are 25 characters or fewer: `yes` ×17, a bare letter ×8, `push`, `commit and push` ×4. `2026-10-31T15-46` "What is your question" — the owner asking the agent to restate the ask. Cost: about one fifth of the owner's turns. Change: when the default is safe, take it and state it as taken. Matches: same shape as ticket 88; report the volume as evidence.

## Trigger failures

- **`understanding-ladder` — the trigger fails.** `2026-11-01T19-57` the owner wrote "just the understanding ladder to explain what the 3 tickets found", a request the description should fire. The agent answered with ticket structure, and the owner corrected it at `2026-11-01T19-59`: "So I want to understand what you learned from all the tickes. Not the ticket structure." Change: the skill says that request is a ladder request.
- **`document-review` against `writing-for-agents` — the boundary is unclear.** `2026-11-01T22-09` the owner ran both on one doc, 2 turns apart: "can you check whether the change … makes sense. I feel like it's probably sediment", then "Can you review the whole document holistically". Inferred: the owner does not know which skill owns a doc change. Change: one line in each skill that names the other and its case.
- **`wayfinder` — the trigger works, the map state does not.** 51 named invocations, mostly the terse `.scratch/<map>/map.md work on ticket N`. `2026-11-01T13-51` "I'm a bit confused right now from this map. What exactly is the next step?" shows the cost is map state (pattern 2), not the trigger.
