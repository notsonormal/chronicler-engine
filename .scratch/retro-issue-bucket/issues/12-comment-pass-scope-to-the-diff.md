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

## User ruling (2026-10-11)

> The comments and doc reviews SHOULD catch older comments and document problems. Just like how updating code should fix existing code problems.

This rejects the question's premise. The scope is every comment in each file that the change touches, not only the lines that the diff adds. The finder already gives that scope (`comment_finder.py --uncommitted`), and `SKILL.md:10` already says "Its output is the scope". `document-review/SKILL.md:17` holds the same rule for docs.

**Evidence from the run that raised the ruling** (`/simple-after-plan` over tickets 15 and 16 of the dashboard UI review phase 2, lead session `01a12cf3-c68e-7568-beff-572aec227289`):

- The pass changed 56 comments in 40 touched files. Most of them were older than the change, for example the migration headers in `plumbing.rs` (v19–v23). That is the intended behaviour.
- The spec-axis code review reported those edits as scope creep (finding 5, session `01a12d31-7858-74b1-b842-ce6d120162ef`). Under this ruling that finding is wrong. The gap is in the code-review spec brief (`code-review/SKILL.md:76`, "(b) behaviour in the diff that wasn't asked for (scope creep)"), which does not say that cleanup in touched files is expected.
- The pass also deleted two lines of code that look like comments: [Stop the comment pass from deleting code inside string literals](49-comment-pass-code-inside-string-literals.md). Under this ruling the fix belongs there, not in a narrower scope.

**Recurrence, 2026-10-17** (`/simple-after-plan` on slice P0 of the Rust rebuild build plan; lead session in `.scratch/rust-rebuild-build/`). Both code-review axes again reported the comment pass's `build.py` docstring edits as churn (standards finding 8; spec finding (b)). The lead added one sentence to both briefs in `code-review/SKILL.md`: cleanup in touched files is expected, not scope creep. The open half of this ticket is the finder's handling of text inside string literals: [Stop the comment pass from deleting code inside string literals](49-comment-pass-code-inside-string-literals.md).
