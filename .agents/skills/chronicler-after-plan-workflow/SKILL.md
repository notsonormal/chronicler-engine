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
2. Archive the recently used plan which can usually be found at `local://PLAN_NAME_HERE.md` for the session. The plan might also be in `chronicler_engine/docs/plans`. The archive folder is `chronicler_engine/docs/plans/archived`.
3. Update all the documentation in the `chronicler_engine/docs` folder to match latest changes.
4. Update all the unit and integration tests as needed for the latest changes. 
5. Check the code coverage and try to keet at 80% minimum for all files (run `build.py --coverage`).
6. Ensure that there is no 'ai slop' or 'hacks' in the code due to repetitive fixes without a cleanup.
7. Check if there is any duplicated code, any 'bad tests', any implemented or missing features. 
8. Check to make sure that the code is consistent with any existing patterns or, if the new patterns is an improvement, that older code is updated to match
9. Run the full build with the script `chronicler_engine/build.py`. **All Tests Must Pass**. Failing tests should be fixed even if they are failing for reasons that seem unrelated to the recent changes. "Seems unrelated" is a subjective opinion that is often wrong. 

## Documentation Sync

When implementing changes, ALWAYS update these documents BEFORE writing code:

1. **Plan** (`chronicler_engine/docs/plans/<name>.md`) - Document problem, solution, files to change
2. **Architecture** (`chronicler_engine/docs/architecture/system.md`) - Core module structure changes
3. **System docs** (`chronicler_engine/docs/system/*.md`) - Domain-specific docs (dashboard.md, game_flow.md, etc.)
4. **Reference docs** (`chronicler_engine/docs/reference/*.md`) - Data schemas, API specs
5. **ADRs** (`chronicler_engine/docs/adr/*.md`) - If making architectural decisions
6. **CHANGELOG.md** - Record the change (Date the change was made, not released. Also remove concat and eventually remove older entries if the file gets too large)

Example: If adding a new HTMX polling endpoint, update:
- Plan document
- architecture/system.md (Server tier)
- system/dashboard.md (frontend section)
- reference/testing.md (add test)
- CHANGELOG.md

## Visual/UI Verification (MANDATORY)

For any CSS, layout, or visual changes:

1. **Rebuild** the project after CSS changes
2. **Restart** the server with the new build
3. **Navigate** to the affected page in the browser
4. **Take a screenshot** and **actually look at it** — do not skip this step
5. **Confirm visually** that the change renders correctly before claiming done

**NEVER claim "verified" or "browser verification complete" without a screenshot that you have personally reviewed.** Subagent verification does not count — you must see the rendered result yourself.