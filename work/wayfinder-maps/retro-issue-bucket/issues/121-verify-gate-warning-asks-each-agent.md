# The `verify-gate` warning asks each agent that changed a function to update the model

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The warning says the agent that changed a listed function runs `/maintain-formal-verification-skill` before it ends its task. Must it say "before the commit" instead?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass), with the sessions `98c41966`, `d3622523` and `ed66e704`.

- **known** The model was updated at 00:49 on 2026-11-12; two later fix rounds changed `Job::narrate` again, and their briefs told them not to touch the model. The lead checked the model twice more.
- **known** The text is at `scripts/verify_coverage.py:49`.

## Options to decide

- **A.** "before the commit"; the lead runs the skill once after the last fix round.
- **B.** Nothing.
