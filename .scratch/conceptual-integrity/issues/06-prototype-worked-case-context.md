# Prototype: worked case, `CONTEXT.md`

Type: prototype
Status: open
Blocked by: 05

## Question

What does the framework note say about which terms go into `CONTEXT.md`? What does it say about the `/domain-modeling` rule "Only include terms specific to this project's context. General programming concepts (timeouts, error types, utility patterns) don't belong even if the project uses them extensively" (`.agents/skills/domain-modeling/CONTEXT-FORMAT.md:29`)?

Start with the `Save unit` case:

- `CONTEXT.md:107-109` defines **Save unit** as "One write to storage that lands whole or not at all", with `_Avoid_: Transaction, batch`.
- `src/adapters/driven/storage/transaction.rs:2` says "The transaction: one unit of record writes that lands whole".
- Is this one purpose with two names, or two purposes (the promise, and the storage mechanism that keeps it)?

Then apply the note to a sample of entries: terms that are in, terms that the rule keeps out, and terms that are missing. Show what the note says about each one.

Mark what in the case depends on the writers of this repo, and what would hold for a team of people and agents (map Notes: reference context).

The worked case changes nothing (map Notes). A change to the rule or to `CONTEXT.md` is a hand-off: list it.

## Done when

The worked case is an asset (`assets/worked-case-context.md`), the owner has reacted to it, and the hand-offs are listed.
