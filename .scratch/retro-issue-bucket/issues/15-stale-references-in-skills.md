# Widen the stale-reference sweep to `.agents/skills/`

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

When ticket 70 removed the `POST /check-text` route, the stale-reference sweep covered `docs/` and left a live reference in a skill body. A review axis caught it: "Stale live reference to the removed route — `.agents/skills/chronicler-ui-investigator/SKILL.md:81`: '`/check-text` standalone text check…' This is the only remaining reference to the route outside `.scratch/`/`old-docs/`."

`chronicler-docs-hygiene`'s description scopes the audit to `docs/diataxis/`. Skill bodies rot the same way and no review's default scope includes them.

How should the sweep cover skills?

- **Widen the skill's scope.** `chronicler-docs-hygiene` sweeps `.agents/skills/` for removed identifiers alongside `docs/`.
- **A mechanical check.** A script greps `docs/` and `.agents/skills/` for route/identifier tokens `extract_http_routes.py` no longer emits.
- **Both.** The mechanical check is the durable form; the scope line points at it.

## Context

- Source: `/reflect` 2026-10-07, session `01a11377` (spec review axis).
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 7; Backlog "Removed-reference sweep for skills").
- `.agents/skills/chronicler-docs-hygiene/SKILL.md:3` — description scopes to `docs/diataxis/`.
- `.agents/skills/chronicler-ui-investigator/SKILL.md:81` — the stale `/check-text` reference.
- Related: [Catch hand-typed derived numbers in prose](14-derived-numbers-in-prose.md).
