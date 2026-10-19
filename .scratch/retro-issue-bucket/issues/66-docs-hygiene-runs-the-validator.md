# Make the docs-hygiene skill run its validator under a no-build brief

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Phase 1 of `/chronicler-docs-hygiene` says "Run the validator first (`python scripts/validate_docs.py --strict`)". The lead's brief said "Do not run `python build.py`". The scout read that as a ban on the validator too, and left Phase 1 unchecked. Should the skill say that the validator is read-only and always runs?

- **One sentence in Phase 1**: the validator is read-only and fast; run it unless the brief names it.
- **Leave the skill**; the lead's briefs name the validator explicitly.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Scout `69dd27b4` final message: "I did not run `python build.py` or `scripts/validate_docs.py`, so the machine layer (Phase 1) is unchecked."
- `.agents/skills/chronicler-docs-hygiene/SKILL.md` Phase 1.
