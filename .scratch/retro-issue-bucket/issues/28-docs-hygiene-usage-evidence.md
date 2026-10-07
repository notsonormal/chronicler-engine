# Add a usage-evidence phase to `chronicler-docs-hygiene`

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

*(Carried over from the 2026-10-04 reflect run, which accepted it and never applied it.)*

Docs earn their place by evidence of use, not plausibility, and that evidence lives in session transcripts. The user asked "Can you read through the session history and see if docs/diataxis/how-to/debugging.md is ever actually used in any of the sessions … If not, there is no point in keeping it around"; `debugging.md` was deleted (`47f97eef`). The same session reinstated the removed session-search section of `AGENTS.md` only after usage evidence.

`chronicler-docs-hygiene`'s description covers rule violations, mode drift and stale-source drift, not usage-evidence pruning.

Should the skill gain a usage-evidence phase?

- **Yes.** Add a phase that mines past sessions (semantic search) and git history before deleting or reinstating a doc, and extend the description.
- **Yes, but as a separate skill.** Usage archaeology is a discovery task, distinct from auditing.
- **No.** Keep the audit read-only against `docs/diataxis/`.

## Context

- Source: `/reflect` 2026-10-04, session `01a10894-6127`; still absent on 2026-10-07 (no match for "usage"/"prune" in the skill).
- Evidence: `tmp/reflect/final_01a10894-6127.md` (Accepted, unlanded).
- `.agents/skills/chronicler-docs-hygiene/SKILL.md:3` — the description scope.
- `session_search` is the semantic tool; raw transcripts live under `~/.pi/agent/sessions/`.
- Related: [Widen the stale-reference sweep to `.agents/skills/`](15-stale-references-in-skills.md).
