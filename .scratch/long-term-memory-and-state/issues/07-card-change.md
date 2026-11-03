# Decide what a card change of one Game holds and who writes it

Type: grilling
Status: open
Blocked by: 01

## Question

Which parts of a Character card can change in one Game? Which call writes a change, and how often? How does the changed card reach a prompt?

## Context

- **Fixed at charting (map Notes, "Card changes").**
  - A card change applies to one Game, and is saved against the Character of that Game. The World card does not change.
  - A writer changes the card directly, with no approval. A person can view, edit and revert the change in the dashboard.
  - Each change records the Message and the Swipe that made it. A Retry, a swipe switch or a delete that removes that Swipe also removes the change. The list of changes is the version history of the card in that Game.
- **The card today (known, `src/domain/model/character.rs`).** The text fields are `description`, `personality`, `scenario`, `example_dialogue` and `summary`. A Character also has `relationships` (each with `static_text` and `dynamic`), `inventory` and `triggers`.
  - Nothing writes `relationships[].dynamic` during play. The prompt shows `dynamic` when it is set.
  - No prompt part reads `inventory`.
  - `summary` is the short entry of an absent Character in `<KnownCharacters>`.
- **ME (known, [how other tools keep long-term state](../../prompt-flexibility/assets/long-term-state.md), section 3).** The Card Evolution Auditor proposes durable changes every 8 Messages. It names what to ignore: a temporary mood, the place, clothes, and healed injuries. A person approves each change, with a check of the old text, a stale marker and a card version history. [Find how Marinara Engine keeps memory with its agents](01-marinara-memory.md), point 5, reads it in depth.
- **The link to memory (open).** The coordinator's draft at charting: a card change is a durable change to who a Character is (a relationship, a role or a trait), not an event. The owner did not confirm it. Is a card change the durable part of a Character memory, or a separate thing? If [Choose the memory candidates to try](05-memory-candidates.md) is resolved, read it first.
- **The Persona.** A Persona belongs to no World and plays many Games. Can its card change inside one Game too?

## Output

A resolution with:

- the fields that can change;
- the writer, its trigger and its failure rule;
- how a change reaches the prompt: the whole changed card, or the World card plus a list of changes;
- whether the Persona card changes too;
- the facts that the build ticket needs.
