# Decide what chooses the Expression in Chronicler

Type: grilling (HITL)
Status: open
Blocked by: 02

## Question

What part of Chronicler chooses the Expression of each Portrait, and is the Expression part of the Swipe state?

## Context

- Candidates: a new field in the Quantifier result for each Character present, a new agent hook, tags that the narrator writes in the Narration, or a method with no model call.
- The Swipe state holds the game state of one Swipe (`CONTEXT.md`). If the Expression is part of it, a Swipe change or a Retry also changes the Portrait, and an older Message can show the Expression it had.
- Also ask whether the Persona's Portrait gets an Expression, and from what: an Input is written by the player or by the narrator (Impersonate).
- The Expression list (fixed for every Character, or defined for each Character) is in the map's fog. Settle it here if this decision depends on it.
- From ticket 02: the methods that fit are a Quantifier field (no extra model call), a narrator-written tag (shares the Narration call, and the tag can leak into the story text), and a word list as a fallback. Marinara reads the Persona's Expression from the latest user message.
- The cost of a longer Quantifier prompt is not measured.
- Chronicler does not stream the reply (`"stream": false` in `src/adapters/driven/llm/transport/utils/request.rs`, known). The Quantifier runs after the Narration, so with a Quantifier field the Portrait changes after the text shows. Decide whether that delay matters.
- Use the findings of [Research: how other apps choose an Expression](02-research-expression-selection.md). Use `/grilling` and `/domain-modeling`.

Done when the owner has chosen, and the answer records the choice, the Expression list rule if settled, the failure case (an Expression that is not on the list, a failed call), and any new tickets.
