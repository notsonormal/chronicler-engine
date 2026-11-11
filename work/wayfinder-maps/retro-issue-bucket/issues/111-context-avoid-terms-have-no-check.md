# The *Avoid* terms of `CONTEXT.md` have no check, so only a reviewer finds them

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`CONTEXT.md` lists an *Avoid* line for each term, but no check reads it. Must a Vale rule, generated from those lines, flag an avoided term in the docs and in the generated diagram pages?

## Context

Source: `/retro` 2026-11-09, the session of statechart-models ticket 14 and its `/simple-after-plan` pass, with its subagent sessions (comment fixer `f4d7c890`, document review `20490787`, Standards review `383723c2`, Spec review `e2e6cad4`).

- **known** The document review found about 10 term breaches in the diagram views of ticket 14: "run" alone for an Autoplay run (`CONTEXT.md`: *Avoid* "Run (alone)"), the lane "You" for the player, and "analysis call" for the Quantifier call. I fixed them by hand in five `.view.mjs` files and wrote the 13 pages again (about 90 s each time).
- **known** `styles/Chronicler/` holds ForumLinks, FutureTense, ParagraphLength, PastDeletion, Semicolon and SentenceLength. None reads `CONTEXT.md`.
- **known** `scripts/vale_lint.py` (`GENERATED_DIRS`) skips `docs/diataxis/reference/diagrams/` entirely, because a label built by a view can break the sentence rules. A term rule would still apply there.
- **inferred** Some *Avoid* entries are context words ("character" in lower case, "run" alone), so a plain substitution rule gives false hits. A rule for the capitalised or exact forms only is safer.

## Options to decide

- **A.** A script turns the *Avoid* lines into `styles/Chronicler/Terms.yml` (a Vale `substitution` rule at warning level). `vale_lint.py` runs only that rule on the generated pages.
- **B.** A line in `/document-review`: check each term against `CONTEXT.md` *Avoid*. No tool.
- **C.** Nothing: the document review found them.
