# Scope the comment pass to the lines the diff adds

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`comment_finder.py --uncommitted` lists **every** comment in each touched file, not the comments the change added. A working tree that touched `build.py` returned 236 comment units against a change set that added about 11; 170 were pre-existing `build.py` docstrings. Three sessions independently reinvented the narrowing with `git diff -U0` and `tmp/added_comments*.txt`.

`chronicler-comment-fixer/SKILL.md:10` says "Pick the finder mode that matches the request. Its output is the scope." No mode produces added-lines output.

The 2026-10-04 reflect accepted the same finding ("give the finder a diff-line mode") and it never landed.

How should scope be defined?

- **Fix the finder.** `--uncommitted` and `--branch` emit only comment lines the diff adds or changes; the script's output is the scope.
- **Fix the skill text.** §1 states that a pass on an uncommitted change set judges only the added lines, and shows the `git diff -U0` narrowing.
- **Both.** The script removes the ambiguity; the text records the rule for other modes.

## Context

- Source: `/reflect` 2026-10-07, sessions `01a11774`, `01a108e0`, `01a108c6`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 4, amended by the parent to route through the script).
- `.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py:234` — `--uncommitted` selects files, then `find_comments_in_file` returns every comment.
- `.agents/skills/chronicler-comment-fixer/SKILL.md:10` — "Its output is the scope".
- Prior art: `tmp/reflect/final_01a1089d-5ec4.md` (2026-10-04 Accepted, unlanded) — "give the finder a diff-line mode".
