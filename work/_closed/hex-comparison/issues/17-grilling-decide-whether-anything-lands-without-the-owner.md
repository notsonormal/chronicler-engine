# Decide whether Chronicler works in small changes

Type: grilling (HITL)
Status: resolved
Blocked by: 16, 21

## Question

Should Chronicler work in smaller changes? If yes, what keeps a change small, and what is in the way today?

## Context

- **Why the question changed** (owner, 2026-11-01). The first question of this ticket was "May any change land without the owner's approval?". The owner's aim for the map is to find what keeps Hex's code good while several developers build it fast with agents. The one clear measured difference is the size of a change. Landing without the owner is a later step that small changes make possible, so it is now a sub-question.
- **Input.** [Measure whether large changes need more fixes](./21-research-change-size-and-fixes.md): whether large Chronicler commits need more fixes per line, the same measure for Hex, and what makes the large Chronicler commits large. If the data shows no effect of size, say so first. The ticket can then end in "no change".
- **Hex (known).**
  - One issue gives one agent change, one PR and one squash commit. The issue form requires the behaviour and the steps to reproduce it (`assets/research-02-issue-pipeline.md` row 1).
  - Pipeline commits: median 80 changed lines, p90 253, largest 475 (row 11). All recent Hex commits: median 152, p90 1543.
  - `.github/agent-policy/check.ts` (118 lines, no AI) flags a diff of more than 600 lines or 25 files. The rule never fired (K-02). The verdicts were 46 `clean`, 17 `flagged` and 1 `dax` (K-05).
  - A second model reviews each diff before the merge. Its criteria are not public.
  - 60 of 64 agent PRs merged, with a median of 3 s from open to merge (K-04). 8 of 59 pipeline commits are `fix(ci)` repairs, and none was reverted (K-07).
  - `.hivemind.toml` verifies, builds and installs in batches of up to 4.
- **Chronicler (known).**
  - The median commit has 377.5 changed lines, and p90 is 3049 (K-41).
  - `AGENTS.md` says "Don't commit without explicit approval". The permission config allows `git commit`, so the written rule is the only barrier.
  - Work is planned in maps and tickets. A ticket is sized to one agent session, not to one small change.
- **The practices this ticket decides.** C-19 (the agent lands its own small changes), C-17 (a deterministic risk sort of a diff, which holds the size rule), C-18 (a second model as the strict reviewer) and C-50 (Labs). See `assets/candidates.md`, themes D and G. The rule from ticket 08 still holds: if C-19 drops, C-17 drops with it, and C-18 counts as covered by the `code-review` skill. C-50 joins only if C-19 stays.
- **Sub-question.** Only if small changes are adopted: may a small change that passes the gate and the review land without the owner? It needs a durable record of each change: the work report of [Draft the work report](./13-task-draft-the-work-report.md), as [Confirm the recommendations of tickets 10 to 15 and 20](./16-grilling-confirm-the-recommendations.md) confirms it.
- **Likely end.** A yes on small changes ends as a brief: how a ticket is cut into small changes, a size check, and the review of each change. If the brief is larger than one session, it leaves as a question for a new map. A no ends in "no change", with the data as the reason.
- **Skills.** `/grilling` and `/domain-modeling`.
- **When done.** Add `## Answer` and set `Status: resolved`.

## Answer

Closed by the owner on 2026-11-01, with no grilling session. The answer is "no change".

- [Measure whether large changes need more fixes](./21-research-change-size-and-fixes.md) found that repaired lines per changed line do not grow with commit size, in Chronicler or in Hex. Hex does not show a lower fix rate (22.0 against 17.8 repaired lines per 1000 changed lines).
- So small changes are not shown to keep code better, and C-17, C-18, C-19 and C-50 end as "no change". The rule "Don't commit without explicit approval" stays.
- Details are in part 3 of [the conclusion](../assets/conclusion.md).
