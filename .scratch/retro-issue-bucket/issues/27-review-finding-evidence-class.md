# Give every review finding an evidence class

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

*(Carried over from the 2026-10-04 reflect run, which accepted it and never applied it.)*

A findings list with no evidence class turns the operator's only safe instruction into "apply all", so a reviewer error ships with the same weight as a quoted-standard violation. The lead repeatedly said "Apply all those fixes"; the one time triage happened by hand the user said "Fix the valid issues. Ignore the ones that were intentially changed from the spec". An implementer recorded a verified pushback that half a finding was wrong ("I disagree with half of S1 — the rollback would be a bug … I verified why").

Should each finding carry its evidence class?

- **Yes.** Label each finding quoted-rule / quoted-spec-line / inference, and list the ticket's accepted deviations in the review bundle.
- **Only in aggregate.** The lead records the class when merging the two axes.
- **No.** Keep the current free-form findings.

## Context

- Source: `/reflect` 2026-10-04, session `01a10894-87dd`; re-observed in the 2026-10-07 window.
- Evidence: `tmp/reflect/final_01a10894-87dd.md` (Accepted, unlanded); `tmp/reflect/final_01a1089d-5ec4.md`.
- `.agents/skills/code-review/SKILL.md` §4 sub-agent briefs / §5 aggregate.
- Related: [Never tell a review axis to skip `tooling.patch`](21-review-bundle-tooling-patch.md).
