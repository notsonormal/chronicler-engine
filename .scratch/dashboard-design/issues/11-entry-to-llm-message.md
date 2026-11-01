# Decide the link from a story entry to its LLM message

Type: grilling
Status: open
Blocked by: 09

## Question

Does a story entry lead to the LLM message that made it, and where does that lead go?

## Context

- The critique's tuner persona: a bad Narration has no path from the entry to its LLM message. The author must switch to the LLM Messages tab and match the rows by time ([the Game tab critique](../assets/game-tab-critique/report.md)).
- An LLM message is the stored record of one model call and belongs to no story (`CONTEXT.md`). A Swipe can come from several calls (Narrator, Quantifier, Options, a Trigger).
- Whether a released player sees this link depends on [Decide the navigation and what a released player sees](09-navigation.md).
- The specs `docs/specs/llm_messages.md` and `browser_llm_messages.md` hold the current LLM message dialog.

Done when the answer records whether the link exists, who sees it, which calls of a Swipe it shows, and the build tickets it needs.
