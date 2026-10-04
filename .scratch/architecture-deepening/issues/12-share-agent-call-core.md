# 12 — Share one Agent call core between Options and Quantifier

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to one shared Agent call core (scene-context prompt blocks plus a
call-with-retries helper) used by both the Options and Quantifier Agents —
and if so, what does each Agent keep, and how is the retry policy passed in?

## Background

This is **candidate D** of the 2026-10-04 review. See
`assets/architecture-review-2026-10-04.html`, card D. The preset-fetch part of card D
is ticket 07 and is out of this ticket's scope.

The friction: the two Agents copy the same body:

- **Recent history.** Both orchestrations take the last 4 history entries the
  same way (`.iter().rev().take(4).rev()`), at
  `options/utils/orchestration.rs:36` and
  `quantifier/utils/orchestration.rs:40`.
- **Prompt blocks.** `options/prompt.rs` and `quantifier/prompt.rs` each
  render a byte-identical `<CurrentRoom>` block (options `:43-54`,
  quantifier `:57-68`) and `<RecentHistory>` block, including the same
  `sender_label` match. About 35 lines each.
- **Retry loop.** Both run a 2-attempt loop around the recorder call. The
  retry rules differ: Options retries on zero parsed options, Quantifier on
  low confidence.

Sizes: `options/utils/orchestration.rs` 105 lines,
`quantifier/utils/orchestration.rs` 224 lines.

Deletion test: removing one copy leaves one scene-context renderer. Deleting
the orchestration wholesale would lose the two retry rules, so they become
parameters, not deletions.

## What this ticket resolves

- **Commit or reject.**
- **Interface.** What the shared core takes (Agent context, prompt tail,
  parser, retry rule) and returns.
- **Home.** `application/agents/` shared module vs part of the deepened Agent
  module ticket 13 may produce.
- **What survives.** Prompt-shape tests for both Agents; whether the
  `<CurrentRoom>` / `<RecentHistory>` shape gets one test instead of two.

## Constraints

- The two prompts may diverge on purpose later. The grilling should state
  whether the shared blocks are a contract or a convenience.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`.
- Related: ticket 07 (preset on `AgentContext`) and ticket 13 (registry). The
  map's fog "Agent cluster shape" may merge these three after 13 resolves.
- Changes in agent prompts fall under the LLM-test policy at implementation
  time (note in hand-off).
