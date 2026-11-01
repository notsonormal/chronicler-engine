# Research how agents use the current docs and plans

Type: research (AFK)
Status: open
Blocked by: —

## Question

When an agent works on code, does it read the current docs and plans? What makes the docs churn?

## Context

- **Why.** An AI writes the docs, and an AI reads them. The owner does not know how useful they are to an agent, and wants no rewrite that rests on a hypothetical (owner, 2026-11-01, in [Agree the practices to decide](./08-grilling-agree-the-practices-to-decide.md)). The doc practices wait for this answer: C-01, C-02, C-03, C-09, C-10, C-11 and C-32 (`assets/candidates.md`, part 2). Ticket 08's answer records the owner's first view of each.
- **A rough first count** (ticket 08, known, with limits). Only `read` tool calls in the main session folder were counted. October 2026 had 888 sessions with tool calls. 351 read a file in `src/`, 122 a Diátaxis Reference doc, 100 a spec, 89 `CONTEXT.md`, and 52 an Explanation doc. File reads: `src/` 3218, Reference 331, specs 243. The count does not separate code work from doc work. Git: 93 of 234 October commits touch `docs/diataxis/`. In September it was 22 of 101.
- **Sources.**
  - Session logs: `~/.pi/agent/sessions/`. The main folder is `--workspace-chronicler-engine--` (1440 `.jsonl` files on 2026-11-01). Worktree sessions are in the `--home-node-.herdr-worktrees-chronicler-engine-*` folders. Delegate transcripts are in the same session folders. `~/.pi/agent/delegate/<id>/job.json` holds the `label`, `role`, `sessionId` and `parentSessionId` of each delegate (342 on 2026-11-01), so delegate sessions can be sorted by kind with no guessing. A tool call is a `content` item with `"type": "toolCall"`, a `name` and `arguments`. A folder name starts with `--`, so give it to a shell command as `./--…`.
  - Git history since 2026-08-11. That is when `docs/diataxis/` moved to the root.
- **Measure.**
  1. Sort the sessions into kinds: code work, doc work, review and planning. Use the first user message, the files that the session edited, or the delegate label. State the rule you used.
  2. For code-work sessions, find which docs they read. Count `read` calls, and also `bash` reads (`sed`, `cat`, `rg`) on doc paths. Record whether each doc read comes before or after the first edit.
  3. Do the same for plans and decisions: `docs/plans/`, `.scratch/<effort>/map.md` and `.scratch/_closed/`. This part feeds C-42, C-43 and C-44: one ranked work list, plan conventions, and a decision record folder.
  4. Doc churn. Take the commits that touch `docs/diataxis/` or `docs/specs/`. Count how many also change `src/` (so they follow a behaviour change), how many change only docs, and how many only regenerate an index.
- **Not in scope.** Do not judge the doc layout, and do not propose feature maps. Record what the data shows. Label each claim known or inferred. Use the `/research` skill. Quote no raw transcript text in the asset. Give counts, file paths and session ids only, because `.scratch/` is tracked in git and a transcript can hold keys or private text.
- **What already exists.** Ticket 08 made its rough count with a short Python loop over the `.jsonl` lines that took each `toolCall` item. The `session_search` and `session_list` tools give a summary of each session, and those summaries can help sort the main sessions by kind.
- **Failure modes.**
  - A line that is not JSON: skip it and count it.
  - A session whose kind is not clear: count it as `unknown`. Do not guess.
  - A `bash` read that uses a glob or a pipe: count only literal doc paths, and state the undercount.
  - Sessions that ran in other checkouts: these are the worktree folders. Count them apart.
- **Output.** Write `assets/research-10-agent-use-of-docs.md`. The `## Answer` gives:
  - the gist
  - a recommendation for C-42, C-43 and C-44: keep for ticket 16, or drop with a reason
  - the questions that the data raises for the doc practices

  The coordinator uses those questions to turn the map entry **Docs organized as feature maps** into tickets.
- **When done.** Add `## Answer` and set `Status: resolved`.
