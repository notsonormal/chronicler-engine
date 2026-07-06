---
name: chronicler-after-plan-workflow
description: Chronicler skill used to update the chronicler tests and documentions after a successful plan
compatibility: opencode
metadata:
  language: rust
  workspace: chronicler_engine
---


# What I do

1. Verify that the implemention matches the existing plan. This is a post implementation verification so you MUST read the plan and actively check changed files. Any missing deferred, missing or changed features MUST be clearly presented to the user, with the reasoning included.
2. Archive the recently used plan which can usually be found at `local://PLAN_NAME_HERE.md` for the session. The plan might also be in `chronicler_engine/docs/plans`. The archive folder is `chronicler_engine/old-docs/archived-plans`.
3. Update all the documentation in the `chronicler_engine/docs` folder to match latest changes.
4. Update all the unit and integration tests as needed for the latest changes.
5. Check the code coverage and try to keet at 80% minimum for all files (run `build.py --coverage`).
6. Ensure that there is no 'ai slop' or 'hacks' in the code due to repetitive fixes without a cleanup.
7. Check if there is any duplicated code, any 'bad tests', any implemented or missing features.
   Run `python chronicler_engine/scripts/healthcheck.py duplicates` to get a prioritized duplicate-code summary.
   Feed the results into your own review (either as context for analysis or by asking the user).
   For full options, run `python chronicler_engine/scripts/healthcheck.py duplicates --help`.
8. Check to make sure that the code is consistent with any existing patterns or, if the new patterns is an improvement, that older code is updated to match
9. Run the full build with the script `chronicler_engine/build.py`. **All Tests Must Pass**. Failing tests should be fixed even if they are failing for reasons that seem unrelated to the recent changes. "Seems unrelated" is a subjective opinion that is often wrong.


_See `.agents/skills/_shared/chronicler-shared.md` for documentation sync and visual verification steps.
