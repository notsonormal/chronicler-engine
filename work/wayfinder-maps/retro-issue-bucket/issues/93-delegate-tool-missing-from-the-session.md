# The delegate tool is missing, so the subagent steps of the after-plan skills cannot run

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`/simple-after-plan` steps 2 to 4 and `/code-review` each ask for `implementer` or `reviewer` subagents. In this session no delegate tool was in the tool list, although `.pi/extensions/delegate/` exists. Earlier in the same session (2026-10-11) delegation failed twice with `pane_not_ready` ("Agent did not initialize its mailbox"). The lead ran the comment pass, the document review and both review axes itself, in one context, so the two axes were not independent.

Why was the tool not loaded, and what should a skill tell the agent to do when it is absent?

## Context

- Source: `/retro` 2026-10-31, session `01a122c2-4e16-73db-89b0-50a2465e456b` (a long session: started 2026-10-09, resumed 2026-10-30).
- Compare Done/Won't do line 40 ("the lead lost its file tools (delegation is now `.pi/extensions/delegate/`)").
- **Inferred:** a session that started before the extension changed keeps its old tool set. Not checked.
- **Possible shapes (proposals).** A: the after-plan skills name a fallback (run the steps in order in the lead, and say so in the report). B: a session start check that lists the extensions that failed to load. C: no change; start a new session for the after-plan pass.
