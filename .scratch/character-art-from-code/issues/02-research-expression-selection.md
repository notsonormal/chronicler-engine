# Research: how other apps choose an Expression

Type: research (AFK)
Status: open
Blocked by: none

## Question

How do SillyTavern, Marinara Engine and similar apps choose the Expression of a character picture from the story, and what fails?

## Context

- Candidates to read: the SillyTavern Character Expressions extension (its classifier, its LLM mode, its default Expression list), Marinara Engine, and other roleplay front ends that show character pictures.
- For each app, record: what reads the text (a local classifier, an LLM call, tags that the narrator writes), when it runs, the list of Expressions and who defines it, what happens when the answer is not on the list, and what users complain about.
- Chronicler already runs the Quantifier after each generation. It records the Characters present and the destination of the Persona (`QuantifierResult`, `src/domain/model/quantifier.rs`). Note which outside methods could share that call.
- A turn costs money and takes several seconds (`PRODUCT.md`). Note the cost and the extra wait of each method.
- Use `/research`. Prefer primary sources: source code and the official docs.

Done when a summary is linked as an asset of this ticket, and the answer lists the methods with their cost, their Expression lists and their known failures.
