# Documentation for AI steering features

Type: task
Status: pending

## Question

Document the three steering surfaces in the repo's docs, following the diataxis reference convention and the Documentation Strategy: Semantic Mapping from AGENTS.md.

Per the design synthesis (`../research/04-design-synthesis.md`):

1. Author reference docs under `docs/diataxis/reference/` for the three features. Likely targets: a steering/guide reference, a narrator-action reference, an impersonate reference — or a consolidated `ai_steering.md` if the features share enough surface. Follow the existing doc structure (see `docs/AGENTS.md` index).
2. Add `[DOC: docs/diataxis/reference/<area>/<name>.md]` line-1 anchors + human-readable summaries to every new/changed `src/` file (per AGENTS.md "Module-Level Two-Line Headers"). The implementation tickets will create files in `src/domain/model/`, `src/application/prompting/`, `src/adapters/driving/http/` — each needs the anchor.
3. Update `CONTEXT.md` if the effort introduces or sharpens domain terms (per the domain-modeling skill). Candidate terms: "guided generation," "narrator action," "impersonate," "replay blob," "steering." The domain boundary (guide steers content / impersonate substitutes speaker / narrator is a permanent directive) is the kind of distinction CONTEXT.md exists to record.
4. Regenerate the docs index: `python scripts/generate_docs_index.py` (the pre-commit hook does this, but run it explicitly to verify).

Blocked by: 05, 06, 07, 08, 09, 10, 11, 12 (documentation follows implementation; it documents what was built).
