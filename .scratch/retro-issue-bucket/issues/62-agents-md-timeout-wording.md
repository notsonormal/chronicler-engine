# Reword the build timeout instruction so it does not read as the `timeout` command

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`AGENTS.md` says "Call `build.py` with a bash timeout of 1200 seconds." The permission config denies the `timeout` command ("Set the timeout parameter of the bash tool instead"). Three agents in this run wrote `timeout 1200 python build.py ...` or `timeout 70 yes` and lost a turn to the denial, the lead among them. Should the sentence name the bash tool's `timeout` parameter?

- **Reword**: "set the bash tool's `timeout` parameter to 1200".
- **Leave it**: the denial message already teaches the fix, at one turn per agent.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- `AGENTS.md` Development Loop: "Call `build.py` with a bash timeout of 1200 seconds."
- Denials: lead session `01a15614`; fix-tests `63ee340e` entry 172; test-police `d448fab2` entry 165.
