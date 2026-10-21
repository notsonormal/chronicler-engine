---
name: simple-after-plan
description: Post-implementation pass — comment cleanup, doc review, code review, full build, then a retro
disable-model-invocation: true
---

Create a task list (using `TaskCreate`) for all these steps.

1. Work out the review scope from git state: uncommitted changes, a commit on top of a commit, or a branch on top of `main` or another branch. Use that same scope for every step: the `comment_finder.py` mode (for example `--uncommitted`, `--branch <parent>`, or `--files …`) and the `/code-review` fixed point.
2. In parallel, run `/chronicler-comment-fixer` (`.agents/skills/chronicler-comment-fixer/SKILL.md`) in an `implementer` subagent and `/document-review` (`.agents/skills/document-review/SKILL.md`) in a `reviewer` subagent over the changed `.md` files in scope. Each brief carries its `SKILL.md` path and the scope.
3. Wait for both subagents to finish. Verify each finding — findings are inputs, not conclusions. Fix each valid finding, and list each rejected finding with the reason.
4. Run `/code-review` (`.agents/skills/code-review/SKILL.md`) against the scope's fixed point. Handle its findings as in step 3.
5. Run the full gate once, here: `python build.py`. All tests must pass, including tests whose failure seems unrelated to the changes.
6. Run `/retro` (`.agents/skills/retro/SKILL.md`) on this session and on each subagent session (find them with `session_list`). Add new issues to `.scratch/retro-issue-bucket/` or amend existing ones. Read `/wayfinder` (`.agents/skills/wayfinder/SKILL.md`) first for the issue format.

## Slice mode

Use when the plan or the owner asks for it: one commit of a multi-slice build, done by an implementer subagent.

1. The implementer cleans its own comments and runs `python build.py --no-browser` once before it reports.
2. One `reviewer` checks spec, standards and comments/docs in one pass, from one bundle (`prepare_review_bundle.py --uncommitted`).
3. The fixes go back to the implementer's resumed session.
4. Run the full gate once: `python build.py`. Then commit.
5. Run the retro once per effort, not once per slice.
