# Decide what the story log shows past 50 Messages

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Past 50 Messages the story log shows the 50 oldest (`take(MAX_LOG_DISPLAY)`), so the newest Messages are hidden and the last-entry controls act on a hidden Message (UI F1, P1; older than the rebuild). What does the story log show for a long story: the newest 50, all of them, or a window with a way to see earlier Messages? And where does that rule live (candidate S3: one story-window read, with the Retrigger rule from the `Tail`)? The answer adds the spec scenarios.

## Context

- Inputs: [the UI findings](../assets/ui-review/findings.md) F1 (a click deleted hidden Narration 67); [synthesis.md](../assets/architecture-review/synthesis.md) row S3 and §4 bug 2; FINDINGS "Rejected" TN-L4.
- Specs: `docs/specs/story_log.md`, `docs/specs/browser_story_log.md`. Skills: `/grilling`, `/domain-modeling`.
- From ticket 19 (resolved): the Retrigger button reads `Tail::retrigger_allowed` (R-65, an Impersonate Input included) and adds no condition of its own.

## Answer
