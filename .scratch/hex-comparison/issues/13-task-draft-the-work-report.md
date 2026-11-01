# Draft the work report

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

What does the work report hold, and where does it live?

## Context

- **Term.** The **work report** is the durable record that one agent session leaves when it finishes a change. It says which checks ran, which did not, and what the review verdict was. Do not call it a "run report": `CONTEXT.md` reserves "run" for an Autoplay run (owner, 2026-11-01).
- **Covers** C-20. See `assets/candidates.md`, theme D.
- **Hex (known).** An agent PR body is the report of one automatic session, and all 64 bodies use one template: `Fixes #N`, the issue form read back, `**Change**`, `**Gates**`, and `Run:` with the session id. 13 to 18 of the 64 bodies say openly that a part was not compiled or not run. Rows 2, 6 and 12 of `assets/research-02-issue-pipeline.md` have the detail.
- **Chronicler (known).** `logs/build_history.txt` records each `build.py` run, and a ticket's `## Answer` records results. The verdict of a review, and "what I did not run", stay in the session transcript.
- **Do.**
  1. List where parts of a report exist today: the `build_history.txt` row, the `logs/build_*.log` files, ticket answers, the output of the `code-review` skill, and commit bodies.
  2. Draft a template. Compare the candidate places in a table: the commit body, the ticket answer, and a file in `logs/` or `.scratch/`.
  3. Fill the template in for 3 recent changes. Use `git log`, `build_history.txt` and the session logs.
  4. Recommend a place and a template. Say whether the change is a follow-on task of one session or less.
- **What already exists.** Each delegate leaves `~/.pi/agent/delegate/<id>/result.json`, which holds its final message, and `job.json`, which holds its label and role. That is already part of a work report, but it is outside the repo. The `code-review` skill shows its two reports in the chat only. `scripts/build_history.py` names the columns of `logs/build_history.txt`.
- **Failure modes.** A session that ends with no commit and no ticket has no place for a report. Say where its report goes, or say that it needs none.
- **Not in scope.** Do not edit `AGENTS.md`, skills or `build.py`.
- **Output.** Write `assets/task-13-work-report.md`. The `## Answer` gives the gist, the recommendation and at most 2 questions for ticket 16. [Decide whether anything lands without the owner](./17-grilling-decide-whether-anything-lands-without-the-owner.md) builds on this answer.
- **When done.** Add `## Answer` and set `Status: resolved`.

## Answer

**Gist.** Parts of a work report exist today, but none is durable and whole. The checks are in `logs/build_history.txt`, which is git-ignored and keeps about 5 days (known). The review verdict stays in the chat (known). "What did not run" is nowhere (known). Hex's PR body holds all three.

**New finding (known).** The `tree` column of the history can be rebuilt for a commit after the commit. I matched 3 of 3 commits to their rows. So a report can say on which tree each check ran. This shows a gap in 2 of 3 recent changes: `integration` and `clippy` did not run on the committed tree of `b4684aea` and `1a0c9162`, and `py-tests` did not run on the tree of `1a0c9162`, which changes a file that `py-tests` covers. The gate was green, but the set of green steps was not whole on one tree.

**Recommendation.** Put the work report in the commit body, as a block of 4 to 5 lines: `Checks` (with the tree id), `Not run` (with a reason), `Review`, `Limit`, `Record`. Reject `logs/` (not in git) and a new `.scratch/` file for each change. A ticket Answer and a plan `Status:` line keep their own results and do not copy the block. A session with no commit gives the block in its final message, and the session that commits copies it. The template and one rule line are one session or less (inferred). An optional helper script that lists the history rows of a commit is a second task of one session (inferred). No change to `build.py` is needed.

**Questions for ticket 16.**
1. Does the owner accept the commit body as the one place, with a block of about 5 lines in each commit that changes tracked files?
2. Is a report that only states the facts enough, or must a commit stop when a relevant step has no green row on its own tree? The second answer is the base of ticket 17.

Asset: [task-13-work-report.md](../assets/task-13-work-report.md)
