# Review create, save and delete flows

Type: task (AFK)
Status: open
Blocked by: —

## Question

What happens, on success and on failure, when the user creates, saves, or deletes a connection, a prompt preset, a world, a game, and a message? The main question: is every failure shown to the user?

## Context

- The first review did not exercise these flows. It found silent failures by reading code: `submitEdit`, `submitNewSwipe`, and `submitRetrigger` check neither `response.ok` nor `.catch` (finding 1.7).
- Provoke failures on purpose: invalid input, duplicate keys, a stopped engine mid-request.
- Use throwaway entities only. Do not change or delete the user's existing games, worlds, presets, or connections.

## Work

- Use `/chronicler-ui-investigator`. Use the same method and P1–P3 scale as the [first review](../review-2026-09-29.md).

## Done when

- The ticket answer has a table: flow × outcome (success / failure) → what the user sees.
- Each finding is a new ticket in this map or is added to an existing one. New tickets are also added to the final re-review's `Blocked by:`.
