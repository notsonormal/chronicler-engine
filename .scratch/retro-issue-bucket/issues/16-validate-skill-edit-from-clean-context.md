# Validate a skill edit from a clean context

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A skill or comment rewrite is validated by its author re-reading it in the context that produced it, which supplies assumptions the text does not carry. The user prescribed the fix directly: "Fix the skill and then create a subagent to run the updated skill. That way it will run without this context and you can check the results." The fresh-context subagent then classified 310 and deleted 235 where the author-context pass had kept them.

`writing-for-agents/SKILL.md:45` defines completion criteria but names no clean-context run.

Should a clean-context run be the completion criterion?

- **Yes, add it.** A skill/instructions edit is done when a fresh-context agent runs the updated artifact and the author compares the outcome.
- **Only for behavioural skills.** Apply it to skills whose body changes agent behaviour; skip pure reference edits.
- **No.** Keep the criterion as author review; a clean run is too expensive.

## Context

- Source: `/reflect` 2026-10-07, session `01a108d1` (2026-10-04 21:28).
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 8).
- `writing-for-agents/SKILL.md:45` — "Steps and completion criteria".
- `:49` — clarity/premature-completion reasoning; the same logic applies to validating the artifact.
- The clearest existing instance is ticket [09 — Choose the AGENTS.md restructure to adopt](09-choose-agents-md-restructure.md), where three subagents ran the variants from a detached worktree.
- Related: [Measure the generator-owned share before trimming](26-measure-generated-share.md).
