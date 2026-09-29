# Copy sweep

Type: task (AFK)
Status: open
Blocked by: —

## Question

Fix the UI copy problems from the review:

- "Add LlmProviderConfig" (a Rust type name) as a heading and button in Settings (finding 5.1).
- "(1 games)", and the leading "- " before world descriptions (finding 5.2).
- The jargon "Active · IF" and "(transient)" in the slash menu (finding 5.4).
- New Game world/persona selects with no labels. The "CURRENT" badge repeats the "Active Game" heading. "No saved games." shows while one game exists, because the list holds only non-active games (finding 5.5).

## Context

- Use `CONTEXT.md` terms in user-facing copy where they fit, for example "Guided Generation" rather than "transient".
- Game naming (finding 5.3) is in [Decide how games are named and renamed](21-decide-game-names.md), not here.

## Done when

- Each item is fixed. Existing test assertions on the old strings are updated.
- `python build.py` is green. Commit after user approval.
