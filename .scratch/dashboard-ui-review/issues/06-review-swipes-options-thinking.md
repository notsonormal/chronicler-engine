# Review swipes, the options dock and the Thinking states

Type: task (AFK)
Status: open
Blocked by: —

## Question

What UI findings do these three Game-tab states have?

- **Several swipes:** switching, new swipe, and whether the snapshot restore is visible.
- **Options dock with options present:** both after a turn with auto-generate on, and after `/options`.
- **Thinking / generating states** during a turn.

## Context

- The first review's turn finished too fast to capture the Thinking states. Slow the turn down, for example with a Mock connection, rather than changing the user's narrator connection.
- The options dock was empty in the first review because the world has "Auto-generate options after each turn" off. That is expected, not a finding.
- Use a throwaway game.

## Work

- Use `/chronicler-ui-investigator`. Use the same method and P1–P3 scale as the [first review](../review-2026-09-29.md).

## Done when

- The findings are in the ticket answer.
- Each finding is a new ticket in this map or is added to an existing one. New tickets are also added to the final re-review's `Blocked by:`.
