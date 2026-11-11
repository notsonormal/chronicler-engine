# The `grep * . *` denial costs a turn because its reason starts with the problem

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Agents in a subfolder run `grep -r … .`, the permission policy denies it, and the agent needs one more turn. Must the reason text start with the command to use?

## Context

Source: `/retro` 2026-11-08, the coordinator session of statechart-models tickets 08, 10, 11, 12, 13 and 16 (`01a1bd22`), and its subagent sessions (t13, t16, the t13 plan review, the statechart docs review and the Standards review).

- **known** Five denials, each from a subfolder, for example `cd .agents/skills/statechart/scripts; grep -n "written" -r .`. Each agent used `rg` in the next call.
- **known** The reason in `.pi/extensions/pi-permission-system/config.json` starts with "Repo-root grep walks target/ (~10 GB) and hangs."
- **known** `AGENTS.md`, Permissions System: the agent may recommend a permission change, but only the owner applies it.
- **known** Again on 2026-11-09: two more denials in the ticket-14 sessions. One in the coordinator (`grep -n verify_coverage … ` from the repo root) and one in the comment fixer (`grep * . *`). Each agent used `rg` in the next call.

## Options to decide

- **A.** The owner changes the reason to start with "Use `rg` for a recursive search (it skips target/)." The rule stays.
- **B.** Nothing: one turn per denial is a small cost.
