# Research how agents use the current docs and plans

Type: research (AFK)
Status: resolved
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

## Answer

Asset: [research-10-agent-use-of-docs.md](../assets/research-10-agent-use-of-docs.md). Data: 1355 main-folder sessions with tool calls (2026-08-11 to 2026-11-01) and 399 commits. Labels, limits and the scripts are in the asset.

**Gist (known unless the asset says inferred).**

- Agents read docs. 92% of 407 code-work sessions read some doc, plan or instruction file. 56% read a Diátaxis or spec file (50% of non-delegates, 75% of delegates), and 175 of those 227 did so before the first code edit.
- Many reads follow a brief. 58% of the doc reads of implementer delegates name the doc in their brief. Specs (40% of code-work sessions) are read as often as Reference (41%). Explanation docs are read 3.4 times less (12%).
- Agents follow the `[DOC: …]` anchor in `src/` in 14% of the cases (20% in October).
- Docs change with the code. 126 of 178 commits that touch `docs/diataxis/` or `docs/specs/` also change `src/`. 12 are docs only, 16 only regenerate an index. A Reference doc is edited in 28% of the commits that touch its anchored files, so no doc copies the code.
- Plans are read as an input (26% of code-work sessions, 25% of reviews, mostly because a brief names them). Maps and tickets are the most read planning files (39% and 46%). No session read a `docs/adr/` file.

**Recommendation.**

- **C-42** (one ranked work list): drop. Agents search inside one effort folder; a search across efforts is in 2% to 8% of sessions.
- **C-43** (plan conventions): keep for ticket 16, narrow. Ask only whether to add a limits section and a Status rule to the plan template.
- **C-44** (decision record folder): drop. Agents meet decisions in tickets and maps. `_closed/` is read in 31 sessions. The Hex reversal argument cannot be tested with this data.

**Questions for the doc practices** (the coordinator turns these into tickets; details in part 7 of the asset):

1. How should an agent find the doc for the code it touches: a route it finds alone (anchor, module map, capability page), or a route that briefs and skills carry? (C-09, C-11, C-01)
2. Are the Explanation docs meant for agents, for the owner or for both? (C-01, C-03)
3. Do the docs mislead an agent when it reads them? A follow-on AFK ticket can sample the 230 session and doc pairs where a doc was read and then edited. (C-03, C-11)
4. Is a spec the right unit for a capability page? (C-09, C-10)

C-02, C-10 and C-32 have no data here. They stay owner questions.

**Questions for ticket 16** (at most 2): (a) C-43: add a limits section and a Status rule to the plan template, or leave plans as they are? (b) C-42 and C-44: confirm the drops.
