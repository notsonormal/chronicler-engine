# Build the local chief-of-staff skill

Type: task
Status: resolved
Blocked by: 12

## Question

Write `.agents/skills/chief-of-staff/` with the changes that [Decide the local changes to the chief-of-staff skill](12-chief-of-staff-skill-changes.md) decided. Point the tracker doc's wiki line at the local skill. Pass `python build.py validate-work-records` and the full `python build.py`. Use `/writing-for-agents` for the skill text.

## Answer

Done on 2026-11-11.

- `.agents/skills/chief-of-staff/SKILL.md` is `mattpocock/skills@4ae2317`, `skills/in-progress/chief-of-staff/SKILL.md`, with the two text changes of [ticket 12](12-chief-of-staff-skill-changes.md): the Wiki section names `work/wiki/` and the tracker doc, and large work goes through `../wayfinder/SKILL.md` and then `../implement-spec/SKILL.md`. A `diff` against upstream shows only those two lines. `agents/openai.yaml` is not copied.
- The Wiki line of `docs/agents/issue-tracker.md` now cites `.agents/skills/chief-of-staff/SKILL.md` instead of the upstream path.
- `python build.py validate-work-records`, `validate-docs`, `vale` and the full `python build.py` passed (`logs/build_20261111_201450_3677023.log`).
- Not committed.
