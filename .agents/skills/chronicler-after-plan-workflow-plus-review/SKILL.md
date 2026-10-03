---
name: chronicler-after-plan-workflow-plus-review
description: The post-implementation pass, plus the four review subagents and the consolidated findings report
disable-model-invocation: true
---


# What I do

Create a task list (using `TaskCreate`) for all these steps.

1. Verify that the implementation matches the existing plan. This is a post implementation verification so you MUST read the plan and actively check changed files. Any missing deferred, missing or changed features MUST be clearly presented to the user, with the reasoning included.
2. Archive the recently used plan for the session. The plan might be in `docs/plans`. The archive folder is `old-docs/archived-plans`.
3. If the plan was created through the skill `/wayfinder` (`.agents/skills/wayfinder/SKILL.md`), it will be associated with a ticket in `.scratch`. Rather than being archived, you need to follow the workflow in the wayfinder skill. 
4. Update all the documentation in the `docs` folder to match latest changes. Do not update documentation for the sake of updating as this results in sediment. See the skill `/chronicler-docs-hygiene` (`.agents/skills/chronicler-docs-hygiene/SKILL.md`) for standards.
5. Update all the unit and integration tests as needed for the latest changes.
6. Ensure that there is no 'ai slop' or 'hacks' in the code due to repetitive fixes without a cleanup.
7. Check if there is any duplicated code, any 'bad tests', any implemented or missing features.
    - Run `python scripts/healthcheck.py duplicates` to get a prioritized duplicate-code summary. For full options, run `python scripts/healthcheck.py duplicates --help`.
8. Check to make sure that the code is consistent with any existing patterns or, if the new pattern is an improvement, that older code is updated to match.
9. Run the `/code-simplification` skill against the (usually uncommitted) changes
10. Run the `/chronicler-comment-fixer` skill against the (usually uncommitted) changes. Sometimes comments are written in lieu of fixing issues, surface any comments like that for investigation.
11. Run the full build with the script `build.py`. **All Tests Must Pass**. Failing tests should be fixed even if they are failing for reasons that seem unrelated to the recent changes. "Seems unrelated" is a subjective opinion that is often wrong.
12. Run the 4 review subagents — `/thermo-nuclear-code-quality-review`, `/code-review` and `/antipattern-checker` under the read-only `reviewer` definition, `/test-police` under `implementer`, which re-runs the tests in its own target dir (see Concurrent Builds in `AGENTS.md`). Every definition disables skill discovery, so each brief carries its `SKILL.md` path; the read-only three also need the branch diff (`git diff` saved under `tmp/`) and the standards sources, because they hold no shell. Reviews answer inline — write all four findings files under `tmp/` yourself.
13. Report the results of the different reviews to the user: every finding, grouped by priority and de-duplicated. Verify each finding yourself — they are inputs, not conclusions, and some are invalid or aimed at the wrong thing.

This is intentionally a copy of `.agents/skills/chronicler-after-plan-workflow/SKILL.md`. Everything is the same except for the additional review step and removal of the code coverage step (handled by `test-police`).