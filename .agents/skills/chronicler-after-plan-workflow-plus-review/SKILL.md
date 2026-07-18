---
name: chronicler-after-plan-workflow-plus-review
description: Chronicler skill used to update the chronicler tests and documentions after a successful plan
metadata:
  language: rust
  workspace: chronicler_engine
---


# What I do

1. Verify that the implemention matches the existing plan. This is a post implementation verification so you MUST read the plan and actively check changed files. Any missing deferred, missing or changed features MUST be clearly presented to the user, with the reasoning included.
2. Archive the recently used plan for the session. The plan might be in `chronicler_engine/docs/plans`. The archive folder is `chronicler_engine/old-docs/archived-plans`.
3. If the plan was created through the skill `/wayfinder` (`.agents/skills/wayfinder/SKILL.md`) , it will be associated with a ticket in `.scratch`. Rather than being archived, you need to follow the workflow in the wayfinder skill. 
4. Update all the documentation in the `chronicler_engine/docs` folder to match latest changes. Do not update documentation for the sake of updating as this results in sediment. See the skill `/chronicler-docs-hygiene` (`.agents/skills/chronicler-docs-hygiene/SKILL.md`) for standards.
5. Update all the unit and integration tests as needed for the latest changes.
6. Check the code coverage and try to keet at 80% minimum for all files (run `build.py --coverage`).
7. Ensure that there is no 'ai slop' or 'hacks' in the code due to repetitive fixes without a cleanup.
8. Check if there is any duplicated code, any 'bad tests', any implemented or missing features.
  - Run `python chronicler_engine/scripts/healthcheck.py duplicates` to get a prioritized duplicate-code summary. For full options, run `python chronicler_engine/scripts/healthcheck.py duplicates --help`.
9. Check to make sure that the code is consistent with any existing patterns or, if the new patterns is an improvement, that older code is updated to match
10. Run the `/code-simplification` skill against the (usually uncommited) changes
11. Run the `/chronicler-comment-fixer` skill against the (usually uncommited) changes. Sometimes comments are written in lieu of fixing issues, surface any comments like that for investigation.
12. Run the full build with the script `chronicler_engine/build.py`. **All Tests Must Pass**. Failing tests should be fixed even if they are failing for reasons that seem unrelated to the recent changes. "Seems unrelated" is a subjective opinion that is often wrong.
13. Run the 3 different subagents for each of the 3 skills `/thermo-nuclear-code-quality-review`, `/code-review`, `test-police`. The first two subagents should be expliclty instructed now to run tests or to build, as we've already done both as they can't both run `build.py` as the same time. The `/test-police` does not need this instruction as re-running the build/tests is part of its workflow. 

This is intentionally a copy of `.agents/skills/chronicler-after-plan-workflow/SKILL.md`. Everything is the same except for the additional 13th step.

_See `.agents/skills/_shared/chronicler-shared.md` for documentation sync and visual verification steps.
