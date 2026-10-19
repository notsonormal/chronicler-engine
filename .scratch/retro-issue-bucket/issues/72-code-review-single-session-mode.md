# Give `/code-review` a single-session mode

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`/code-review` spawns two sub-agents (Standards and Spec), each capped at 400 words. The after-plan workflow runs it as a delegated reviewer whose brief says "do not start sub-agents" and "cap at 40 findings". The reviewer followed the brief, and the skill has no shape for that case. Should the skill have a single-session mode?

- **Add a branch**: when the skill runs as a delegated reviewer, it does both axes in one session with two sections.
- **Let the workflow call the skill as written**, with its own sub-agents.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- `.agents/skills/code-review/SKILL.md` line 11 and section 4; reviewer `20247c8f`.
