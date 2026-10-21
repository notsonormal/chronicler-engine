# Give the Lead the file tools it needs to check a quote

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

Every Lead in the 2026-10-11 run held agent management plus supervisor/peer messaging and no file tool: `{"mode": "orchestrate", "leadTools": []}` is recorded in each session's `pi-herdsman-lead-execution` entry, and the six `toolsAdded` lists are identical (`delegate_agent`, `list_agents`, `steer_agent`, `message_supervisor`, `list_peers`, …). So a Lead cannot read the skill it was told to follow, the ticket it is working from, or one quoted line of a finding. In the after-plan pass two of its eight subagents existed only to print text the Lead could not open, and its first action was two `read` calls that returned `Tool read not found`.

How much file access should a Lead hold?

- **`read` only, plus `ls`/`find`/`grep`.** Enough to check a quote, read the skill and brief, and judge a finding against the ticket; every mutation still leaves through a delegated Agent.
- **`read` + `bash`.** The Lead can also inspect git state, run one `test-pattern`, and commit its own `.scratch` edits; the Lead's steering text keeps mutations delegated.
- **Keep orchestration-only, and state the toolset in the brief.** Every brief opens with the tools the Lead actually has, so no brief orders a read (see [Check a delegated brief against the role's tool grants before dispatch](11-delegated-brief-tool-grants.md)). The cost — one agent dispatch per quote — is accepted.

## Context

- Source: `/retro` 2026-10-11 on the phase2 tickets 09–13 and the after-plan pass. Lead sessions: `7466f432` (t09), `dcac3f11` (t10), `dbb9e4fc` (t11), `ca0ec729` (t12), `600984c6` (t13), `11b62cb1` (after-plan).
- **Shared, not specific to the after-plan Lead.** All six records read `leadTools: []`. Ticket-12 Lead at idx 12–14: two `Tool read not found`, then "The read tool isn't in my tool list … I have no read/edit/bash tools. So I must delegate all execution to Agents." Ticket-10 Lead: "the brief says run `/code-review` against the branch's base; but I have no bash". Ticket-13 Lead: "I have no bash tool! I can't commit myself … So: use a dedicated agent for commits" (it spawned a `gate-answer-commit` implementer for the commit).
- After-plan Lead at idx 11–14 (`11b62cb1`): `read` of `.agents/skills/simple-after-plan/SKILL.md` and `tmp/phase2-after-plan-brief.md` both failed; "I have no direct file/bash tools in this session — orchestration only, so every execution unit gets delegated."
- Cost in that pass: 8 subagents; the delivered reports ran to 735, 535, 381 and 298 lines (the two long ones truncated, see [Deliver the tail of a long delegated report](41-delegated-report-truncation.md)); the ticket-Answer record had to be re-printed in full by a second agent because the Lead could not open the five ticket files itself.
- Correction to the Manager's list: in all six rosters the `reviewer` definition WAS present (`tools read, ls, find, grep, bash`), and tickets 09, 10, 12 (three reviews) and 13 delegated it. "Leads have no reviewer definition" does not hold for these six sessions.
- Related: [Deliver the tail of a long delegated report](41-delegated-report-truncation.md).
- More evidence (`/retro` 2026-10-19, whole-rebuild review): every agent had `read`, but `read` returns no line numbers, so each `file:line` citation went through `cat -n`, `sed -n` or `grep -n`. The simplify pass pulled 570 KB of tool output in 68 calls (`a4c24ef9` entries 27, 29, 74), and about 40 of test-police's 142 bash calls were line-range reads. This may be a harness change, not a repo change.

## Answer

2026-10-21. **Dropped (stale).** Triage pass 2 (`../triage.md`), HEAD `511e28dd`; the owner approved the close.

- The herdsman Lead (`leadTools: []`) is gone. Delegation is now the repo-side `.pi/extensions/delegate/` (`f3af8992`, 2026-10-18), and the parent session keeps its own tools.
- The residual point (`read` returns no line numbers) is the pi harness, not this repo. Reopen only if a tools-less coordinator returns.
