# Check that every backticked identifier in a Diátaxis doc exists in the tree

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Phases 4 and 6 of `/chronicler-docs-hygiene` ask the agent to check that each symbol and path a doc names exists. The docs scout of this run wrote that check four times as ad-hoc scripts over more than 1200 backticked tokens. It found `StateRepository` and `RwLock`, which do not exist. The check repeats on every docs review. Should it be a machine check?

- **Add it to `validate_docs.py`** as a warning (identifiers are noisy: XML tags, config keys).
- **Add a `healthcheck.py docs-identifiers`** subcommand that the skill runs, report-only.
- **Leave it to the skill.**

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Scout `69dd27b4` entries 190, 192, 197, 231; findings DOC-5 and DOC-7 in `.scratch/rust-rebuild-build/assets/whole-rebuild-review/findings/docs.md`.

## Amendment 2026-10-25: routes in skills

- Source: lead session of 2026-10-24/25 (glossary alignment, `docs/plans/glossary-alignment-plan.md`); doc review `7f11991f` finding 1.
- After the rename `/status/generating` → `/status/busy`, `.agents/skills/chronicler-ui-investigator/SKILL.md` kept the old route in four places. No check found it; only the document review did. An agent that follows the skill gets a 404.
- Extra scope: the check also covers `.agents/skills/**/SKILL.md`, and compares each route string (`/status/…`, `/fragment/…`) with `docs/diataxis/reference/frontend/http_routes.md`, which `scripts/extract_http_routes.py` generates. Routes are a cheap, low-noise subset of identifiers: they could ship first.
