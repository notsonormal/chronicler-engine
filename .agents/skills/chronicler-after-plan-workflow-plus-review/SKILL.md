---
name: chronicler-after-plan-workflow-plus-review
description: The post-implementation pass, plus the four review subagents and the consolidated findings report
disable-model-invocation: true
---


# What I do

Create a task list (using `TaskCreate`) for all these steps.

1. Verify that the implementation matches the existing plan. This is a post implementation verification so you must read the plan and actively check changed files. Any deferred, missing or changed features must be clearly presented to the user, with the reasoning included.
2. Archive the recently used plan for the session. The plan might be in `docs/plans`. The archive folder is `old-docs/archived-plans`.
3. If the plan was created through the skill `/wayfinder` (`.agents/skills/wayfinder/SKILL.md`), it will be associated with a ticket in `.scratch`. Rather than being archived, you need to follow the workflow in the wayfinder skill. 
4. Update all the unit and integration tests as needed for the latest changes.
5. Ensure that there is no 'ai slop' or 'hacks' in the code due to repetitive fixes without a cleanup.
6. Check if there is any duplicated code, any 'bad tests', any implemented or missing features.
    - The full gate already reports duplicate-code clones scoped to files changed vs `main` (the `Checking for duplicate code...` step). Read that summary from the gate log instead of running the script again, so the two scopes cannot disagree. For a whole-repo view or a different base ref, run `python scripts/healthcheck.py duplicates --all` or `python scripts/healthcheck.py duplicates --ref <ref>` (full options: `--help`).
7. Check to make sure that the code is consistent with any existing patterns or, if the new pattern is an improvement, that older code is updated to match.
8. Update the docs in `docs/` that the changes make false. Do this after the steps above that change code. A behaviour change edits its spec in `docs/specs/`. A doc in `docs/diataxis/` changes only when the changes add, remove or rename a part, an invariant or a name that the doc states (`docs/AGENTS.md` §Where a fact lives). Standards: `/chronicler-docs-hygiene` (`.agents/skills/chronicler-docs-hygiene/SKILL.md`).
9. Run the `/chronicler-comment-fixer` skill against the (usually uncommitted) changes. Sometimes comments are written in lieu of fixing issues, surface any comments like that for investigation.
10. Run the full build with the script `build.py`. All tests must pass. Failing tests should be fixed even if they are failing for reasons that seem unrelated to the recent changes. "Seems unrelated" is a subjective opinion that is often wrong.
11. Run the 4 review subagents — `/thermo-nuclear-code-quality-review`, `/code-review` and `/antipattern-checker` under the read-only `reviewer` definition, `/test-police` under `implementer`, which re-runs the tests in its own target dir (see `ENVIRONMENT.md`). Each brief carries its `SKILL.md` path; the read-only three also get the standards sources and one fixed diff. Build that diff with `python scripts/prepare_review_bundle.py --ref <fixed-point>` (or `--uncommitted` for work in progress) and pass the printed `tmp/review/...` bundle path to each read-only subagent, so all three review the same diff even if the tree changes. The bundle replaces each reviewer skill's own diff-acquisition step. Reviews answer inline — write all four findings files under `tmp/` yourself.
12. Report the results of the different reviews to the user: every finding, grouped by priority and de-duplicated. Verify each finding yourself — they are inputs, not conclusions, and some are invalid or aimed at the wrong thing.

This is intentionally a copy of `.agents/skills/chronicler-after-plan-workflow/SKILL.md`. Everything is the same except for the additional review step and removal of the code coverage step (handled by `test-police`).