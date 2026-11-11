# The comment finder does not read `.mjs` or `.ts` files

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`comment_finder.py` reads `.rs`, `.py`, `.js`, `.html` and `.css` only. The views and scenarios of the verify skills are `.mjs`, and the `/statechart` scripts are `.ts`. Must the finder read them as JavaScript?

## Context

Source: `/retro` 2026-11-09, the session of statechart-models ticket 14 and its `/simple-after-plan` pass, with its subagent sessions (comment fixer `f4d7c890`, document review `20490787`, Standards review `383723c2`, Spec review `e2e6cad4`).

- **known** The comment fixer ran `comment_finder.py --files` on 14 files, 7 of them `.mjs`. The finder returned 2 lines. The implementer then listed the comments by hand from `git diff` and `rg`.
- **known** `UNCOMMITTED_EXTENSIONS = {".rs", ".py", ".js", ".html", ".css"}`, and `_comment_lines` has a branch for each of those suffixes only (`.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py`).

## Options to decide

- **A.** Add `.mjs` and `.ts` to `UNCOMMITTED_EXTENSIONS`, and read them with the `.js` branch of `_comment_lines`. Add a test for one `.mjs` file.
- **B.** Nothing: the comment fixer can list `.mjs` comments by hand.
