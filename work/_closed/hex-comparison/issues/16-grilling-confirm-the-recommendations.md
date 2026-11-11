# Confirm the recommendations of tickets 10 to 15 and 20

Type: grilling (HITL)
Status: resolved
Blocked by: 10, 11, 12, 13, 14, 15, 20

## Question

Does the owner accept the recommendation of each of tickets 10 to 15 and 20?

## Context

- **The shape** (owner, 2026-11-01). Each AFK ticket measures its practice and ends with one recommendation and at most 2 questions. This session asks only where the evidence does not settle a recommendation. It asks at most 3 questions in each round, and it gives the evidence with each question.
- **Inputs.** The `## Answer` and the asset of each ticket:
  - [Research how agents use the current docs and plans](./10-research-agent-use-of-docs-and-plans.md): its recommendation for C-42, C-43 and C-44. Its questions on the doc practices went to ticket 20.
  - [Measure an env and clock ban, and the two hook traits](./11-task-measure-env-clock-ban-and-hook-traits.md): C-30, C-31
  - [Find where evidence of a working feature lands today](./12-research-where-evidence-lands.md): C-12, C-13, C-14
  - [Draft the work report](./13-task-draft-the-work-report.md): C-20
  - [Measure what docs-only gate runs cost](./14-research-docs-only-gate-cost.md): C-21
  - [Build one golden-image test and run it five times](./15-task-golden-image-test.md): C-15
  - [Recommend a decision for each doc practice](./20-task-recommend-the-doc-practices.md): C-01, C-02, C-03, C-09, C-10, C-11, C-32. Ask its questions 1 (C-09) and 2 (C-02). Its question 3 is answered: the owner put the doc practices in this session (owner, 2026-11-01). Its evidence includes [Research whether the docs mislead the agents that read them](./19-research-do-docs-mislead.md).
- **Owner input for C-12, C-13 and C-14** (owner, 2026-11-01, after ticket 12). A run with a real provider costs money and is slower, and no evidence shows that it finds more bugs. When a trial or a manual check finds a fault, the right fix is to find why the tests did not catch it, and to add the test. A record that a run passed adds nothing after that. Autoplay and `chronicler-ui-investigator` stay available for a manual check when one is needed. So:
  - **C-12 (a dated record of what someone saw work): dropped.** Do not ask about it again. Record the drop and this reason in the Answer.
  - **C-13 (proof limits): keep.** The limit lines tell an agent where the tests do not reach, so it knows when a manual check is worth its cost. Confirm only the form (inside the spec or in a file next to it, ticket 12 question 1).
  - **C-14 (known gaps): keep.** A gap stays in the spec until a test covers it.
  - Ticket 12 question 2 is answered by the C-12 drop.
  - **Evidence for the reason (known).** The prompt-flexibility trials found two faults, a `HashMap` card order and a failure text that hid `content_filter` ([Shape the test setup and the trial](../../../wayfinder-maps/prompt-flexibility/issues/08-test-setup-and-trial.md)). [Fix the prompt text](../../../wayfinder-maps/prompt-flexibility/issues/19-fix-the-prompt-text.md) then pinned the order with CASE ids such as `narrator-cards-in-id-order`.
- **For each accepted recommendation.** If the change takes one session or less, create a follow-on task ticket that builds it (map Notes, "Planning, with one exception"). If it is larger, write a brief, or a question for a new map or an existing one. A rejected practice is a drop with a reason.
- **Skills.** `/grilling` and `/domain-modeling`.
- **When done.** Add `## Answer` with the result for each practice, and set `Status: resolved`.

## Answer

Closed by the owner on 2026-11-01, with no grilling session. No recommendation of tickets 10 to 15 and 20 is adopted.

- C-12 is dropped, for the owner's reason in the Context above.
- The other recommendations are not adopted. For each one, no problem that someone has seen supports it. The owner judged the findings to be already covered, trivial or not worth doing.
- The end state of each practice, with its reason, is in part 4 of [the conclusion](../assets/conclusion.md). The small facts found on the way, such as the three date tests, are in part 5. Each one can be fixed at any time.
