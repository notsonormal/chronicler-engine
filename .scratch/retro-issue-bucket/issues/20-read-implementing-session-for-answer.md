# Read the implementing session before writing a ticket's answer

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Tickets implemented in another session carry their answer only in that session's transcript. No skill body names the tool, so the user had to supply it: "you should be able to use session_search or just search the pi session history directlyu". Later: "Can you search through the sessions for tickets … 68 … and 70 … I'm pretty sure they have been completed they so should be set the resolved and answered based on what happened in those sessions." The agent then rebuilt both answers with `recall` and `session_read` entry ranges.

How should the answer be recovered?

- **Name the tools in the map Notes.** A line tells the resolver to use `session_search`/`session_read` and names the raw JSONL path.
- **Put it in the resolve step.** `wayfinder` "Work through the map" step 4 tells the resolver to read the implementing session when the work happened elsewhere.
- **Both.**

## Context

- Source: `/reflect` 2026-10-07, sessions `01a11775`, `01a10d53`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 13).
- `session_search` is semantic; `session_read` supports entry ranges; raw transcripts live at `~/.pi/agent/sessions/--workspace-chronicler-engine--/<timestamp>_<uuid>.jsonl`.
- 2026-10-05 decision: naming `session_search` in `/retro` step 2 was dropped by the user, so route it to the map, not to `retro`.
- Related: [Give ticket resolution an owner after the commit](01-ticket-resolution-owner-after-commit.md).
