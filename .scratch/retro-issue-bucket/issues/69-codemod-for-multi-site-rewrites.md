# Give implementers a safe way to rewrite many call sites

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`fix-tests` replaced 70 copies of the story read and 54 form-request builds across 11 files. The `edit` tool replaces literal blocks only. The agent wrote about ten regex heredocs: four errored, two changed nothing, and two broke a file (`mismatched closing delimiter`). It reverted with `git checkout` five times. This was about 12 of its 22 minutes. Should the repo offer a structural rewrite?

- **A documented tool** (for example `ast-grep`, or a `scripts/` helper with a dry-run count) named in `AGENTS.md` or the implementer role.
- **A rule for briefs**: a rewrite of more than N sites goes file by file with `edit`, and `clippy` runs after each file.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Implementer `63ee340e` entries 195, 199, 239, 273, 310, 312, 423, 427; reverts 198, 341, 347, 422, 432, 436.
