# Research: does a `CONTEXT.md` entry make writers align?

Type: research
Status: open

## Question

When a term enters `CONTEXT.md` with an `_Avoid_` list, do writers stop using the avoided words after that? This is the map's first outside check: it tests the assumption that a name makes writers align. Read the map's Notes first (`.scratch/conceptual-integrity/map.md`). Record what happened. Do not judge it.

Facts known at charting: 16 commits touch `CONTEXT.md` since 2026-08-11 (commit `d8644394`). `Save unit` entered on 2026-10-19 (commit `4eeadba2`), so its history after the entry is short.

Method, as a start:

- `git log -p -- CONTEXT.md` gives the date when each term and each `_Avoid_` list entered.
- For a set of terms, count the avoided words in `src/`, `tests/` and `docs/` at commits before and after the entry. Count identifiers, code comments and doc prose apart where that helps.
- Some avoided words are common English ("run", "input", "text", "story"). Choose the terms where a count means something, and say why.
- Say whether a change came in the same commit as the entry (a rename) or later (writers who followed the entry).
- The cause can also run the other way: a writer may have added the entry because writers already diverged. Say how far the counts can show cause.

Write `assets/research-03-names-and-alignment.md`:

- **Summary:** at most ten lines, first.
- **Method:** the commits and the commands used.
- **Per term:** the term, the date it entered, its avoided words, the counts before and after, where the remaining uses are, and whether the change was a rename or came later.
- **What the counts can show, and what they cannot.**
- **Questions raised.**

## Done when

At least eight terms, or every term with a usable count if there are fewer, have counts before and after, with the commits used. The asset states the limits of the method.
